//! Filesystem transport stays outside the frontend's typed metadata consumers.
use rustgm::support::crates::{self, CrateIdentity, Dependencies, Dependency};
use std::{collections::HashMap, fs, io::Read, path::PathBuf, sync::Arc};

#[derive(Default)]
pub struct Options {
    pub name: Option<String>,
    pub disambiguator: Option<String>,
    pub library: bool,
    pub metadata: Option<String>,
    pub externs: Vec<String>,
    pub source_core: Option<String>,
}
impl Options {
    pub fn validate(&self, mode: &str, count: bool, output: Option<&str>) -> Result<(), String> {
        let enabled = self.library
            || self.disambiguator.is_some()
            || self.metadata.is_some()
            || !self.externs.is_empty();
        if enabled && mode != "--emit-lowir" && self.source_core.is_none() {
            return Err("crate artifact options require --emit-lowir".into());
        }
        if self.source_core.is_some() {
            if !matches!(
                mode,
                "--emit-names"
                    | "--emit-expanded"
                    | "--emit-types"
                    | "--emit-semantics"
                    | "--emit-trait-semantics"
                    | "--emit-generic-semantics"
                    | "--emit-patterns"
                    | "--emit-layouts"
                    | "--emit-body-ir"
                    | "--emit-ownership"
                    | "--emit-regions"
                    | "--emit-loans"
                    | "--emit-constants"
                    | "--emit-lowir"
            ) {
                return Err("source-core configuration requires a name or semantic view".into());
            }
            self.context()?;
            if mode != "--emit-lowir"
                && (self.library || self.metadata.is_some() || !self.externs.is_empty())
            {
                return Err("source-core artifact options require --emit-lowir".into());
            }
        }
        if self.library && self.identity()?.is_none() {
            return Err("library output requires --crate-name and --crate-disambiguator".into());
        }
        if self.metadata.is_some() && (!self.library || count) {
            return Err(
                "--metadata-output requires library LowIR output without --count-lowir".into(),
            );
        }
        if let Some(metadata) = &self.metadata
            && let Some(output) = output
            && absolute(metadata)? == absolute(output)?
        {
            return Err("metadata and LowIR output paths must differ".into());
        }
        if self.externs.len() > 1024 {
            return Err("external crate arguments exceed profile budget".into());
        }
        Ok(())
    }
    pub fn identity(&self) -> Result<Option<CrateIdentity>, String> {
        self.disambiguator
            .as_ref()
            .map(|disambiguator| {
                let name = self
                    .name
                    .as_deref()
                    .ok_or("--crate-disambiguator requires --crate-name")?;
                CrateIdentity::new(name, disambiguator).map_err(str::to_owned)
            })
            .transpose()
    }
    pub fn context(&self) -> Result<rustgm::resolve::bindings::Context, String> {
        let mode = if let Some(provider) = &self.source_core {
            let (name, disambiguator) = provider
                .split_once('@')
                .ok_or("--source-core requires NAME@DISAMBIGUATOR")?;
            let provider = CrateIdentity::new(name, disambiguator).map_err(str::to_owned)?;
            let current = self
                .identity()?
                .ok_or("--source-core requires complete current crate identity")?;
            if current != provider {
                return Err("source-core provider differs from current crate identity".into());
            }
            rustgm::support::bindings::Mode::Source { provider }
        } else {
            rustgm::support::bindings::Mode::Bootstrap
        };
        Ok(rustgm::resolve::bindings::Context {
            crate_name: self.name.clone(),
            current: self.identity()?,
            dependencies: self.dependencies()?,
            mode,
        })
    }
    pub fn dependencies(&self) -> Result<Option<Arc<Dependencies>>, String> {
        if self.externs.is_empty() {
            return Ok(None);
        }
        let mut paths = HashMap::new();
        let mut artifacts = Vec::new();
        let mut aliases = Vec::new();
        let mut total = 0usize;
        for argument in &self.externs {
            let (alias, path) = argument
                .split_once('=')
                .ok_or("--extern requires ALIAS=METADATA")?;
            let path = fs::canonicalize(path).map_err(|error| error.to_string())?;
            let identity = if let Some(identity) = paths.get(&path) {
                CrateIdentity::clone(identity)
            } else {
                let remaining = 16_000_000usize.saturating_sub(total);
                let mut bytes = Vec::new();
                fs::File::open(&path)
                    .map_err(|error| error.to_string())?
                    .take(
                        u64::try_from(remaining + 1)
                            .map_err(|_| "metadata byte budget overflow")?,
                    )
                    .read_to_end(&mut bytes)
                    .map_err(|error| error.to_string())?;
                if bytes.len() > remaining {
                    return Err("crate artifacts exceed 16 MB input budget".into());
                }
                total += bytes.len();
                let artifact = crates::decode(&bytes).map_err(str::to_owned)?;
                let identity = artifact.identity().clone();
                paths.insert(path, identity.clone());
                artifacts.push(artifact);
                identity
            };
            aliases.push(Dependency::new(alias.to_owned(), identity).map_err(str::to_owned)?);
        }
        Dependencies::new(artifacts, aliases)
            .map(|dependencies| Some(Arc::new(dependencies)))
            .map_err(str::to_owned)
    }
}
fn absolute(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        return fs::canonicalize(path).map_err(|error| error.to_string());
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
    Ok(parent.join(path.file_name().ok_or("output path requires a filename")?))
}
