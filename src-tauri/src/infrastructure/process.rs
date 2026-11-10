use std::{path::PathBuf, sync::Mutex};

use sysinfo::{Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::domain::ports::ProcessMonitor;

/// Detecta emuladores em execução pelo nome do executável.
pub struct SysinfoProcessMonitor {
    /// `None` se o sistema não permitir consultar processos (então assume "não está rodando").
    system: Mutex<Option<System>>,
}

impl Default for SysinfoProcessMonitor {
    fn default() -> Self {
        let system = System::new()
            .inspect_err(|e| eprintln!("[save-nuvem] detecção de processos indisponível: {e}"))
            .ok();
        Self { system: Mutex::new(system) }
    }
}

impl SysinfoProcessMonitor {
    /// Executa `f` sobre os processos que contêm algum dos `names` no nome do executável.
    fn with_matching<R>(&self, names: &[&str], f: impl FnOnce(&mut dyn Iterator<Item = &Process>) -> R) -> Option<R> {
        let mut guard = self.system.lock().unwrap_or_else(|e| e.into_inner());
        let system = guard.as_mut()?;
        // Apenas nome e caminho do executável: bem mais barato que um refresh completo.
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        );
        let mut matching = system.processes().values().filter(|process| {
            let name = process.name().to_string_lossy().to_lowercase();
            names.iter().any(|n| name.contains(n))
        });
        Some(f(&mut matching))
    }

    /// Caminhos dos executáveis em execução (ex.: para achar instalações portáteis).
    pub fn running_executables(&self, names: &[&str]) -> Vec<PathBuf> {
        self.with_matching(names, |procs| procs.filter_map(|p| p.exe().map(PathBuf::from)).collect())
            .unwrap_or_default()
    }
}

impl ProcessMonitor for SysinfoProcessMonitor {
    fn is_any_running(&self, names: &[&str]) -> bool {
        !names.is_empty() && self.with_matching(names, |procs| procs.next().is_some()).unwrap_or(false)
    }
}
