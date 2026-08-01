use std::fs;
use std::path::{Path, PathBuf};

pub struct BoundaryContract {
    path: PathBuf,
    find: String,
    replacement: String,
}

impl BoundaryContract {
    pub fn from_prompt(prompt: &str) -> Result<Self, String> {
        let mut parts = prompt.splitn(3, '|').map(str::trim);
        let path = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or("pipeline-architect requires: rust_file|exact_boundary|replacement")?;
        let find = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or("pipeline-architect requires a non-empty exact boundary")?;
        let replacement = parts
            .next()
            .ok_or("pipeline-architect requires replacement text")?;

        let path = fs::canonicalize(path)
            .map_err(|error| format!("pipeline boundary path is invalid: {error}"))?;
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            return Err("pipeline boundary must target one .rs file".to_string());
        }
        let content = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read bounded Rust file: {error}"))?;
        let matches = content.matches(find).count();
        if matches != 1 {
            return Err(format!(
                "pipeline boundary must match exactly once; observed {matches}"
            ));
        }

        Ok(Self {
            path,
            find: find.to_string(),
            replacement: replacement.to_string(),
        })
    }

    pub fn describe(&self) -> String {
        format!(
            "[pipeline-architect] BOUNDARY LOCKED\nfile: {}\nmatch-count: 1\nownership: exact replacement only\nstop: after one transactional commit",
            self.path.display()
        )
    }

    pub fn apply(&self) -> Result<String, String> {
        let content = fs::read_to_string(&self.path)
            .map_err(|error| format!("rust-surgeon cannot read target: {error}"))?;
        if content.matches(&self.find).count() != 1 {
            return Err("rust-surgeon boundary changed after architecture lock".to_string());
        }

        let backup = backup_path(&self.path);
        if backup.exists() {
            fs::remove_file(&backup)
                .map_err(|error| format!("rust-surgeon cannot refresh backup: {error}"))?;
        }
        let updated = content.replacen(&self.find, &self.replacement, 1);
        let temporary = self.path.with_extension("rs.octopus.tmp");
        fs::write(&temporary, updated)
            .map_err(|error| format!("rust-surgeon cannot write temporary file: {error}"))?;
        fs::rename(&self.path, &backup)
            .map_err(|error| format!("rust-surgeon cannot lock original as backup: {error}"))?;
        if let Err(error) = fs::rename(&temporary, &self.path) {
            let _ = fs::rename(&backup, &self.path);
            return Err(format!(
                "rust-surgeon cannot commit transactional replacement: {error}"
            ));
        }

        Ok(format!(
            "[rust-surgeon] CUT COMPLETE\nfile: {}\nmodified-boundaries: 1\ntransactional-backup: {}",
            self.path.display(),
            backup.display()
        ))
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("source.rs");
    path.with_file_name(format!("{name}.octopus.bak"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn architect_bounds_and_surgeon_cuts_once() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("octopus-contract-{id}"));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("main.rs");
        fs::write(&source, "fn value() -> u8 { 1 }\n").unwrap();
        let prompt = format!(
            "{}|fn value() -> u8 {{ 1 }}|fn value() -> u8 {{ 2 }}",
            source.display()
        );
        let contract = BoundaryContract::from_prompt(&prompt).unwrap();
        contract.apply().unwrap();
        assert_eq!(
            fs::read_to_string(&source).unwrap(),
            "fn value() -> u8 { 2 }\n"
        );
        assert!(backup_path(&source).is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn architect_rejects_ambiguous_boundary() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("octopus-contract-{id}"));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("main.rs");
        fs::write(&source, "x();\nx();\n").unwrap();
        let prompt = format!("{}|x();|y();", source.display());
        assert!(BoundaryContract::from_prompt(&prompt).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
