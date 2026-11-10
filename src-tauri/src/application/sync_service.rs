use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use chrono::Utc;
use tokio::sync::Mutex;

use crate::{
    domain::{
        hashing::{content_hash, sha256_hex},
        models::*,
        ports::*,
        sync_policy::{self, RemoteState},
    },
    error::{AppError, AppResult},
};

/// Dependências injetadas no serviço de sync.
pub struct SyncDeps {
    pub storage_factory: Arc<dyn CloudStorageFactory>,
    pub settings: Arc<dyn SettingsRepository>,
    pub state: Arc<dyn SyncStateRepository>,
    pub activity: Arc<dyn ActivityLog>,
    pub local: Arc<dyn LocalUnitStore>,
    pub backups: Arc<dyn BackupStore>,
    pub process: Arc<dyn ProcessMonitor>,
    pub events: Arc<dyn SyncEventSink>,
    /// Nome desta máquina, gravado nos manifests.
    pub machine: String,
}

pub struct SyncService {
    deps: SyncDeps,
    /// Garante uma sincronização por vez (manual, watcher e retry compartilham a fila).
    lock: Mutex<()>,
}

/// Contexto imutável de uma execução de sync.
struct RunContext {
    storage: Arc<dyn CloudStorage>,
    cloud: CloudConfig,
    backups_to_keep: usize,
    emulator_running: bool,
}

impl RunContext {
    fn manifest_key(&self, unit_id: &str) -> String {
        self.cloud.key(&format!("{unit_id}/manifest.json"))
    }

    fn blob_key(&self, unit_id: &str, sha256: &str) -> String {
        self.cloud.key(&format!("{unit_id}/blobs/{sha256}"))
    }
}

enum Outcome {
    Skipped,
    InSync,
    Uploaded,
    Downloaded,
    Deleted,
    Conflict,
    Deferred,
}

