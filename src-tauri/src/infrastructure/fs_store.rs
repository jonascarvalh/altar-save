//! Leitura e substituição atômica dos arquivos locais de uma unidade.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    domain::{
        models::{FileEntry, SyncUnit, UnitKind},
        ports::LocalUnitStore,
    },
    error::{AppError, AppResult},
};

/// Marcador usado em arquivos/pastas temporários; o watcher ignora esses caminhos.
pub const TEMP_MARKER: &str = ".save-nuvem-";

pub struct FsUnitStore;

impl LocalUnitStore for FsUnitStore {
    fn read_file(&self, unit: &SyncUnit, relative_path: &str) -> AppResult<Vec<u8>> {
        let path = match unit.kind {
            UnitKind::File => unit.local_path.clone(),
            UnitKind::Directory => unit.local_path.join(safe_relative_path(relative_path)?),
        };
        Ok(fs::read(path)?)
    }

    fn replace_unit(&self, unit: &SyncUnit, files: Vec<(FileEntry, Vec<u8>)>) -> AppResult<()> {
        match unit.kind {
            UnitKind::File => replace_file(&unit.local_path, files),
            UnitKind::Directory => replace_dir(&unit.local_path, &files),
        }
    }

    fn remove_unit(&self, unit: &SyncUnit) -> AppResult<()> {
        let path = &unit.local_path;
        match unit.kind {
            UnitKind::File if path.is_file() => fs::remove_file(path)?,
            UnitKind::Directory if path.is_dir() => {
                // Renomeia antes de apagar: se algo travar no meio, nada fica "meio apagado" no lugar do save.
                let trash = sibling(path, "del");
                remove_dir_if_exists(&trash)?;
                fs::rename(path, &trash)?;
                fs::remove_dir_all(&trash)?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Converte um caminho relativo vindo da nuvem em `PathBuf`, rejeitando travessia de diretório.
pub fn safe_relative_path(relative: &str) -> AppResult<PathBuf> {
    let mut path = PathBuf::new();
    for part in relative.split('/') {
        if part.is_empty() || part == "." || part == ".." || part.contains(['\\', ':']) {
            return Err(AppError::Integrity(format!("Caminho inválido: {relative}")));
        }
        path.push(part);
    }
    if path.as_os_str().is_empty() {
        return Err(AppError::Integrity("Caminho vazio.".into()));
    }
    Ok(path)
}

fn sibling(target: &Path, suffix: &str) -> PathBuf {
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    target.with_file_name(format!(".{name}{TEMP_MARKER}{suffix}"))
}

fn replace_file(target: &Path, files: Vec<(FileEntry, Vec<u8>)>) -> AppResult<()> {
    let mut iter = files.into_iter();
    let (Some((_, bytes)), None) = (iter.next(), iter.next()) else {
        return Err(AppError::Integrity("Um save em arquivo deve conter exatamente um arquivo.".into()));
    };
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = sibling(target, "tmp");
    fs::write(&tmp, &bytes)?;
    // No Windows, `rename` substitui o destino (MoveFileEx + REPLACE_EXISTING).
    if let Err(err) = fs::rename(&tmp, target) {
        let _ = fs::remove_file(&tmp);
        return Err(err.into());
    }
    Ok(())
}

fn replace_dir(target: &Path, files: &[(FileEntry, Vec<u8>)]) -> AppResult<()> {
    let parent = target
        .parent()
        .ok_or_else(|| AppError::Internal(format!("Pasta sem pai: {}", target.display())))?;
    fs::create_dir_all(parent)?;

    let tmp = sibling(target, "tmp");
    let old = sibling(target, "old");
    remove_dir_if_exists(&tmp)?;
    remove_dir_if_exists(&old)?;

    // 1. Monta a nova versão inteira numa pasta temporária.
    let write_all = || -> AppResult<()> {
        fs::create_dir_all(&tmp)?;
        for (entry, bytes) in files {
            let path = tmp.join(safe_relative_path(&entry.path)?);
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::write(path, bytes)?;
        }
        Ok(())
    };
    if let Err(err) = write_all() {
        let _ = fs::remove_dir_all(&tmp);
        return Err(err);
    }

    // 2. Troca as pastas com renames (rápido e reversível).
    let had_old = target.exists();
    if had_old && let Err(err) = fs::rename(target, &old) {
        let _ = fs::remove_dir_all(&tmp);
        return Err(err.into());
    }
    if let Err(err) = fs::rename(&tmp, target) {
        if had_old {
            let _ = fs::rename(&old, target);
        }
        let _ = fs::remove_dir_all(&tmp);
        return Err(err.into());
    }
    if had_old {
        let _ = fs::remove_dir_all(&old);
    }
    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> AppResult<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hashing::sha256_hex;

    fn unit(kind: UnitKind, path: PathBuf) -> SyncUnit {
        SyncUnit {
            id: "test/unit".into(),
            source_id: "test".into(),
            group: "g".into(),
            title: "t".into(),
            subtitle: None,
            tags: vec![],
            kind,
            local_path: path,
            files: vec![],
            exists_locally: true,
            modified_at: None,
            busy: false,
        }
    }

    fn entry(path: &str, bytes: &[u8]) -> (FileEntry, Vec<u8>) {
        let e = FileEntry { path: path.into(), sha256: sha256_hex(bytes), size: bytes.len() as u64, volatile: false };
        (e, bytes.to_vec())
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(safe_relative_path("../evil").is_err());
        assert!(safe_relative_path("a/../../b").is_err());
        assert!(safe_relative_path("C:/x").is_err());
        assert!(safe_relative_path("a\\b").is_err());
        assert!(safe_relative_path("ok/file.bin").is_ok());
    }

    #[test]
    fn replaces_directory_atomically() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("BASLUS-20778");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("old.bin"), b"old").unwrap();

        let u = unit(UnitKind::Directory, dir.clone());
        FsUnitStore.replace_unit(&u, vec![entry("new.bin", b"new"), entry("sub/x.bin", b"x")]).unwrap();

        assert!(!dir.join("old.bin").exists());
        assert_eq!(fs::read(dir.join("new.bin")).unwrap(), b"new");
        assert_eq!(fs::read(dir.join("sub").join("x.bin")).unwrap(), b"x");
        // Nenhum resíduo temporário.
        let leftovers: Vec<_> = fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(TEMP_MARKER))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn replaces_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("Mcd001.ps2");
        fs::write(&file, b"old").unwrap();
        let u = unit(UnitKind::File, file.clone());
        FsUnitStore.replace_unit(&u, vec![entry("Mcd001.ps2", b"new")]).unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"new");
    }
}
