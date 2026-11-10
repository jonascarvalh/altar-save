use std::{future::Future, time::Instant};

use chrono::Utc;
use serde::Serialize;

use crate::{
    domain::{models::CloudConfig, ports::CloudStorageFactory},
    error::{AppError, AppResult},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStep {
    pub name: String,
    pub ok: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionReport {
    pub ok: bool,
    pub endpoint: String,
    pub bucket: String,
    pub steps: Vec<ConnectionStep>,
}

/// Valida a configuração de ponta a ponta: acesso ao bucket, escrita, leitura e remoção.
pub async fn run(factory: &dyn CloudStorageFactory, config: &CloudConfig) -> AppResult<ConnectionReport> {
    config.validate()?;
    let storage = factory.create(config)?;
    let key = config.key(".save-nuvem-test");
    let payload = format!("save-nuvem connection test {}", Utc::now().to_rfc3339()).into_bytes();

    let mut steps = Vec::new();
    let ok = step(&mut steps, "Acessar bucket", storage.check_bucket()).await
        && step(&mut steps, "Gravar arquivo de teste", storage.put_object(&key, payload.clone(), &[])).await
        && step(&mut steps, "Ler arquivo de teste", async {
            match storage.get_object(&key).await? {
                Some(body) if body == payload => Ok(()),
                Some(_) => Err(AppError::Integrity("o conteúdo lido difere do gravado.".into())),
                None => Err(AppError::Storage("o arquivo gravado não foi encontrado.".into())),
            }
        })
        .await
        && step(&mut steps, "Remover arquivo de teste", storage.delete_object(&key)).await;

    Ok(ConnectionReport {
        ok,
        endpoint: config.endpoint_url().unwrap_or_else(|| format!("s3.{}.amazonaws.com", config.signing_region())),
        bucket: config.bucket.clone(),
        steps,
    })
}

async fn step(steps: &mut Vec<ConnectionStep>, name: &str, op: impl Future<Output = AppResult<()>>) -> bool {
    let started = Instant::now();
    let result = op.await;
    let ok = result.is_ok();
    steps.push(ConnectionStep {
        name: name.into(),
        ok,
        duration_ms: started.elapsed().as_millis() as u64,
        error: result.err().map(|e| e.to_string()),
    });
    ok
}
