//! Núcleo do negócio: modelos, contratos (ports) e regras puras.
//! Não depende de Tauri, AWS nem de sistema de arquivos.

pub mod hashing;
pub mod models;
pub mod ports;
pub mod sync_policy;