/// Operações que disputam o mesmo lock (uma por vez).
#[derive(Clone, Copy)]
enum Job<'a> {
    Sync { trigger: SyncTrigger, only: Option<(&'a str, Resolution)> },
    Delete { unit_id: &'a str },
}

impl SyncService {
    pub fn new(deps: SyncDeps) -> Self {
        Self { deps, lock: Mutex::new(()) }
    }

    pub fn state(&self) -> &Arc<dyn SyncStateRepository> {
        &self.deps.state
    }

    /// Sincroniza todas as unidades de uma fonte.
    pub async fn sync_source(&self, source: &dyn SaveSource, trigger: SyncTrigger) -> AppResult<SyncSummary> {
        self.execute(source, Job::Sync { trigger, only: None }).await
    }

    /// Exclui o save desta máquina e da nuvem, deixando uma lápide para as outras máquinas.
    pub async fn delete_unit(&self, source: &dyn SaveSource, unit_id: &str) -> AppResult<SyncSummary> {
        self.execute(source, Job::Delete { unit_id }).await
    }

    /// Resolve um conflito forçando a versão escolhida pelo usuário.
    pub async fn resolve_conflict(
        &self,
        source: &dyn SaveSource,
        unit_id: &str,
        resolution: Resolution,
    ) -> AppResult<SyncSummary> {
        self.execute(source, Job::Sync { trigger: SyncTrigger::Resolution, only: Some((unit_id, resolution)) })
            .await
    }

    /// Há downloads adiados que podem ser feitos agora (emulador fechado)?
    pub fn should_retry_deferred(&self, source: &dyn SaveSource) -> bool {
        let prefix = format!("{}/", source.id());
        let has_deferred = self
            .deps
            .state
            .load()
            .map(|s| {
                s.units
                    .iter()
                    .any(|(id, u)| id.starts_with(&prefix) && u.status == UnitStatus::Deferred)
            })
            .unwrap_or(false);
        has_deferred && !self.deps.process.is_any_running(source.process_names())
    }

    async fn execute(&self, source: &dyn SaveSource, job: Job<'_>) -> AppResult<SyncSummary> {
        let _guard = self.lock.lock().await;
        let mut summary = SyncSummary { source_id: source.id().into(), ..Default::default() };

        let result = match job {
            Job::Sync { trigger, only } => self.run(source, trigger, only, &mut summary).await,
            Job::Delete { unit_id } => self.run_delete(source, unit_id, &mut summary).await,
        };
        if let Err(err) = &result {
            summary.error = Some(err.to_string());
            self.log(source.id(), None, &source.info().name, ActivityKind::Error, err.to_string());
        }
        summary.finished_at = Some(Utc::now());
        self.deps.events.emit(SyncEvent::Finished { summary: summary.clone() });
        result.map(|_| summary)
    }

    async fn run(
        &self,
        source: &dyn SaveSource,
        trigger: SyncTrigger,
        only: Option<(&str, Resolution)>,
        summary: &mut SyncSummary,
    ) -> AppResult<()> {
        let ctx = self.context(source)?;
        let mut units = self.collect_units(source, &ctx).await?;
        if let Some((unit_id, _)) = only {
            units.retain(|u| u.id == unit_id);
            if units.is_empty() {
                return Err(AppError::NotFound(format!("Save '{unit_id}' não encontrado.")));
            }
        }

        let total = units.len();
        self.deps.events.emit(SyncEvent::Started { source_id: source.id().into(), trigger, total });

        let mut state = self.deps.state.load()?;
        if only.is_none() {
            prune_vanished(&mut state, source.id(), &units);
            self.deps.state.save(&state)?;
        }
        for (index, unit) in units.iter().enumerate() {
            self.emit_unit(unit, UnitPhase::Checking, None, index, total);

            let status = match self.sync_unit(&ctx, unit, &mut state, only.map(|(_, r)| r), index, total).await {
                Ok(outcome) => {
                    match outcome {
                        Outcome::Uploaded => summary.uploaded += 1,
                        Outcome::Downloaded => summary.downloaded += 1,
                        Outcome::Deleted => summary.deleted += 1,
                        Outcome::InSync => summary.in_sync += 1,
                        Outcome::Conflict => summary.conflicts += 1,
                        Outcome::Deferred => summary.deferred += 1,
                        Outcome::Skipped => {}
                    }
                    state.units.get(&unit.id).map(|s| s.status)
                }
                Err(err) => {
                    summary.errors += 1;
                    let entry = state.units.entry(unit.id.clone()).or_default();
                    entry.status = UnitStatus::Error;
                    entry.message = Some(err.to_string());
                    self.log(&unit.source_id, Some(&unit.id), &unit.title, ActivityKind::Error, err.to_string());
                    Some(UnitStatus::Error)
                }
            };

            self.deps.state.save(&state)?;
            self.emit_unit(unit, UnitPhase::Done, status, index, total);
        }
        Ok(())
    }

    fn context(&self, source: &dyn SaveSource) -> AppResult<RunContext> {
        let settings = self.deps.settings.load()?;
        let cloud = settings.cloud.clone().ok_or(AppError::CloudNotConfigured)?;
        cloud.validate()?;
        Ok(RunContext {
            storage: self.deps.storage_factory.create(&cloud)?,
            cloud,
            backups_to_keep: settings.backups_to_keep,
            emulator_running: self.deps.process.is_any_running(source.process_names()),
        })
    }

    /// Exclusão pedida pelo usuário: lápide na nuvem (commit), blobs removidos e cópia local
    /// apagada com backup. As outras máquinas aplicam a exclusão no próximo sync.
    async fn run_delete(&self, source: &dyn SaveSource, unit_id: &str, summary: &mut SyncSummary) -> AppResult<()> {
        let ctx = self.context(source)?;
        if ctx.emulator_running {
            return Err(AppError::Validation("Feche o emulador antes de excluir um save.".into()));
        }
        let unit = self
            .collect_units(source, &ctx)
            .await?
            .into_iter()
            .find(|u| u.id == unit_id)
            .ok_or_else(|| AppError::NotFound(format!("Save '{unit_id}' não encontrado.")))?;
        if unit.busy {
            return Err(AppError::Validation("O save está em uso pelo emulador.".into()));
        }

        self.deps.events.emit(SyncEvent::Started {
            source_id: source.id().into(),
            trigger: SyncTrigger::Deletion,
            total: 1,
        });
        self.emit_unit(&unit, UnitPhase::Deleting, None, 0, 1);

        let remote = self.fetch_manifest(&ctx, &unit.id).await?;
        let title = match (&remote, unit.exists_locally) {
            (Some(m), false) => m.title.clone(),
            _ => unit.title.clone(),
        };

        // 1. A lápide é o commit: a partir daqui a exclusão vale para todas as máquinas.
        let tombstone = RemoteManifest::tombstone(&unit, title.clone(), self.deps.machine.clone());
        ctx.storage
            .put_object(&ctx.manifest_key(&unit.id), serde_json::to_vec_pretty(&tombstone)?, &[])
            .await?;

        // 2. Conteúdo na nuvem. Falhar aqui só deixa lixo no bucket.
        if let Some(old) = &remote {
            let blobs: HashSet<&str> = old.files.iter().map(|f| f.sha256.as_str()).collect();
            for sha in blobs {
                let _ = ctx.storage.delete_object(&ctx.blob_key(&unit.id, sha)).await;
            }
        }

        // 3. Cópia local, com backup. Se falhar, o próximo sync tenta de novo pela lápide.
        if unit.exists_locally {
            self.remove_local(&ctx, &unit)?;
        }

        let mut state = self.deps.state.load()?;
        state.units.remove(&unit.id);
        self.deps.state.save(&state)?;

        let message = if unit.exists_locally {
            "Excluído desta máquina e da nuvem. Uma cópia ficou nos backups."
        } else {
            "Excluído da nuvem."
        };
        self.log(&unit.source_id, Some(&unit.id), &title, ActivityKind::Delete, message);
        summary.deleted = 1;
        self.emit_unit(&unit, UnitPhase::Done, None, 0, 1);
        Ok(())
    }

    fn remove_local(&self, ctx: &RunContext, unit: &SyncUnit) -> AppResult<()> {
        self.deps.backups.backup(unit, ctx.backups_to_keep)?;
        self.deps.local.remove_unit(unit)
    }

    /// Unidades locais + unidades que só existem na nuvem (ex.: jogo salvo em outra máquina).
    async fn collect_units(&self, source: &dyn SaveSource, ctx: &RunContext) -> AppResult<Vec<SyncUnit>> {
        let scan = source.scan()?;
        let mut units = scan.units;
        let mut known: HashSet<String> = units.iter().map(|u| u.id.clone()).collect();

        for scope in &scan.remote_scopes {
            let full_prefix = ctx.cloud.key(&scope.id_prefix);
            for key in ctx.storage.list_keys(&full_prefix).await? {
                let Some(name) = key
                    .strip_prefix(&full_prefix)
                    .and_then(|rest| rest.strip_suffix("/manifest.json"))
                else {
                    continue;
                };
                if name.is_empty() || name.contains('/') || name.starts_with('.') {
                    continue;
                }
                let id = format!("{}{}", scope.id_prefix, name);
                if !known.insert(id.clone()) {
                    continue;
                }
                units.push(SyncUnit {
                    id,
                    source_id: source.id().into(),
                    group: scope.group.clone().unwrap_or_else(|| name.into()),
                    title: name.into(),
                    subtitle: Some("Somente na nuvem".into()),
                    tags: Vec::new(),
                    kind: scope.kind,
                    local_path: scope.local_parent.join(format!("{name}{}", scope.local_suffix)),
                    files: Vec::new(),
                    exists_locally: false,
                    modified_at: None,
                    busy: false,
                });
            }
        }
        Ok(units)
    }

    async fn sync_unit(
        &self,
        ctx: &RunContext,
        unit: &SyncUnit,
        state: &mut SyncStateFile,
        forced: Option<Resolution>,
        index: usize,
        total: usize,
    ) -> AppResult<Outcome> {
        // Arquivos travados pelo emulador: tenta de novo quando forem liberados.
        if unit.busy {
            return Ok(self.defer_busy(state, unit));
        }

        let remote = self.fetch_manifest(ctx, &unit.id).await?;
        let local_hash = unit.content_hash();
        let remote_state = match &remote {
            None => RemoteState::Missing,
            Some(m) if m.deleted => RemoteState::Deleted,
            Some(m) => RemoteState::Live(&m.hash),
        };

        let entry = state.units.entry(unit.id.clone()).or_default();
        let action = match (forced, remote_state) {
            (Some(Resolution::KeepLocal), _) if unit.exists_locally => SyncAction::Upload,
            (Some(Resolution::KeepCloud), RemoteState::Live(_)) => SyncAction::Download,
            (Some(Resolution::KeepCloud), RemoteState::Deleted) if unit.exists_locally => SyncAction::DeleteLocal,
            (Some(_), _) => return Err(AppError::Validation("A versão escolhida não está disponível.".into())),
            (None, _) => sync_policy::decide(local_hash.as_deref(), remote_state, entry.base_hash.as_deref()),
        };

        entry.title = match (&remote, unit.exists_locally) {
            (Some(m), false) => m.title.clone(),
            _ => unit.title.clone(),
        };
        entry.group = unit.group.clone();
        entry.kind = Some(unit.kind);

        match action {
            SyncAction::Nothing => {
                state.units.remove(&unit.id);
                Ok(Outcome::Skipped)
            }
            SyncAction::InSync => {
                mark_synced(entry, local_hash, remote.as_ref().map(|m| m.machine.clone()));
                Ok(Outcome::InSync)
            }
            SyncAction::Upload => {
                self.emit_unit(unit, UnitPhase::Uploading, None, index, total);
                let manifest = match self.upload(ctx, unit, remote.as_ref()).await {
                    Ok(manifest) => manifest,
                    Err(err) if err.is_file_in_use() => return Ok(self.defer_busy(state, unit)),
                    Err(err) => return Err(err),
                };
                let entry = state.units.entry(unit.id.clone()).or_default();
                mark_synced(entry, Some(manifest.hash), Some(manifest.machine));
                self.log(&unit.source_id, Some(&unit.id), &unit.title, ActivityKind::Upload, "Enviado para a nuvem.");
                Ok(Outcome::Uploaded)
            }
            SyncAction::Download => {
                let remote = remote.expect("decide() só retorna Download quando existe versão remota");
                if ctx.emulator_running {
                    if entry.status != UnitStatus::Deferred {
                        self.log(
                            &unit.source_id,
                            Some(&unit.id),
                            &entry.title,
                            ActivityKind::Deferred,
                            "Nova versão na nuvem. Feche o emulador para baixar.",
                        );
                    }
                    entry.status = UnitStatus::Deferred;
                    entry.message = Some("Feche o emulador para baixar a versão da nuvem.".into());
                    return Ok(Outcome::Deferred);
                }
                self.emit_unit(unit, UnitPhase::Downloading, None, index, total);
                self.download(ctx, unit, &remote).await?;
                let message = format!("Baixado da nuvem (enviado por {}).", remote.machine);
                mark_synced(entry, Some(remote.hash), Some(remote.machine));
                self.log(&unit.source_id, Some(&unit.id), &entry.title, ActivityKind::Download, message);
                Ok(Outcome::Downloaded)
            }
            SyncAction::DeleteLocal => {
                let machine = remote.as_ref().map(|m| m.machine.clone()).unwrap_or_default();
                if ctx.emulator_running {
                    if entry.status != UnitStatus::Deferred {
                        self.log(
                            &unit.source_id,
                            Some(&unit.id),
                            &unit.title,
                            ActivityKind::Deferred,
                            "Excluído na nuvem. Feche o emulador para apagar daqui.",
                        );
                    }
                    entry.status = UnitStatus::Deferred;
                    entry.message = Some(format!("Excluído em {machine}. Feche o emulador para apagar daqui."));
                    return Ok(Outcome::Deferred);
                }
                self.emit_unit(unit, UnitPhase::Deleting, None, index, total);
                self.remove_local(ctx, unit)?;
                state.units.remove(&unit.id);
                self.log(
                    &unit.source_id,
                    Some(&unit.id),
                    &unit.title,
                    ActivityKind::Delete,
                    format!("Excluído em {machine}; apagado daqui também. Uma cópia ficou nos backups."),
                );
                Ok(Outcome::Deleted)
            }
            SyncAction::Conflict => {
                let remote_deleted = remote_state == RemoteState::Deleted;
                if entry.status != UnitStatus::Conflict {
                    let message = if remote_deleted {
                        "Excluído em outra máquina, mas alterado aqui. Escolha se mantém ou exclui."
                    } else {
                        "Alterado nesta máquina e na nuvem. Escolha qual versão manter."
                    };
                    self.log(&unit.source_id, Some(&unit.id), &unit.title, ActivityKind::Conflict, message);
                }
                let machine = remote.as_ref().map(|m| m.machine.clone());
                let by = machine.as_deref().map(|m| format!(" (por {m})")).unwrap_or_default();
                entry.status = UnitStatus::Conflict;
                entry.message = Some(if remote_deleted {
                    format!("Excluído em outra máquina{by}, mas alterado aqui.")
                } else {
                    format!("Alterado aqui e na nuvem{by}.")
                });
                entry.remote_machine = machine;
                entry.remote_deleted = remote_deleted;
                Ok(Outcome::Conflict)
            }
        }
    }

    fn defer_busy(&self, state: &mut SyncStateFile, unit: &SyncUnit) -> Outcome {
        let entry = state.units.entry(unit.id.clone()).or_default();
        if entry.status != UnitStatus::Deferred {
            self.log(&unit.source_id, Some(&unit.id), &unit.title, ActivityKind::Deferred, BUSY_MESSAGE);
        }
        entry.status = UnitStatus::Deferred;
        entry.message = Some(BUSY_MESSAGE.into());
        entry.title = unit.title.clone();
        entry.group = unit.group.clone();
        entry.kind = Some(unit.kind);
        Outcome::Deferred
    }

    async fn fetch_manifest(&self, ctx: &RunContext, unit_id: &str) -> AppResult<Option<RemoteManifest>> {
        let Some(bytes) = ctx.storage.get_object(&ctx.manifest_key(unit_id)).await? else {
            return Ok(None);
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|e| AppError::Integrity(format!("Manifest inválido na nuvem para '{unit_id}': {e}")))
    }

    /// Envia blobs novos (endereçados por SHA-256), grava o manifest por último
    /// (commit atômico) e remove blobs que deixaram de ser referenciados.
    async fn upload(
        &self,
        ctx: &RunContext,
        unit: &SyncUnit,
        remote: Option<&RemoteManifest>,
    ) -> AppResult<RemoteManifest> {
        let remote_blobs: HashSet<&str> = remote
            .map(|m| m.files.iter().map(|f| f.sha256.as_str()).collect())
            .unwrap_or_default();

        let mut sent = HashSet::new();
        for file in &unit.files {
            if remote_blobs.contains(file.sha256.as_str()) || !sent.insert(file.sha256.as_str()) {
                continue;
            }
            let bytes = self.deps.local.read_file(unit, &file.path)?;
            if sha256_hex(&bytes) != file.sha256 {
                return Err(AppError::Integrity(format!(
                    "'{}' mudou durante o envio; será reenviado no próximo sync.",
                    file.path
                )));
            }
            ctx.storage
                .put_object(&ctx.blob_key(&unit.id, &file.sha256), bytes, &[("sha256", file.sha256.clone())])
                .await?;
        }

        let manifest = RemoteManifest {
            version: 1,
            unit_id: unit.id.clone(),
            kind: unit.kind,
            title: unit.title.clone(),
            hash: content_hash(&unit.files),
            files: unit.files.clone(),
            updated_at: Utc::now(),
            machine: self.deps.machine.clone(),
            deleted: false,
        };
        ctx.storage
            .put_object(&ctx.manifest_key(&unit.id), serde_json::to_vec_pretty(&manifest)?, &[])
            .await?;

        if let Some(old) = remote {
            let referenced: HashSet<&str> = manifest.files.iter().map(|f| f.sha256.as_str()).collect();
            let orphans: HashSet<&str> = old
                .files
                .iter()
                .map(|f| f.sha256.as_str())
                .filter(|sha| !referenced.contains(sha))
                .collect();
            for sha in orphans {
                // Falhar aqui só deixa lixo no bucket; não compromete o sync.
                let _ = ctx.storage.delete_object(&ctx.blob_key(&unit.id, sha)).await;
            }
        }
        Ok(manifest)
    }

    /// Monta a versão remota completa em memória, valida cada hash, faz backup
    /// da versão local e só então substitui os arquivos de forma atômica.
    async fn download(&self, ctx: &RunContext, unit: &SyncUnit, remote: &RemoteManifest) -> AppResult<()> {
        if remote.kind != unit.kind {
            return Err(AppError::Integrity(format!(
                "O formato na nuvem ({:?}) difere do local ({:?}).",
                remote.kind, unit.kind
            )));
        }

        let local_files: HashMap<&str, &str> = unit
            .files
            .iter()
            .map(|f| (f.path.as_str(), f.sha256.as_str()))
            .collect();
        let mut downloaded: HashMap<&str, Vec<u8>> = HashMap::new();
        let mut files = Vec::with_capacity(remote.files.len());

        for file in &remote.files {
            let bytes = if local_files.get(file.path.as_str()) == Some(&file.sha256.as_str()) {
                self.deps.local.read_file(unit, &file.path)?
            } else if let Some(bytes) = downloaded.get(file.sha256.as_str()) {
                bytes.clone()
            } else {
                let bytes = ctx
                    .storage
                    .get_object(&ctx.blob_key(&unit.id, &file.sha256))
                    .await?
                    .ok_or_else(|| AppError::Integrity(format!("'{}' não foi encontrado na nuvem.", file.path)))?;
                downloaded.insert(file.sha256.as_str(), bytes.clone());
                bytes
            };

            if sha256_hex(&bytes) != file.sha256 {
                return Err(AppError::Integrity(format!("'{}' está corrompido na nuvem.", file.path)));
            }
            files.push((file.clone(), bytes));
        }

        if unit.exists_locally {
            self.deps.backups.backup(unit, ctx.backups_to_keep)?;
        }
        self.deps.local.replace_unit(unit, files)
    }

    fn emit_unit(&self, unit: &SyncUnit, phase: UnitPhase, status: Option<UnitStatus>, index: usize, total: usize) {
        self.deps.events.emit(SyncEvent::Unit {
            source_id: unit.source_id.clone(),
            unit_id: unit.id.clone(),
            title: unit.title.clone(),
            phase,
            status,
            index,
            total,
        });
    }

    fn log(&self, source_id: &str, unit_id: Option<&str>, title: &str, kind: ActivityKind, message: impl Into<String>) {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let now = Utc::now();
        let entry = ActivityEntry {
            id: format!("{}-{}", now.timestamp_millis(), SEQ.fetch_add(1, Ordering::Relaxed)),
            at: now,
            source_id: source_id.into(),
            unit_id: unit_id.map(Into::into),
            title: title.into(),
            kind,
            message: message.into(),
        };
        if let Err(err) = self.deps.activity.append(entry) {
            eprintln!("[altar-save] falha ao gravar atividade: {err}");
        }
    }
}

/// Remove do estado os saves desta fonte que não existem mais nem localmente nem na nuvem
/// (ex.: o usuário apagou o memory card nos dois lados).
fn prune_vanished(state: &mut SyncStateFile, source_id: &str, units: &[SyncUnit]) {
    let prefix = format!("{source_id}/");
    let present: HashSet<&str> = units.iter().map(|u| u.id.as_str()).collect();
    state
        .units
        .retain(|id, _| !id.starts_with(&prefix) || present.contains(id.as_str()));
}

fn mark_synced(entry: &mut UnitSyncState, hash: Option<String>, machine: Option<String>) {
    entry.base_hash = hash;
    entry.last_synced_at = Some(Utc::now());
    entry.status = UnitStatus::Synced;
    entry.message = None;
    entry.remote_machine = machine;
    entry.remote_deleted = false;
}
