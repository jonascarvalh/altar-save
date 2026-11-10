use serde::{Serialize, Serializer, ser::SerializeStruct};

/// Erro único da aplicação. Todas as camadas convertem suas falhas para ele,
/// e ele é serializado como `{ kind, message }` para o frontend.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),

    #[error("A nuvem ainda não foi configurada.")]
    CloudNotConfigured,

    #[error("{0}")]
    Storage(String),

    #[error("{0}")]
    Integrity(String),

    #[error("{0}")]
    NotFound(String),

    #[error("Erro de arquivo: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dados inválidos: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Internal(String),
}

/// O arquivo está aberto com exclusividade por outro processo (ex.: o emulador durante o jogo).
pub fn is_file_in_use(err: &std::io::Error) -> bool {
    // Windows: ERROR_SHARING_VIOLATION (32) e ERROR_LOCK_VIOLATION (33).
    matches!(err.raw_os_error(), Some(32 | 33))
}

impl AppError {
    pub fn is_file_in_use(&self) -> bool {
        matches!(self, Self::Io(err) if is_file_in_use(err))
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation",
            Self::CloudNotConfigured => "cloudNotConfigured",
            Self::Storage(_) => "storage",
            Self::Integrity(_) => "integrity",
            Self::NotFound(_) => "notFound",
            Self::Io(_) => "io",
            Self::Serialization(_) => "serialization",
            Self::Internal(_) => "internal",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
