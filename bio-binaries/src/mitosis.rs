/// Mitosis — Self-replication & Ribosome Code Generation
///
/// A binary can:
/// 1. Read its own genome (executable bytes)
/// 2. Send it over the network to another host
/// 3. The receiver saves and spawns the new instance
///
/// The Ribosome can generate new binaries from templates.
use crate::auth::QueenKey;
use crate::bio_protocol::{self, flags, BioMessage, BioOp};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;

pub const GENOME_CHUNK_SIZE: usize = 32768; // 32KB chunks for UDP transfer
pub const MAX_BINARY_SIZE: usize = 50 * 1024 * 1024; // 50MB hard limit
pub const MAX_LOCAL_REPLICATIONS: usize = 16;
pub const MAX_DRONE_NAME_LEN: usize = 64;
pub const RUSTC_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_DIAGNOSTIC_BYTES: u64 = 16 * 1024;
const MINIMAL_DRONE: &str = r#"// Auto-generated minimal Bio-Drone — Generation {{GENERATION}}
// Parent: {{PARENT_NAME}}
// Parent BLAKE3: {{PARENT_HASH}}
// This template is intentionally non-networked and does not self-delete.

const QUEEN_ADDR: &str = {{QUEEN_ADDR_LITERAL}};
const DRONE_NAME: &str = {{DRONE_NAME_LITERAL}};
const GENERATION: u32 = {{GENERATION}};

fn main() {
    println!(
        "[{}] generation={} queen={} mode=minimal-offline",
        DRONE_NAME, GENERATION, QUEEN_ADDR
    );
}
"#;
static STAGE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

struct RemoveOnDrop(PathBuf);

impl RemoveOnDrop {
    fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateId {
    MinimalDrone,
}

impl TemplateId {
    pub const NAMES: &'static [&'static str] = &["minimal-drone"];

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "minimal-drone" => Ok(Self::MinimalDrone),
            _ => Err(format!(
                "unknown template '{value}'; available: {}",
                Self::NAMES.join(", ")
            )),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MinimalDrone => "minimal-drone",
        }
    }

    fn source(self) -> &'static str {
        match self {
            Self::MinimalDrone => MINIMAL_DRONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub template: TemplateId,
    pub drone_name: String,
    pub generation: u32,
    pub queen_addr: SocketAddr,
    pub parent_name: String,
    pub parent_hash: String,
    pub output_root: PathBuf,
    pub source_name: String,
    pub binary_name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RenderedSource {
    pub template: String,
    pub source: String,
    pub blake3: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GeneratedArtifact {
    pub source_path: String,
    pub binary_path: String,
    pub source_blake3: String,
    pub binary_blake3: String,
    pub size_bytes: u64,
    pub compile_time_ms: u64,
}

fn io_invalid(message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message.into())
}

pub fn validate_drone_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > MAX_DRONE_NAME_LEN {
        return Err(format!(
            "drone name must contain 1..={MAX_DRONE_NAME_LEN} ASCII characters"
        ));
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("drone name may contain only ASCII letters, digits, '-' and '_'".to_string());
    }
    Ok(())
}

pub fn validate_simple_filename(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 128 {
        return Err("output filename must contain 1..=128 characters".to_string());
    }
    let path = Path::new(name);
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err("output must be a simple contained filename".to_string());
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"))
            .is_some_and(|suffix| {
                matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            });
    if reserved {
        return Err("output filename is reserved by Windows".to_string());
    }
    Ok(())
}

pub fn canonical_output_root(root: &Path) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(root)
        .map_err(|error| format!("cannot resolve output root {}: {error}", root.display()))?;
    if !canonical.is_dir() {
        return Err(format!(
            "output root is not a directory: {}",
            root.display()
        ));
    }
    Ok(canonical)
}

pub fn validate_replication_count(count: usize) -> Result<(), String> {
    if (1..=MAX_LOCAL_REPLICATIONS).contains(&count) {
        Ok(())
    } else {
        Err(format!(
            "replication count must be between 1 and {MAX_LOCAL_REPLICATIONS}"
        ))
    }
}

fn stage_path(root: &Path, label: &str, extension: &str) -> PathBuf {
    let sequence = STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    root.join(format!(
        ".ribosome-{}-{sequence}-{nanos}.{label}.{extension}",
        std::process::id()
    ))
}

