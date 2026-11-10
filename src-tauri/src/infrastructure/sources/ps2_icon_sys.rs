//! Leitura do título de um save de PS2 a partir do `icon.sys` (texto Shift-JIS, offset 0xC0).

use std::{fs, path::Path};

const MAGIC: &[u8; 4] = b"PS2D";
const TITLE_OFFSET: usize = 0xC0;
const TITLE_LEN: usize = 68;

pub fn read_title(save_dir: &Path) -> Option<String> {
    let bytes = fs::read(save_dir.join("icon.sys")).ok()?;
    parse_title(&bytes)
}

pub fn parse_title(bytes: &[u8]) -> Option<String> {
    if bytes.len() < TITLE_OFFSET + TITLE_LEN || &bytes[..4] != MAGIC {
        return None;
    }
    let raw = &bytes[TITLE_OFFSET..TITLE_OFFSET + TITLE_LEN];
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let raw = &raw[..end];

    // Offset (em bytes) onde começa a segunda linha do título.
    let line_break = u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
    let text = if line_break > 0 && line_break < raw.len() {
        format!("{} {}", decode(&raw[..line_break]), decode(&raw[line_break..]))
    } else {
        decode(raw)
    };

    let title = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!title.is_empty()).then_some(title)
}

/// Decodifica Shift-JIS e converte caracteres de largura total em ASCII.
fn decode(bytes: &[u8]) -> String {
    let (text, _) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(bytes);
    text.chars()
        .map(|c| match c {
            '\u{3000}' => ' ',
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon_sys(title_sjis: &[u8], line_break: u16) -> Vec<u8> {
        let mut bytes = vec![0u8; 0x3C4];
        bytes[..4].copy_from_slice(MAGIC);
        bytes[6..8].copy_from_slice(&line_break.to_le_bytes());
        bytes[TITLE_OFFSET..TITLE_OFFSET + title_sjis.len()].copy_from_slice(title_sjis);
        bytes
    }

    #[test]
    fn decodes_fullwidth_title_with_line_break() {
        // "ＧＯＤ" + "ＷＡＲ" em Shift-JIS de largura total.
        let sjis = [0x82, 0x66, 0x82, 0x6E, 0x82, 0x63, 0x82, 0x76, 0x82, 0x60, 0x82, 0x71];
        assert_eq!(parse_title(&icon_sys(&sjis, 6)).as_deref(), Some("GOD WAR"));
    }

    #[test]
    fn rejects_invalid_files() {
        assert_eq!(parse_title(b"nope"), None);
    }
}
