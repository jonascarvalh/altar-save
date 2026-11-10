//! Backups locais feitos antes de qualquer sobrescrita vinda da nuvem.

use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;

use super::fs_store::safe_relative_path;
use crate::{
    domain::{
        models::{SyncUnit, UnitKind},
        ports::BackupStore,
    },
    error::AppResult,
};

pub struct FsBackupStore {
    root: PathBuf,
}

impl FsBackupStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl BackupStore for FsBackupStore {
    fn backup(&self, unit: &SyncUnit, keep: usize) -> AppResult<Option<PathBuf>> {
        if !unit.exists_locally || !unit.local_path.exists() {
            return Ok(None);
        }

        let unit_dir = self.root.join(safe_relative_path(&unit.id)?);
        let stamp = Local::now().format("%Y-%m-%d_%H-%M-%S%.3f").to_string();
        let dest = unit_dir.join(stamp);
        let name = unit.local_path.file_name().unwrap_or_default();

        fs::create_dir_all(&dest)?;
        match unit.kind {
            UnitKind::File => {
                fs::copy(&unit.local_path, dest.join(name))?;
            }
            UnitKind::Directory => copy_dir(&unit.local_path, &dest.join(name))?,
        }

        rotate(&unit_dir, keep.max(1))?;
        Ok(Some(dest))
    }
}

fn copy_dir(from: &Path, to: &Path) -> AppResult<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Mantém apenas os `keep` backups mais recentes (nomes são timestamps ordenáveis).
fn rotate(unit_dir: &Path, keep: usize) -> AppResult<()> {
    let mut snapshots: Vec<PathBuf> = fs::read_dir(unit_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    snapshots.sort();
    let excess = snapshots.len().saturating_sub(keep);
    for old in snapshots.into_iter().take(excess) {
        let _ = fs::remove_dir_all(old);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_latest_backups() {
        let tmp = tempfile::tempdir().unwrap();
        let card = tmp.path().join("Mcd001.ps2");
        fs::write(&card, b"data").unwrap();
        let unit = SyncUnit {
            id: "pcsx2/memcards/Mcd001".into(),
            source_id: "pcsx2".into(),
            group: "Mcd001".into(),
            title: "Mcd001".into(),
            subtitle: None,
            tags: vec![],
            kind: UnitKind::File,
            local_path: card,
            files: vec![],
            exists_locally: true,
            modified_at: None,
            busy: false,
        };
        let store = FsBackupStore::new(tmp.path().join("backups"));
        for _ in 0..4 {
            store.backup(&unit, 2).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let dir = tmp.path().join("backups").join("pcsx2").join("memcards").join("Mcd001");
        assert_eq!(fs::read_dir(dir).unwrap().count(), 2);
    }
}