fn stage_bytes(
    root: &Path,
    label: &str,
    extension: &str,
    bytes: &[u8],
) -> std::io::Result<PathBuf> {
    for _ in 0..16 {
        let path = stage_path(root, label, extension);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
                    drop(file);
                    let _ = fs::remove_file(&path);
                    return Err(error);
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "cannot allocate a unique Ribosome staging path",
    ))
}

fn publish_no_clobber(staged: &Path, target: &Path) -> std::io::Result<()> {
    fs::hard_link(staged, target)?;
    fs::remove_file(staged)
}

fn hash_file(path: &Path) -> std::io::Result<(String, u64)> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    if metadata.len() > MAX_BINARY_SIZE as u64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "artifact too large: {} bytes (max {MAX_BINARY_SIZE})",
                metadata.len()
            ),
        ));
    }
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok((hasher.finalize().to_hex().to_string(), metadata.len()))
}

fn rust_literal(value: &str) -> String {
    format!("{value:?}")
}

fn unresolved_placeholder(source: &str) -> Option<&str> {
    let start = source.find("{{")?;
    let end = source[start + 2..].find("}}")? + start + 4;
    source.get(start..end)
}

#[cfg(windows)]
fn msvc_build_environment() -> std::io::Result<Vec<(String, String)>> {
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
    let vswhere = program_files_x86
        .join("Microsoft Visual Studio")
        .join("Installer")
        .join("vswhere.exe");
    if !vswhere.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Visual Studio locator not found: {}", vswhere.display()),
        ));
    }

    let discovery = Command::new(&vswhere)
        .args([
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ])
        .output()?;
    if !discovery.status.success() {
        return Err(std::io::Error::other(format!(
            "Visual Studio discovery failed with exit code {:?}",
            discovery.status.code()
        )));
    }
    let discovery_stdout = String::from_utf8_lossy(&discovery.stdout);
    let installation = discovery_stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .ok_or_else(|| std::io::Error::other("Visual Studio C++ tools are not installed"))?;
    if installation
        .chars()
        .any(|ch| matches!(ch, '&' | '|' | '<' | '>' | '^'))
    {
        return Err(io_invalid(
            "Visual Studio installation path contains unsafe shell characters",
        ));
    }

    let dev_cmd = Path::new(installation)
        .join("Common7")
        .join("Tools")
        .join("VsDevCmd.bat");
    if !dev_cmd.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "Visual Studio developer environment not found: {}",
                dev_cmd.display()
            ),
        ));
    }

    let environment = Command::new("cmd.exe")
        .args(["/d", "/c", "call"])
        .arg(&dev_cmd)
        .args([
            "-no_logo",
            "-arch=x64",
            "-host_arch=x64",
            ">nul",
            "&&",
            "set",
        ])
        .output()?;
    if !environment.status.success() {
        let stderr = String::from_utf8_lossy(&environment.stderr);
        return Err(std::io::Error::other(format!(
            "Visual Studio developer environment failed with exit code {:?}: {}",
            environment.status.code(),
            stderr.trim()
        )));
    }

    let variables: Vec<(String, String)> = String::from_utf8_lossy(&environment.stdout)
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(key, _)| !key.is_empty())
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    for required in ["PATH", "LIB", "INCLUDE"] {
        if !variables
            .iter()
            .any(|(key, value)| key.eq_ignore_ascii_case(required) && !value.is_empty())
        {
            return Err(std::io::Error::other(format!(
                "Visual Studio environment did not provide {required}"
            )));
        }
    }
    Ok(variables)
}

/// Self-genome reader — reads the current running binary
pub struct Genome;

