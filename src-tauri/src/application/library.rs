//! Visão da biblioteca: estado local de cada save combinado com o último resultado de sync.
//! Não acessa a rede; o status remoto é atualizado a cada sincronização.

use std::{collections::HashSet, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    domain::{
        models::{BUSY_MESSAGE, SourceInfo, SyncStateFile, SyncUnit, UnitKind, UnitStatus, UnitSyncState},
        ports::{RegisteredSource, SyncStateRepository},
    },
    error::AppResult,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitView {
    pub id: String,
    pub source_id: String,
    pub group: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    pub kind: UnitKind,
    pub exists_locally: bool,
    pub size: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub status: UnitStatus,
    pub message: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub remote_machine: Option<String>,
    /// Conflito porque o save foi excluído em outra máquina.
    pub remote_deleted: bool,
    /// Caminho local da capa do jogo (lido da pasta de capas do emulador).
    pub cover_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceOverview {
    pub info: SourceInfo,
    pub enabled: bool,
    pub auto_sync: bool,
    pub scan_error: Option<String>,
    pub units: Vec<UnitView>,
}

pub fn overview(registered: &RegisteredSource, state_repo: &dyn SyncStateRepository) -> AppResult<SourceOverview> {
    let source = registered.source.as_ref();
    let state = state_repo.load()?;

    let (units, scan_error) = if registered.enabled {
        match source.scan() {
            Ok(scan) => (scan.units, None),
            Err(err) => (Vec::new(), Some(err.to_string())),
        }
    } else {
        (Vec::new(), None)
    };

    let mut covers = source.find_covers(&units);
    let mut views: Vec<UnitView> = units
        .iter()
        .map(|u| UnitView { cover_path: covers.remove(&u.id), ..local_view(u, state.units.get(&u.id)) })
        .collect();
    if registered.enabled {
        views.extend(remote_only_views(source.id(), &units, &state));
    }

    Ok(SourceOverview {
        info: source.info(),
        enabled: registered.enabled,
        auto_sync: registered.auto_sync,
        scan_error,
        units: views,
    })
}

fn local_view(unit: &SyncUnit, state: Option<&UnitSyncState>) -> UnitView {
    let hash = unit.content_hash();
    let (status, message) = if unit.busy {
        (UnitStatus::Deferred, Some(BUSY_MESSAGE.to_string()))
    } else {
        (derive_status(hash.as_deref(), state), state.and_then(|s| s.message.clone()))
    };
    UnitView {
        id: unit.id.clone(),
        source_id: unit.source_id.clone(),
        group: unit.group.clone(),
        title: unit.title.clone(),
        subtitle: unit.subtitle.clone(),
        tags: unit.tags.clone(),
        kind: unit.kind,
        exists_locally: unit.exists_locally,
        size: unit.size(),
        modified_at: unit.modified_at,
        status,
        message,
        last_synced_at: state.and_then(|s| s.last_synced_at),
        remote_machine: state.and_then(|s| s.remote_machine.clone()),
        remote_deleted: state.is_some_and(|s| s.remote_deleted),
        cover_path: None,
    }
}

/// Saves conhecidos apenas pela nuvem que ainda aguardam ação (ex.: download adiado).
fn remote_only_views(source_id: &str, local: &[SyncUnit], state: &SyncStateFile) -> Vec<UnitView> {
    let prefix = format!("{source_id}/");
    let local_ids: HashSet<&str> = local.iter().map(|u| u.id.as_str()).collect();
    let mut views: Vec<UnitView> = state
        .units
        .iter()
        .filter(|(id, s)| {
            id.starts_with(&prefix)
                && !local_ids.contains(id.as_str())
                && matches!(s.status, UnitStatus::Deferred | UnitStatus::Conflict | UnitStatus::Error)
        })
        .map(|(id, s)| UnitView {
            id: id.clone(),
            source_id: source_id.into(),
            group: s.group.clone(),
            title: if s.title.is_empty() { id.rsplit('/').next().unwrap_or(id).into() } else { s.title.clone() },
            subtitle: Some("Somente na nuvem".into()),
            tags: Vec::new(),
            kind: s.kind.unwrap_or(UnitKind::File),
            exists_locally: false,
            size: 0,
            modified_at: None,
            status: s.status,
            message: s.message.clone(),
            last_synced_at: s.last_synced_at,
            remote_machine: s.remote_machine.clone(),
            remote_deleted: s.remote_deleted,
            cover_path: None,
        })
        .collect();
    views.sort_by(|a, b| a.id.cmp(&b.id));
    views
}

fn derive_status(local_hash: Option<&str>, state: Option<&UnitSyncState>) -> UnitStatus {
    match state {
        None if local_hash.is_some() => UnitStatus::Pending,
        None => UnitStatus::Unknown,
        Some(s) => match s.status {
            UnitStatus::Conflict | UnitStatus::Deferred | UnitStatus::Error => s.status,
            _ if s.base_hash.as_deref() == local_hash => UnitStatus::Synced,
            _ => UnitStatus::Pending,
        },
    }
}
