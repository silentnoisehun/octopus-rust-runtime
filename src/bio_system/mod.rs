//! First-class Bio-Binaries subsystem.
//!
//! The bundled `bio-binaries` crate stays an independent Rust system with its
//! own manifest, lockfile, source tree and executable boundary. Octopus only
//! catalogs, authorizes and launches those exact executables; it does not copy
//! their algorithms into the Octopus process.

pub mod external;

use crate::ExecutionOutcome;

pub fn public_names() -> impl Iterator<Item = &'static str> {
    external::catalog().iter().map(|binary| binary.name)
}

pub fn contains(name: &str) -> bool {
    external::find(name).is_some()
}

pub fn execute(name: &str, input: &str, allow_mutation: bool) -> Option<ExecutionOutcome> {
    contains(name).then(|| external::execute(name, input, allow_mutation))
}

pub fn status() -> ExecutionOutcome {
    ExecutionOutcome::completed(format!(
        "BIO SUBSYSTEM\nlayout: separate-bundled-crate\nmanifest: {}\n{}",
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("bio-binaries")
            .join("Cargo.toml")
            .display(),
        external::render_status()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_names_are_the_exact_external_catalog() {
        let names: Vec<_> = public_names().collect();
        assert_eq!(names.len(), external::BIO_BINARY_COUNT);
        assert!(names.contains(&"omega-master"));
        assert!(names.contains(&"homeostasis"));
    }

    #[test]
    fn subsystem_status_preserves_the_separate_crate_boundary() {
        let status = status();
        assert!(!status.is_failed());
        assert!(status.output.contains("layout: separate-bundled-crate"));
        assert!(status.output.contains("availability:"));
    }
}