impl Genome {
    /// Read the current executable's bytes
    pub fn read_self() -> std::io::Result<Vec<u8>> {
        let exe_path = std::env::current_exe()?;
        let data = std::fs::read(&exe_path)?;
        if data.len() > MAX_BINARY_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "binary too large: {} bytes (max {})",
                    data.len(),
                    MAX_BINARY_SIZE
                ),
            ));
        }
        Ok(data)
    }

    /// Get self hash (BLAKE3)
    pub fn self_hash() -> std::io::Result<String> {
        let data = Self::read_self()?;
        Ok(blake3::hash(&data).to_hex().to_string())
    }

    /// Get self path
    pub fn self_path() -> std::io::Result<PathBuf> {
        std::env::current_exe()
    }

    /// Get self name
    pub fn self_name() -> String {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Mitosis — replicate self to a target
pub struct Mitosis {
    pub generation: u32,
    queen_key: QueenKey,
}

impl Mitosis {
    pub fn new(generation: u32, queen_key: QueenKey) -> Self {
        Self {
            generation,
            queen_key,
        }
    }

    /// Replicate self to a local path (cell division on same host)
    pub fn replicate_local(
        &self,
        target_dir: &str,
        new_name: Option<&str>,
    ) -> std::io::Result<ReplicationResult> {
        // Generation limit check
        if self.generation >= bio_protocol::MAX_GENERATION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "generation limit reached: {} >= {}",
                    self.generation,
                    bio_protocol::MAX_GENERATION
                ),
            ));
        }

        let genome = Genome::read_self()?;
        let source_hash = blake3::hash(&genome).to_hex().to_string();
        let source_name = Genome::self_name();

        let target_name = new_name.unwrap_or(&source_name);
        validate_drone_name(target_name).map_err(io_invalid)?;
        let ext = if cfg!(windows) { ".exe" } else { "" };
        let target_file = format!("{}{}", target_name, ext);
        validate_simple_filename(&target_file).map_err(io_invalid)?;
        let root = canonical_output_root(Path::new(target_dir)).map_err(io_invalid)?;
        let target_path = root.join(&target_file);
        let staged = stage_bytes(&root, "clone", "tmp", &genome)?;
        let _staged_cleanup = RemoveOnDrop::new(staged.clone());
        publish_no_clobber(&staged, &target_path)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&target_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&target_path, perms)?;
        }

        let (clone_hash, _) = hash_file(&target_path)?;

        let integrity_match = source_hash == clone_hash;
        Ok(ReplicationResult {
            source_name,
            source_hash,
            target_path: target_path.to_string_lossy().to_string(),
            clone_hash,
            generation: self.generation + 1,
            integrity_match,
            bytes_transferred: genome.len(),
        })
    }

    /// Replicate self to a remote host via UDP (chunked transfer)
    pub async fn replicate_remote(
        &self,
        socket: &UdpSocket,
        target_addr: SocketAddr,
    ) -> std::io::Result<ReplicationResult> {
        if self.generation >= bio_protocol::MAX_GENERATION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "generation limit reached",
            ));
        }

        let genome = Genome::read_self()?;
        let source_hash = blake3::hash(&genome).to_hex().to_string();
        let source_name = Genome::self_name();
        let total_chunks = genome.len().div_ceil(GENOME_CHUNK_SIZE);

        // Send CLONE request first
        let clone_payload = bio_protocol::encode_fields(&[
            ("name", source_name.as_bytes()),
            ("hash", source_hash.as_bytes()),
            ("size", &(genome.len() as u64).to_le_bytes()),
            ("chunks", &(total_chunks as u32).to_le_bytes()),
            ("gen", &(self.generation + 1).to_le_bytes()),
        ]);
        let mut clone_msg = BioMessage::with_flags(
            BioOp::Clone,
            flags::QUEEN_ORIGIN,
            self.generation,
            clone_payload,
        );
        self.queen_key.sign(&mut clone_msg);
        socket.send_to(&clone_msg.encode(), target_addr).await?;

        // Send genome chunks
        for (i, chunk) in genome.chunks(GENOME_CHUNK_SIZE).enumerate() {
            let mut chunk_payload = Vec::with_capacity(8 + chunk.len());
            chunk_payload.extend_from_slice(&(i as u32).to_le_bytes());
            chunk_payload.extend_from_slice(&(total_chunks as u32).to_le_bytes());
            chunk_payload.extend_from_slice(chunk);

            let mut genome_msg = BioMessage::new(BioOp::Genome, self.generation, chunk_payload);
            self.queen_key.sign(&mut genome_msg);
            socket.send_to(&genome_msg.encode(), target_addr).await?;

            // Small delay to avoid overwhelming UDP buffer
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        Ok(ReplicationResult {
            source_name,
            source_hash: source_hash.clone(),
            target_path: format!("{}@remote", target_addr),
            clone_hash: source_hash, // Assumed match; receiver will verify
            generation: self.generation + 1,
            integrity_match: true,
            bytes_transferred: genome.len(),
        })
    }
}

