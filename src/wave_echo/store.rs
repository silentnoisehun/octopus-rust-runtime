//! .wv3 binary store with BLAKE3 checksums and ::: field separators

use super::template::WaveTemplate;
use blake3::Hasher;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub const WV3_MAGIC: &[u8; 8] = b"WV3_STOR";

pub struct WaveStore;

impl WaveStore {
    /// Load .wv3 binary store
    pub fn load_wv3(path: impl AsRef<Path>) -> Result<Vec<WaveTemplate>, String> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut file = File::open(path)
            .map_err(|e| format!("cannot open .wv3 file {}: {e}", path.display()))?;

        let mut magic = [0u8; 8];
        if file.read_exact(&mut magic).is_err() || &magic != WV3_MAGIC {
            return Err(format!("invalid .wv3 store magic in {}", path.display()));
        }

        let mut count_bytes = [0u8; 4];
        file.read_exact(&mut count_bytes)
            .map_err(|e| format!("cannot read count: {e}"))?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        let mut stored_hash = [0u8; 32];
        file.read_exact(&mut stored_hash)
            .map_err(|e| format!("cannot read hash: {e}"))?;

        let mut payload = Vec::new();
        file.read_to_end(&mut payload)
            .map_err(|e| format!("cannot read payload: {e}"))?;

        let mut hasher = Hasher::new();
        hasher.update(&payload);
        let computed_hash = hasher.finalize();

        if computed_hash.as_bytes() != &stored_hash {
            return Err(format!("BLAKE3 checksum mismatch in {}", path.display()));
        }

        let payload_str = String::from_utf8(payload)
            .map_err(|e| format!("invalid UTF-8 in .wv3 payload: {e}"))?;

        let mut templates = Vec::with_capacity(count);
        for (index, line) in payload_str.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            if let Some(template) = Self::parse_line(index + 1, line) {
                templates.push(template);
            }
        }

        Ok(templates)
    }

    /// Save templates to .wv3 binary store
    pub fn save_wv3(path: impl AsRef<Path>, templates: &[WaveTemplate]) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut payload = String::new();
        for t in templates {
            let keywords = t.keywords.join(",");
            let tech = t.tech_stack.join(",");
            let line = format!(
                "{} ::: {} ::: {} ::: {} ::: {:.4}\n",
                t.label, keywords, t.code, tech, t.amplitude
            );
            payload.push_str(&line);
        }

        let payload_bytes = payload.as_bytes();
        let mut hasher = Hasher::new();
        hasher.update(payload_bytes);
        let hash = hasher.finalize();

        let mut file = File::create(path)
            .map_err(|e| format!("cannot create .wv3 file {}: {e}", path.display()))?;

        file.write_all(WV3_MAGIC).map_err(|e| format!("{e}"))?;
        file.write_all(&(templates.len() as u32).to_le_bytes())
            .map_err(|e| format!("{e}"))?;
        file.write_all(hash.as_bytes())
            .map_err(|e| format!("{e}"))?;
        file.write_all(payload_bytes).map_err(|e| format!("{e}"))?;

        Ok(())
    }

    /// Add native text template file (append-only) to .wv3 store
    pub fn add_native_file(
        wv3_path: impl AsRef<Path>,
        txt_path: impl AsRef<Path>,
    ) -> Result<usize, String> {
        let mut templates = Self::load_wv3(&wv3_path).unwrap_or_default();
        let txt_content = fs::read_to_string(txt_path.as_ref())
            .map_err(|e| format!("cannot read native text file: {e}"))?;

        let mut added = 0;
        let start_id = templates.len() + 1;
        for line in txt_content.lines() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            if let Some(template) = Self::parse_line(start_id + added, line) {
                templates.push(template);
                added += 1;
            }
        }

        Self::save_wv3(wv3_path, &templates)?;
        Ok(added)
    }

    fn parse_line(id: usize, line: &str) -> Option<WaveTemplate> {
        let parts: Vec<&str> = line.split(":::").collect();
        if parts.len() < 3 {
            return None;
        }

        let label = parts[0].trim().to_string();
        let keywords: Vec<String> = parts[1]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let code = parts[2].trim().to_string();

        let tech_stack: Vec<String> = if parts.len() >= 4 {
            parts[3]
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        let amplitude: f64 = if parts.len() >= 5 {
            parts[4].trim().parse().unwrap_or(0.80)
        } else {
            0.80
        };

        Some(WaveTemplate::new(
            id, label, keywords, code, tech_stack, amplitude,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_wv3_save_and_load_roundtrip() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let templates = vec![WaveTemplate::new(
            1,
            "cli-vaz",
            vec!["rust".to_string(), "cli".to_string()],
            "WRITE||src/main.rs@@@fn main() {}",
            vec!["rust".to_string(), "clap".to_string()],
            0.85,
        )];

        WaveStore::save_wv3(path, &templates).unwrap();
        let loaded = WaveStore::load_wv3(path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].label, "cli-vaz");
        assert_eq!(loaded[0].keywords, vec!["rust", "cli"]);
        assert!((loaded[0].amplitude - 0.85).abs() < 1e-4);
    }
}
