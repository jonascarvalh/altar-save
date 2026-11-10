use sha2::{Digest, Sha256};

use super::models::FileEntry;

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Hash de conteúdo de uma unidade de sync.
///
/// Calculado sobre a lista ordenada de `caminho + sha256` dos arquivos, ignorando os
/// arquivos voláteis (ex.: `_pcsx2_index`). Esses arquivos mudam sem que o save mude
/// e não devem disparar sincronização, mas continuam sendo transferidos junto com a unidade.
pub fn content_hash(files: &[FileEntry]) -> String {
    let mut relevant: Vec<&FileEntry> = files.iter().filter(|f| !f.volatile).collect();
    relevant.sort_by(|a, b| a.path.cmp(&b.path));

    let mut hasher = Sha256::new();
    for file in relevant {
        hasher.update(file.path.as_bytes());
        hasher.update([0u8]);
        hasher.update(file.sha256.as_bytes());
        hasher.update([b'\n']);
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, sha: &str, volatile: bool) -> FileEntry {
        FileEntry { path: path.into(), sha256: sha.into(), size: 1, volatile }
    }

    #[test]
    fn hash_is_order_independent() {
        let a = vec![entry("a", "1", false), entry("b", "2", false)];
        let b = vec![entry("b", "2", false), entry("a", "1", false)];
        assert_eq!(content_hash(&a), content_hash(&b));
    }

    #[test]
    fn volatile_files_do_not_change_hash() {
        let a = vec![entry("save.dat", "1", false), entry("_pcsx2_index", "x", true)];
        let b = vec![entry("save.dat", "1", false), entry("_pcsx2_index", "y", true)];
        let c = vec![entry("save.dat", "1", false)];
        assert_eq!(content_hash(&a), content_hash(&b));
        assert_eq!(content_hash(&a), content_hash(&c));
    }

    #[test]
    fn content_change_changes_hash() {
        let a = vec![entry("save.dat", "1", false)];
        let b = vec![entry("save.dat", "2", false)];
        assert_ne!(content_hash(&a), content_hash(&b));
    }
}