/// Result of a replication operation
#[derive(Debug)]
pub struct ReplicationResult {
    pub source_name: String,
    pub source_hash: String,
    pub target_path: String,
    pub clone_hash: String,
    pub generation: u32,
    pub integrity_match: bool,
    pub bytes_transferred: usize,
}

impl serde::Serialize for ReplicationResult {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = s.serialize_struct("ReplicationResult", 7)?;
        state.serialize_field("source_name", &self.source_name)?;
        state.serialize_field("source_hash", &self.source_hash)?;
        state.serialize_field("target_path", &self.target_path)?;
        state.serialize_field("clone_hash", &self.clone_hash)?;
        state.serialize_field("generation", &self.generation)?;
        state.serialize_field("integrity_match", &self.integrity_match)?;
        state.serialize_field("bytes_transferred", &self.bytes_transferred)?;
        state.end()
    }
}

/// Genome receiver — handles incoming CLONE + GENOME messages
pub struct GenomeReceiver {
    pub name: String,
    pub expected_size: usize,
    pub expected_chunks: usize,
    pub expected_hash: String,
    pub generation: u32,
    chunks: Vec<Option<Vec<u8>>>,
    received: usize,
}

impl GenomeReceiver {
    pub fn new(name: String, size: usize, chunks: usize, hash: String, generation: u32) -> Self {
        Self {
            name,
            expected_size: size,
            expected_chunks: chunks,
            expected_hash: hash,
            generation,
            chunks: vec![None; chunks],
            received: 0,
        }
    }

    /// Add a received chunk
    pub fn add_chunk(&mut self, index: usize, data: Vec<u8>) -> bool {
        if index < self.chunks.len() && self.chunks[index].is_none() {
            self.chunks[index] = Some(data);
            self.received += 1;
        }
        self.is_complete()
    }

    pub fn is_complete(&self) -> bool {
        self.received >= self.expected_chunks
    }

    /// Assemble the full genome and verify
    pub fn assemble(&self) -> Result<Vec<u8>, String> {
        if !self.is_complete() {
            return Err(format!(
                "incomplete: {}/{} chunks",
                self.received, self.expected_chunks
            ));
        }
        let mut genome = Vec::with_capacity(self.expected_size);
        for (i, chunk) in self.chunks.iter().enumerate() {
            match chunk {
                Some(data) => genome.extend_from_slice(data),
                None => return Err(format!("missing chunk {}", i)),
            }
        }

        // Verify hash
        let actual_hash = blake3::hash(&genome).to_hex().to_string();
        if actual_hash != self.expected_hash {
            return Err(format!(
                "hash mismatch: expected {} got {}",
                self.expected_hash, actual_hash
            ));
        }

        Ok(genome)
    }

    /// Save an assembled genome without starting a process.
    pub fn save_verified(&self, target_dir: &str) -> Result<PathBuf, String> {
        let genome = self.assemble()?;
        validate_drone_name(&self.name)?;
        let ext = if cfg!(windows) { ".exe" } else { "" };
        let target_name = format!("{}-gen{}{}", self.name, self.generation, ext);
        validate_simple_filename(&target_name)?;
        let root = canonical_output_root(Path::new(target_dir))?;
        let target_path = root.join(target_name);
        let staged = stage_bytes(&root, "received", "tmp", &genome).map_err(|e| e.to_string())?;
        let _staged_cleanup = RemoveOnDrop::new(staged.clone());
        if let Err(error) = publish_no_clobber(&staged, &target_path) {
            return Err(error.to_string());
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&target_path)
                .map_err(|e| e.to_string())?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&target_path, perms).map_err(|e| e.to_string())?;
        }

        let (actual_hash, actual_size) = hash_file(&target_path).map_err(|e| e.to_string())?;
        if actual_hash != self.expected_hash || actual_size != self.expected_size as u64 {
            let _ = fs::remove_file(&target_path);
            return Err("published genome failed post-commit verification".to_string());
        }

        Ok(target_path)
    }

    /// Automatic process creation is intentionally disabled.
    #[deprecated(note = "automatic spawn is disabled; use save_verified")]
    pub fn save_and_spawn(&self, _target_dir: &str) -> Result<u32, String> {
        Err("automatic spawn is disabled; use save_verified and start explicitly".to_string())
    }
}

