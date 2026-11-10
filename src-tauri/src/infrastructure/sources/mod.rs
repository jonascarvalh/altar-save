//! Fontes de saves (emuladores). Para adicionar um emulador, implemente `SaveSource`
//! num novo módulo e registre-o em `DefaultSourceProvider`.

mod ini;
pub mod pcsx2;
mod ps2_icon_sys;

use std::sync::Arc;

use crate::domain::{
    models::AppSettings,
    ports::{RegisteredSource, SourceProvider},
};

pub struct DefaultSourceProvider;

impl SourceProvider for DefaultSourceProvider {
    fn sources(&self, settings: &AppSettings) -> Vec<RegisteredSource> {
        vec![RegisteredSource {
            source: Arc::new(pcsx2::Pcsx2Source::new(&settings.pcsx2)),
            enabled: settings.pcsx2.enabled,
            auto_sync: settings.pcsx2.auto_sync,
        }]
    }
}
