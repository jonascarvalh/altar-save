//! Adapter único para Cloudflare R2, Amazon S3 e qualquer serviço compatível com S3.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use aws_sdk_s3::{
    Client,
    config::{
        BehaviorVersion, Credentials, Region, RequestChecksumCalculation, ResponseChecksumValidation,
        http::HttpResponse, retry::RetryConfig, timeout::TimeoutConfig,
    },
    error::{DisplayErrorContext, ProvideErrorMetadata, SdkError},
    primitives::ByteStream,
};

use crate::{
    domain::{
        models::CloudConfig,
        ports::{CloudStorage, CloudStorageFactory},
    },
    error::{AppError, AppResult},
};

pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub fn new(config: &CloudConfig) -> Self {
        let credentials = Credentials::new(
            config.access_key_id.trim(),
            config.secret_access_key.trim(),
            None,
            None,
            "save-nuvem",
        );

        let mut builder = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.signing_region()))
            .credentials_provider(credentials)
            // R2 e outros serviços compatíveis não aceitam todos os checksums novos do SDK.
            .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
            .response_checksum_validation(ResponseChecksumValidation::WhenRequired)
            .retry_config(RetryConfig::standard().with_max_attempts(3))
            .timeout_config(
                TimeoutConfig::builder()
                    .connect_timeout(Duration::from_secs(10))
                    .operation_attempt_timeout(Duration::from_secs(120))
                    .build(),
            );

        if let Some(endpoint) = config.endpoint_url() {
            builder = builder.endpoint_url(endpoint).force_path_style(true);
        }

        Self { client: Client::from_conf(builder.build()), bucket: config.bucket.trim().to_string() }
    }
}

#[async_trait]
impl CloudStorage for S3Storage {
    async fn check_bucket(&self) -> AppResult<()> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|e| storage_error("Bucket", e))?;
        Ok(())
    }

    async fn put_object(&self, key: &str, body: Vec<u8>, metadata: &[(&str, String)]) -> AppResult<()> {
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(body));
        for (name, value) in metadata {
            request = request.metadata(*name, value.clone());
        }
        request.send().await.map_err(|e| storage_error("Envio", e))?;
        Ok(())
    }

    async fn get_object(&self, key: &str) -> AppResult<Option<Vec<u8>>> {
        match self.client.get_object().bucket(&self.bucket).key(key).send().await {
            Ok(output) => {
                let data = output
                    .body
                    .collect()
                    .await
                    .map_err(|e| AppError::Storage(format!("Download: {e}")))?;
                Ok(Some(data.into_bytes().to_vec()))
            }
            Err(SdkError::ServiceError(ctx)) if ctx.err().is_no_such_key() || ctx.raw().status().as_u16() == 404 => {
                Ok(None)
            }
            Err(err) => Err(storage_error("Download", err)),
        }
    }

    async fn delete_object(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| storage_error("Remoção", e))?;
        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> AppResult<Vec<String>> {
        let mut keys = Vec::new();
        let mut token: Option<String> = None;
        loop {
            let output = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix)
                .set_continuation_token(token.take())
                .send()
                .await
                .map_err(|e| storage_error("Listagem", e))?;

            keys.extend(output.contents().iter().filter_map(|o| o.key().map(String::from)));

            match (output.is_truncated(), output.next_continuation_token()) {
                (Some(true), Some(next)) => token = Some(next.to_string()),
                _ => break,
            }
        }
        Ok(keys)
    }
}

pub struct S3StorageFactory;

impl CloudStorageFactory for S3StorageFactory {
    fn create(&self, config: &CloudConfig) -> AppResult<Arc<dyn CloudStorage>> {
        Ok(Arc::new(S3Storage::new(config)))
    }
}

/// Traduz erros do SDK em mensagens úteis para o usuário.
fn storage_error<E>(operation: &str, err: SdkError<E, HttpResponse>) -> AppError
where
    E: ProvideErrorMetadata + std::error::Error + Send + Sync + 'static,
{
    let detail = match &err {
        SdkError::DispatchFailure(_) => {
            "não foi possível conectar ao endpoint. Verifique o Account ID/endpoint e sua internet.".to_string()
        }
        SdkError::TimeoutError(_) => "o tempo de conexão esgotou.".to_string(),
        SdkError::ServiceError(ctx) => {
            let status = ctx.raw().status().as_u16();
            let code = ctx.err().code().unwrap_or_default();
            match (status, code) {
                (_, "InvalidAccessKeyId") => "Access Key ID inválida.".into(),
                (_, "SignatureDoesNotMatch") => "Secret Access Key inválida.".into(),
                (_, "NoSuchBucket") | (404, _) => "bucket não encontrado.".into(),
                (_, "AccessDenied") | (401 | 403, _) => {
                    "acesso negado. Verifique as chaves e as permissões do token.".into()
                }
                (301, _) | (_, "PermanentRedirect" | "AuthorizationHeaderMalformed") => {
                    "região incorreta para este bucket.".into()
                }
                _ => {
                    let message = ctx.err().message().unwrap_or_default();
                    let code = if code.is_empty() { "erro" } else { code };
                    format!("{code} (HTTP {status}) {message}").trim().to_string()
                }
            }
        }
        _ => DisplayErrorContext(&err).to_string(),
    };
    AppError::Storage(format!("{operation}: {detail}"))
}