/// Ribosome — code generation from templates
pub struct Ribosome;

impl Ribosome {
    pub fn templates() -> &'static [&'static str] {
        TemplateId::NAMES
    }

    pub fn default_source_name(drone_name: &str) -> String {
        format!("{drone_name}.drone.rs")
    }

    pub fn default_binary_name(drone_name: &str) -> String {
        if cfg!(windows) {
            format!("{drone_name}.drone.exe")
        } else {
            format!("{drone_name}.drone")
        }
    }

    fn validate_render_request(request: &GenerationRequest) -> Result<(), String> {
        validate_drone_name(&request.drone_name)?;
        validate_drone_name(&request.parent_name)?;
        validate_simple_filename(&request.source_name)?;
        validate_simple_filename(&request.binary_name)?;
        if request.source_name == request.binary_name {
            return Err("source and binary filenames must differ".to_string());
        }
        if request.generation > bio_protocol::MAX_GENERATION {
            return Err(format!(
                "generation {} exceeds maximum {}",
                request.generation,
                bio_protocol::MAX_GENERATION
            ));
        }
        if request.parent_hash.len() != 64
            || !request
                .parent_hash
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("parent hash must be exactly 64 hexadecimal characters".to_string());
        }
        Ok(())
    }

    pub fn validate_request(request: &GenerationRequest) -> Result<PathBuf, String> {
        Self::validate_render_request(request)?;
        let root = canonical_output_root(&request.output_root)?;
        for target in [
            root.join(&request.source_name),
            root.join(&request.binary_name),
        ] {
            if target.exists() {
                return Err(format!(
                    "refusing to overwrite existing target: {}",
                    target.display()
                ));
            }
        }
        Ok(root)
    }

    pub fn render(request: &GenerationRequest) -> Result<RenderedSource, String> {
        Self::validate_render_request(request)?;
        let mut source = request.template.source().to_string();
        let generation = request.generation.to_string();
        let parent_name = request.parent_name.replace(['\r', '\n'], " ");
        source = source.replace("{{GENERATION}}", &generation);
        source = source.replace("{{PARENT_NAME}}", &parent_name);
        source = source.replace("{{PARENT_HASH}}", &request.parent_hash.to_ascii_lowercase());
        source = source.replace(
            "{{QUEEN_ADDR_LITERAL}}",
            &rust_literal(&request.queen_addr.to_string()),
        );
        source = source.replace("{{DRONE_NAME_LITERAL}}", &rust_literal(&request.drone_name));
        if let Some(placeholder) = unresolved_placeholder(&source) {
            return Err(format!("unresolved template placeholder: {placeholder}"));
        }
        let hash = blake3::hash(source.as_bytes()).to_hex().to_string();
        Ok(RenderedSource {
            template: request.template.name().to_string(),
            source,
            blake3: hash,
        })
    }

    pub fn planned_paths(request: &GenerationRequest) -> Result<(PathBuf, PathBuf), String> {
        let root = Self::validate_request(request)?;
        Ok((
            root.join(&request.source_name),
            root.join(&request.binary_name),
        ))
    }

    pub fn generate(request: &GenerationRequest) -> Result<GeneratedArtifact, String> {
        let root = Self::validate_request(request)?;
        let rendered = Self::render(request)?;
        let source_path = root.join(&request.source_name);
        let binary_path = root.join(&request.binary_name);
        let staged_source = stage_bytes(&root, "source", "rs", rendered.source.as_bytes())
            .map_err(|error| format!("cannot stage generated source: {error}"))?;
        let _source_cleanup = RemoveOnDrop::new(staged_source.clone());
        let staged_binary = stage_path(&root, "binary", if cfg!(windows) { "exe" } else { "bin" });
        let _binary_cleanup = RemoveOnDrop::new(staged_binary.clone());

        let compile = match Self::compile_staged(&staged_source, &staged_binary, RUSTC_TIMEOUT) {
            Ok(result) => result,
            Err(error) => {
                return Err(format!("cannot run rustc: {error}"));
            }
        };
        if !compile.success {
            return Err(format!(
                "generated source did not compile{}: {}",
                if compile.timed_out {
                    " before timeout"
                } else {
                    ""
                },
                compile
                    .errors
                    .unwrap_or_else(|| "rustc failed without diagnostics".to_string())
            ));
        }

        let (staged_source_hash, _) = hash_file(&staged_source)
            .map_err(|error| format!("cannot verify staged source: {error}"))?;
        if staged_source_hash != rendered.blake3 {
            return Err("staged source hash mismatch".to_string());
        }
        let (staged_binary_hash, staged_binary_size) = hash_file(&staged_binary)
            .map_err(|error| format!("cannot verify staged binary: {error}"))?;
        if staged_binary_size == 0 {
            return Err("rustc produced an empty binary".to_string());
        }

        if let Err(error) = publish_no_clobber(&staged_source, &source_path) {
            return Err(format!("cannot publish generated source: {error}"));
        }
        if let Err(error) = publish_no_clobber(&staged_binary, &binary_path) {
            let _ = fs::remove_file(&source_path);
            return Err(format!("cannot publish generated binary: {error}"));
        }

        let (source_hash, _) = hash_file(&source_path)
            .map_err(|error| format!("cannot verify published source: {error}"))?;
        let (binary_hash, size_bytes) = hash_file(&binary_path)
            .map_err(|error| format!("cannot verify published binary: {error}"))?;
        if source_hash != rendered.blake3
            || binary_hash != staged_binary_hash
            || size_bytes != staged_binary_size
        {
            let _ = fs::remove_file(&source_path);
            let _ = fs::remove_file(&binary_path);
            return Err("published artifact verification failed".to_string());
        }

        Ok(GeneratedArtifact {
            source_path: source_path.to_string_lossy().to_string(),
            binary_path: binary_path.to_string_lossy().to_string(),
            source_blake3: source_hash,
            binary_blake3: binary_hash,
            size_bytes,
            compile_time_ms: compile.compile_time_ms,
        })
    }

    pub fn planned_replication_paths(
        output_root: &Path,
        base_name: &str,
        count: usize,
    ) -> Result<Vec<PathBuf>, String> {
        validate_drone_name(base_name)?;
        validate_replication_count(count)?;
        let root = canonical_output_root(output_root)?;
        let extension = if cfg!(windows) { ".exe" } else { "" };
        let mut paths = Vec::with_capacity(count);
        for index in 1..=count {
            let filename = format!("{base_name}-copy-{index}{extension}");
            validate_simple_filename(&filename)?;
            let target = root.join(filename);
            if target.exists() {
                return Err(format!(
                    "refusing to overwrite existing target: {}",
                    target.display()
                ));
            }
            paths.push(target);
        }
        Ok(paths)
    }

    pub fn replicate_local_copies(
        output_root: &Path,
        base_name: &str,
        count: usize,
    ) -> Result<Vec<ReplicationResult>, String> {
        let targets = Self::planned_replication_paths(output_root, base_name, count)?;
        let root = canonical_output_root(output_root)?;
        let genome = Genome::read_self().map_err(|error| error.to_string())?;
        let source_name = Genome::self_name();
        let source_hash = blake3::hash(&genome).to_hex().to_string();
        let mut published = Vec::new();
        let mut results = Vec::with_capacity(targets.len());

        for target in targets {
            let staged = stage_bytes(&root, "replica", "tmp", &genome)
                .map_err(|error| format!("cannot stage replica: {error}"))?;
            let _staged_cleanup = RemoveOnDrop::new(staged.clone());
            if let Err(error) = publish_no_clobber(&staged, &target) {
                for path in &published {
                    let _ = fs::remove_file(path);
                }
                return Err(format!(
                    "cannot publish replica {}: {error}",
                    target.display()
                ));
            }
            let (clone_hash, clone_size) = match hash_file(&target) {
                Ok(receipt) => receipt,
                Err(error) => {
                    let _ = fs::remove_file(&target);
                    for path in &published {
                        let _ = fs::remove_file(path);
                    }
                    return Err(format!(
                        "cannot verify replica {}: {error}",
                        target.display()
                    ));
                }
            };
            if clone_hash != source_hash || clone_size != genome.len() as u64 {
                let _ = fs::remove_file(&target);
                for path in &published {
                    let _ = fs::remove_file(path);
                }
                return Err(format!("replica integrity mismatch: {}", target.display()));
            }
            published.push(target.clone());
            results.push(ReplicationResult {
                source_name: source_name.clone(),
                source_hash: source_hash.clone(),
                target_path: target.to_string_lossy().to_string(),
                clone_hash,
                generation: 1,
                integrity_match: true,
                bytes_transferred: genome.len(),
            });
        }
        Ok(results)
    }

    /// Compatibility helper: synthesize one source with atomic no-clobber publication.
    pub fn synthesize(
        template: &str,
        substitutions: &[(&str, &str)],
        output_path: &str,
    ) -> std::io::Result<()> {
        let mut source = template.to_string();
        for (placeholder, value) in substitutions {
            source = source.replace(placeholder, value);
        }
        if let Some(placeholder) = unresolved_placeholder(&source) {
            return Err(io_invalid(format!(
                "unresolved template placeholder: {placeholder}"
            )));
        }
        let output_path = Path::new(output_path);
        let parent = output_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let root = canonical_output_root(parent).map_err(io_invalid)?;
        let filename = output_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io_invalid("source output must have a UTF-8 filename"))?;
        validate_simple_filename(filename).map_err(io_invalid)?;
        let target = root.join(filename);
        let staged = stage_bytes(&root, "source", "rs", source.as_bytes())?;
        let _staged_cleanup = RemoveOnDrop::new(staged.clone());
        publish_no_clobber(&staged, &target)?;
        Ok(())
    }

    /// Compatibility helper: compile to an atomically published, no-clobber artifact.
    pub fn compile(source_path: &str, output_path: &str) -> std::io::Result<CompileResult> {
        let source = fs::canonicalize(source_path)?;
        if !source.is_file() {
            return Err(io_invalid("source path is not a file"));
        }
        let output_path = Path::new(output_path);
        let parent = output_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let root = canonical_output_root(parent).map_err(io_invalid)?;
        let filename = output_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io_invalid("binary output must have a UTF-8 filename"))?;
        validate_simple_filename(filename).map_err(io_invalid)?;
        let target = root.join(filename);
        if target.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "refusing to overwrite existing target: {}",
                    target.display()
                ),
            ));
        }
        let staged = stage_path(&root, "binary", if cfg!(windows) { "exe" } else { "bin" });
        let _staged_cleanup = RemoveOnDrop::new(staged.clone());
        let mut result = Self::compile_staged(&source, &staged, RUSTC_TIMEOUT)?;
        if result.success {
            publish_no_clobber(&staged, &target)?;
            result.output_path = target.to_string_lossy().to_string();
        }
        Ok(result)
    }

    fn compile_staged(
        source_path: &Path,
        output_path: &Path,
        timeout: Duration,
    ) -> std::io::Result<CompileResult> {
        let root = output_path.parent().unwrap_or_else(|| Path::new("."));
        let diagnostics_path = stage_path(root, "rustc", "log");
        let diagnostics = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&diagnostics_path)?;
        let _diagnostics_cleanup = RemoveOnDrop::new(diagnostics_path.clone());
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let start = Instant::now();
        let mut compiler_command = Command::new(compiler);
        #[cfg(windows)]
        compiler_command.envs(msvc_build_environment()?);
        let mut child = compiler_command
            .arg("--crate-name")
            .arg("ribosome_generated")
            .arg("--edition")
            .arg("2021")
            .arg("-O")
            .arg("-o")
            .arg(output_path)
            .arg(source_path)
            .stdout(Stdio::null())
            .stderr(Stdio::from(diagnostics))
            .spawn()?;
        let (status, timed_out) = loop {
            match child.try_wait() {
                Ok(Some(status)) => break (status, false),
                Ok(None) => {}
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error);
                }
            }
            if start.elapsed() >= timeout {
                let _ = child.kill();
                break (child.wait()?, true);
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let duration = start.elapsed();
        let mut diagnostics = Vec::new();
        File::open(&diagnostics_path)?
            .take(MAX_DIAGNOSTIC_BYTES)
            .read_to_end(&mut diagnostics)?;
        let errors = String::from_utf8_lossy(&diagnostics).trim().to_string();
        let success = status.success() && !timed_out && output_path.is_file();
        Ok(CompileResult {
            success,
            output_path: output_path.to_string_lossy().to_string(),
            compile_time_ms: duration.as_millis() as u64,
            errors: if errors.is_empty() {
                None
            } else {
                Some(errors)
            },
            timed_out,
        })
    }

    /// The only currently implemented template: a non-networked minimal drone.
    pub fn drone_template() -> &'static str {
        MINIMAL_DRONE
    }
}

