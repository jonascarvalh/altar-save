//! Leitor mínimo de INI (seções e `chave = valor`), sem diferenciar maiúsculas.

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Ini {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Ini {
    pub fn parse(text: &str) -> Self {
        let mut ini = Ini::default();
        let mut section = String::new();

        for raw in text.trim_start_matches('\u{feff}').lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.trim().to_lowercase();
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                ini.sections
                    .entry(section.clone())
                    .or_default()
                    .insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }
        ini
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(&section.to_lowercase())?
            .get(&key.to_lowercase())
            .map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_case_insensitively() {
        let ini = Ini::parse("\u{feff}; comment\n[Folders]\nMemoryCards = memcards\n[MemoryCards]\nSlot1_Filename=Mcd001.ps2\n");
        assert_eq!(ini.get("folders", "memorycards"), Some("memcards"));
        assert_eq!(ini.get("MemoryCards", "Slot1_Filename"), Some("Mcd001.ps2"));
        assert_eq!(ini.get("MemoryCards", "missing"), None);
    }
}
