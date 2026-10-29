//! Filesystem transport stays outside the frontend's typed metadata consumers.
use rustgm::support::crates::{self, CrateIdentity, Dependencies, Dependency};
use rustgm::support::hash::HashMap;
use std::{fs, io::Read, path::PathBuf, sync::Arc};

#[derive(Default)]
pub struct Options {
    pub name: Option<String>,
    pub disambiguator: Option<String>,
    pub library: bool,
    pub metadata: Option<String>,
    pub externs: Vec<String>,
    pub dependency_metadata: Vec<String>,
    pub source_core: Option<String>,
    pub source_client: Option<String>,
    pub source_profile: Option<rustgm::lower::instances::SourcePublication>,
}
impl Options {
    pub fn select_source_profile(&mut self, value: Option<&str>) -> Result<(), String> {
        if self.source_profile.is_some() {
            return Err("duplicate --source-profile".into());
        }
        match value {
            Some("module-safety-v8") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleSafety);
            }
            Some("module-callables-v7") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleCallables);
            }
            Some("module-compatibility-v6") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleCompatibility);
            }
            Some("module-definitions-v5") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleDefinitions);
            }
            Some("module-availability-v3") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleAvailability);
            }
            Some("module-contracts-v2") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleContracts);
            }
            Some("provided-contracts-v1") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ProvidedContracts);
            }
            _ => {
                return Err(
                    "--source-profile requires module-contracts-v2, module-availability-v3, module-definitions-v5, module-compatibility-v6, module-callables-v7, module-safety-v8 or provided-contracts-v1".into(),
                );
            }
        }
        Ok(())
    }
    pub fn source_mode(&mut self, client: bool, provider: Option<String>) -> Result<(), String> {
        let flag = if client {
            "--source-client"
        } else {
            "--source-core"
        };
        let provider = provider.ok_or_else(|| format!("{flag} requires NAME@DISAMBIGUATOR"))?;
        if client {
            self.source_client = Some(provider);
        } else {
            self.source_core = Some(provider);
        }
        Ok(())
    }
    pub fn validate(&self, mode: &str, count: bool, output: Option<&str>) -> Result<(), String> {
        if self.source_profile.is_some()
            && (self.source_core.is_none()
                || self.source_client.is_some()
                || !self.library
                || mode != "--emit-lowir"
                || self.metadata.is_none()
                || count
                || self.identity()?.is_none())
        {
            return Err("--source-profile requires SourceCore library LowIR and metadata output with full identity".into());
        }
        if self.source_core.is_some() && self.source_client.is_some() {
            return Err("--source-core and --source-client are mutually exclusive".into());
        }
        if self.source_client.is_some() {
            if mode != "--emit-lowir" {
                return Err("--source-client requires --emit-lowir".into());
            }
            self.bindings_mode()?;
        }
        let enabled = self.library
            || self.disambiguator.is_some()
            || self.metadata.is_some()
            || self.dependency_count() != 0;
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
            self.bindings_mode()?;
            if mode != "--emit-lowir"
                && (self.library || self.metadata.is_some() || self.dependency_count() != 0)
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
        if self.dependency_count() > 1024 {
            return Err("external crate arguments exceed profile budget".into());
        }
        Ok(())
    }
    pub fn dependency_count(&self) -> usize {
        self.externs
            .len()
            .saturating_add(self.dependency_metadata.len())
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
        Ok(rustgm::resolve::bindings::Context {
            crate_name: self.name.clone(),
            current: self.identity()?,
            dependencies: self.dependencies()?,
            mode: self.bindings_mode()?,
        })
    }
    fn bindings_mode(&self) -> Result<rustgm::support::bindings::Mode, String> {
        if self.source_core.is_some() && self.source_client.is_some() {
            return Err("--source-core and --source-client are mutually exclusive".into());
        }
        if let Some(provider) = &self.source_client {
            let (name, disambiguator) = provider
                .split_once('@')
                .ok_or("--source-client requires NAME@DISAMBIGUATOR")?;
            let provider = CrateIdentity::new(name, disambiguator).map_err(str::to_owned)?;
            let current = self
                .identity()?
                .ok_or("--source-client requires complete current crate identity")?;
            if current == provider {
                return Err(
                    "source-client provider must differ from current crate identity".into(),
                );
            }
            return Ok(rustgm::support::bindings::Mode::SourceClient { provider });
        }
        if let Some(provider) = &self.source_core {
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
            Ok(rustgm::support::bindings::Mode::Source { provider })
        } else {
            Ok(rustgm::support::bindings::Mode::Bootstrap)
        }
    }
    pub fn dependencies(&self) -> Result<Option<Arc<Dependencies>>, String> {
        if self.dependency_count() == 0 {
            return Ok(None);
        }
        let mut paths = HashMap::default();
        let mut artifacts = Vec::new();
        let mut aliases = Vec::new();
        let mut total = 0usize;
        let named = self.externs.iter().map(|argument| {
            argument
                .split_once('=')
                .map(|(alias, path)| (Some(alias), path))
                .ok_or("--extern requires ALIAS=METADATA")
        });
        let inventory = self
            .dependency_metadata
            .iter()
            .map(|path| Ok((None, path.as_str())));
        for input in named.chain(inventory) {
            let (alias, path) = input?;
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
            if let Some(alias) = alias {
                aliases.push(Dependency::new(alias.to_owned(), identity).map_err(str::to_owned)?);
            }
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
