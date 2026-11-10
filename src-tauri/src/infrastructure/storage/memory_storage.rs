//! Storage em memória para testes. Simula um bucket compartilhado entre "máquinas".

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::{
    domain::{
        models::CloudConfig,
        ports::{CloudStorage, CloudStorageFactory},
    },
    error::AppResult,
};

#[derive(Clone, Default)]
pub struct MemoryStorage {
    pub objects: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
}

impl MemoryStorage {
    pub fn keys(&self) -> Vec<String> {
        self.objects.lock().unwrap().keys().cloned().collect()
    }
}

#[async_trait]
impl CloudStorage for MemoryStorage {
    async fn check_bucket(&self) -> AppResult<()> {
        Ok(())
    }

    async fn put_object(&self, key: &str, body: Vec<u8>, _metadata: &[(&str, String)]) -> AppResult<()> {
        self.objects.lock().unwrap().insert(key.into(), body);
        Ok(())
    }

    async fn get_object(&self, key: &str) -> AppResult<Option<Vec<u8>>> {
        Ok(self.objects.lock().unwrap().get(key).cloned())
    }

    async fn delete_object(&self, key: &str) -> AppResult<()> {
        self.objects.lock().unwrap().remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> AppResult<Vec<String>> {
        Ok(self.objects.lock().unwrap().keys().filter(|k| k.starts_with(prefix)).cloned().collect())
    }
}

impl CloudStorageFactory for MemoryStorage {
    fn create(&self, _config: &CloudConfig) -> AppResult<Arc<dyn CloudStorage>> {
        Ok(Arc::new(self.clone()))
    }
}
