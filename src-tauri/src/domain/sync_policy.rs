//! Decisão de sincronização em 3 vias (local × nuvem × base).
//! A base é o hash da última versão sincronizada com sucesso nesta máquina.

use super::models::SyncAction;

/// O que existe na nuvem para uma unidade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteState<'a> {
    Missing,
    Live(&'a str),
    /// Lápide: o save foi excluído de propósito em alguma máquina.
    Deleted,
}

pub fn decide(local: Option<&str>, remote: RemoteState<'_>, base: Option<&str>) -> SyncAction {
    match (local, remote) {
        (None, RemoteState::Missing | RemoteState::Deleted) => SyncAction::Nothing,
        (Some(_), RemoteState::Missing) => SyncAction::Upload,
        (None, RemoteState::Live(_)) => SyncAction::Download,
        // Excluído na nuvem: só apaga aqui se esta máquina não mexeu no save desde o último sync.
        (Some(l), RemoteState::Deleted) => match base {
            Some(b) if l == b => SyncAction::DeleteLocal,
            _ => SyncAction::Conflict,
        },
        (Some(l), RemoteState::Live(r)) if l == r => SyncAction::InSync,
        (Some(l), RemoteState::Live(r)) => match base {
            Some(b) if r == b => SyncAction::Upload,
            Some(b) if l == b => SyncAction::Download,
            // Os dois lados mudaram, ou nunca houve sync nesta máquina e eles diferem.
            _ => SyncAction::Conflict,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use RemoteState::*;
    use SyncAction::*;

    #[test]
    fn nothing_anywhere() {
        assert_eq!(decide(None, Missing, None), Nothing);
        assert_eq!(decide(None, Missing, Some("a")), Nothing);
    }

    #[test]
    fn only_local_uploads() {
        assert_eq!(decide(Some("a"), Missing, None), Upload);
        assert_eq!(decide(Some("a"), Missing, Some("a")), Upload);
    }

    #[test]
    fn only_remote_downloads() {
        assert_eq!(decide(None, Live("a"), None), Download);
        assert_eq!(decide(None, Live("a"), Some("a")), Download);
    }

    #[test]
    fn equal_is_in_sync() {
        assert_eq!(decide(Some("a"), Live("a"), None), InSync);
        assert_eq!(decide(Some("a"), Live("a"), Some("z")), InSync);
    }

    #[test]
    fn local_changed_uploads() {
        assert_eq!(decide(Some("new"), Live("base"), Some("base")), Upload);
    }

    #[test]
    fn remote_changed_downloads() {
        assert_eq!(decide(Some("base"), Live("new"), Some("base")), Download);
    }

    #[test]
    fn both_changed_conflicts() {
        assert_eq!(decide(Some("l"), Live("r"), Some("base")), Conflict);
    }

    #[test]
    fn differing_without_base_conflicts() {
        assert_eq!(decide(Some("l"), Live("r"), None), Conflict);
    }

    #[test]
    fn tombstone_deletes_unchanged_local_copy() {
        assert_eq!(decide(Some("base"), Deleted, Some("base")), DeleteLocal);
    }

    #[test]
    fn tombstone_with_local_changes_conflicts() {
        assert_eq!(decide(Some("changed"), Deleted, Some("base")), Conflict);
        assert_eq!(decide(Some("never-synced"), Deleted, None), Conflict);
    }

    #[test]
    fn tombstone_without_local_copy_is_nothing() {
        assert_eq!(decide(None, Deleted, Some("base")), Nothing);
    }
}