/// Result of a compilation
#[derive(Debug, serde::Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub output_path: String,
    pub compile_time_ms: u64,
    pub errors: Option<String>,
    pub timed_out: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);

    impl Fixture {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "ribosome-{label}-{}-{}",
                std::process::id(),
                STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn request(root: &Path) -> GenerationRequest {
        GenerationRequest {
            template: TemplateId::MinimalDrone,
            drone_name: "drone_1".to_string(),
            generation: 2,
            queen_addr: "127.0.0.1:9000".parse().unwrap(),
            parent_name: "ribosome-synth".to_string(),
            parent_hash: "ab".repeat(32),
            output_root: root.to_path_buf(),
            source_name: "drone_1.drone.rs".to_string(),
            binary_name: "drone_1.drone.exe".to_string(),
        }
    }

    #[test]
    fn rendering_is_deterministic_complete_and_non_destructive() {
        let fixture = Fixture::new("render");
        let request = request(&fixture.0);
        let first = Ribosome::render(&request).unwrap();
        let second = Ribosome::render(&request).unwrap();
        assert_eq!(first.source, second.source);
        assert_eq!(first.blake3, second.blake3);
        assert!(!first.source.contains("{{"));
        assert!(!first.source.contains("remove_file"));
        assert!(!first.source.contains("UdpSocket"));
    }

    #[test]
    fn rust_literals_escape_control_characters_and_quotes() {
        let literal = rust_literal("node\n\"quoted\"");
        assert_eq!(literal, "\"node\\n\\\"quoted\\\"\"");
    }

    #[test]
    fn request_validation_rejects_names_paths_and_excess_generation() {
        let fixture = Fixture::new("validation");
        let mut invalid_request = request(&fixture.0);
        invalid_request.drone_name = "../escape".to_string();
        assert!(Ribosome::validate_request(&invalid_request).is_err());
        invalid_request = request(&fixture.0);
        invalid_request.binary_name = "subdir/drone.exe".to_string();
        assert!(Ribosome::validate_request(&invalid_request).is_err());
        invalid_request = request(&fixture.0);
        invalid_request.generation = bio_protocol::MAX_GENERATION + 1;
        assert!(Ribosome::validate_request(&invalid_request).is_err());
    }

    #[test]
    fn atomic_synthesis_is_no_clobber() {
        let fixture = Fixture::new("no-clobber");
        let target = fixture.0.join("generated.rs");
        Ribosome::synthesize("fn main() {}", &[], target.to_str().unwrap()).unwrap();
        let error = Ribosome::synthesize("changed", &[], target.to_str().unwrap()).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(target).unwrap(), "fn main() {}");
    }

    #[test]
    fn artifact_receipt_matches_committed_bytes() {
        let fixture = Fixture::new("artifact");
        let staged = stage_bytes(&fixture.0, "artifact", "tmp", b"verified-bytes").unwrap();
        let target = fixture.0.join("artifact.bin");
        publish_no_clobber(&staged, &target).unwrap();
        let (hash, size) = hash_file(&target).unwrap();
        assert_eq!(hash, blake3::hash(b"verified-bytes").to_hex().to_string());
        assert_eq!(size, 14);
    }

    #[test]
    fn generated_artifact_compiles_and_runs() {
        let fixture = Fixture::new("compiled-artifact");
        let artifact = Ribosome::generate(&request(&fixture.0)).unwrap();
        assert!(Path::new(&artifact.source_path).is_file());
        assert!(Path::new(&artifact.binary_path).is_file());
        assert!(artifact.size_bytes > 0);

        let output = Command::new(&artifact.binary_path).output().unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("mode=minimal-offline"));
    }

    #[test]
    fn local_replication_count_is_bounded() {
        assert!(validate_replication_count(0).is_err());
        assert!(validate_replication_count(1).is_ok());
        assert!(validate_replication_count(MAX_LOCAL_REPLICATIONS).is_ok());
        assert!(validate_replication_count(MAX_LOCAL_REPLICATIONS + 1).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn discovered_msvc_environment_exposes_the_linker() {
        let environment = msvc_build_environment().unwrap();
        let output = Command::new("where.exe")
            .arg("link.exe")
            .envs(environment)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("link.exe"));
    }
}
