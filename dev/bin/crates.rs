//! Filesystem transport stays outside the frontend's typed metadata consumers.
use crate::census;
use rustgm::support::crates::{
    self, ArtifactContext, CrateIdentity, Dependencies, Dependency, Guest, OutputKind,
};
use rustgm::support::hash::HashMap;
use std::{fs, io::Read, path::PathBuf, sync::Arc};

const BINARY_METADATA_MAGIC: &str = "rustgm-executable-metadata-v1\n";

pub struct ExecutableMetadata<'a> {
    path: &'a str,
    bytes: Vec<u8>,
}
impl ExecutableMetadata<'_> {
    pub fn publish(self, output: &mut impl std::io::Write) -> Result<(), String> {
        output.flush().map_err(|error| error.to_string())?;
        fs::write(self.path, self.bytes).map_err(|error| error.to_string())
    }
}

#[derive(Default)]
pub struct Options {
    pub name: Option<String>,
    pub disambiguator: Option<String>,
    pub output: Option<OutputKind>,
    pub metadata: Option<String>,
    pub native_links: Option<String>,
    pub externs: Vec<String>,
    pub dependency_metadata: Vec<String>,
    pub source_core: Option<String>,
    pub source_client: Option<String>,
    pub source_profile: Option<rustgm::lower::instances::SourcePublication>,
    /// Analysis-only census records; never a build output.
    pub census: Option<String>,
    pub census_progress: Option<String>,
    pub export_census: Option<String>,
    guests: Vec<Guest>,
    guest_cfgs: Vec<GuestCfg>,
    bringup_guests: bool,
}
struct GuestCfg {
    name: String,
    setting: Option<String>,
    option: &'static str,
}
impl Options {
    /// Consume only arguments owned by this crate/environment boundary.
    pub fn consume(
        &mut self,
        arg: &str,
        args: &mut impl Iterator<Item = String>,
    ) -> Result<bool, String> {
        match arg {
            "--source-core" | "--source-client" => {
                self.source_mode(arg == "--source-client", args.next())?;
            }
            "--source-profile" => self.select_source_profile(args.next().as_deref())?,
            "--crate-name" => {
                self.name = Some(args.next().ok_or("--crate-name requires a name")?);
            }
            "--crate-disambiguator" => {
                self.disambiguator =
                    Some(args.next().ok_or("--crate-disambiguator requires an ID")?);
            }
            "--native-links-output" => self.select_native_output(args.next())?,
            "--census-output" => census::select(&mut self.census, args.next())?,
            "--export-census" => {
                census::select_path(&mut self.export_census, args.next(), "--export-census")?;
            }
            "--census-progress" => {
                census::select_progress(&mut self.census_progress, args.next())?;
            }
            "--metadata-output" => {
                self.metadata = Some(args.next().ok_or("--metadata-output requires a path")?);
            }
            "--bringup-guest" | "--source-guest" => self.guest(arg, args.next().as_deref())?,
            "--bringup-guest-cfg" => self.guest_cfg("--bringup-guest-cfg", args.next())?,
            "--source-guest-cfg" => self.guest_cfg("--source-guest-cfg", args.next())?,
            "--crate-type=lib" => self.output = Some(OutputKind::Library),
            "--crate-type=bin" => self.output = Some(OutputKind::Binary),
            "--extern" | "--dependency-metadata" => {
                if self.dependency_count() >= 1024 {
                    return Err("external crate arguments exceed profile budget".into());
                }
                let value = args.next().ok_or(if arg == "--extern" {
                    "--extern requires [noprelude:]ALIAS=METADATA"
                } else {
                    "--dependency-metadata requires a path"
                })?;
                if arg == "--extern" {
                    self.externs.push(value);
                } else {
                    self.dependency_metadata.push(value);
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
    /// `--source-guest NAME[@EDITION]=ROOT.rs`: one more crate in this session.
    fn guest(&mut self, option: &str, value: Option<&str>) -> Result<(), String> {
        self.bringup_guests |= option == "--bringup-guest";
        let (name, root) = value
            .and_then(|value| value.split_once('='))
            .filter(|(_, root)| !root.is_empty())
            .ok_or_else(|| format!("{option} requires NAME=ROOT.rs"))?;
        if self.guests.len() >= 64 {
            return Err("guest crates exceed the source graph limit of 64".into());
        }
        let (name, edition) = match name.split_once('@') {
            Some((name, edition)) => {
                use rustgm::support::edition::Edition;
                let edition = match edition {
                    "2015" => Edition::Rust2015,
                    "2018" => Edition::Rust2018,
                    "2021" => Edition::Rust2021,
                    "2024" => Edition::Rust2024,
                    _ => {
                        return Err(format!("{option} edition must be 2015, 2018, 2021 or 2024"));
                    }
                };
                (name, Some(edition))
            }
            None => (name, None),
        };
        let guest = Guest::new(name, PathBuf::from(root))
            .map_err(|message| format!("{option} {name}: {message}"))?;
        self.guests.push(match edition {
            Some(edition) => guest.with_edition(edition),
            None => guest,
        });
        Ok(())
    }
    /// `--source-guest-cfg NAME[=SETTING]`: the guest crate NAME has cfg
    /// options of its own instead of the session's `--cfg` options; SETTING,
    /// spelled as for `--cfg`, is one of them.
    fn guest_cfg(&mut self, option: &'static str, value: Option<String>) -> Result<(), String> {
        self.bringup_guests |= option == "--bringup-guest-cfg";
        let required = || format!("{option} requires NAME[=SETTING]");
        let value = value.ok_or_else(required)?;
        let (name, setting) = match value.split_once('=') {
            Some((name, setting)) => (name, Some(setting.to_owned())),
            None => (value.as_str(), None),
        };
        if name.is_empty() || self.guest_cfgs.len() >= 4096 {
            return Err(required());
        }
        self.guest_cfgs.push(GuestCfg {
            name: name.to_owned(),
            setting,
            option,
        });
        Ok(())
    }
    pub fn select_native_output(&mut self, path: Option<String>) -> Result<(), String> {
        if self.native_links.is_some() {
            return Err("duplicate --native-links-output".into());
        }
        self.native_links = Some(path.ok_or("--native-links-output requires a path")?);
        Ok(())
    }
    pub fn select_source_profile(&mut self, value: Option<&str>) -> Result<(), String> {
        if self.source_profile.is_some() {
            return Err("duplicate --source-profile".into());
        }
        match value {
            Some("bootstrap-bodies-v15") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::BootstrapModuleBodies);
            }
            Some("module-body-operations-v14") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleBodyOperations);
            }
            Some("module-compiler-operations-v11") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::CompilerOperations);
            }
            Some("module-bodies-v13") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleBodies);
            }
            Some("module-branches-v12") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleBranches);
            }
            Some("module-nominals-v10") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleNominals);
            }
            Some("module-conversions-v9") => {
                self.source_profile =
                    Some(rustgm::lower::instances::SourcePublication::ModuleConversions);
            }
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
                    "--source-profile requires module-contracts-v2, module-availability-v3, module-definitions-v5, module-compatibility-v6, module-callables-v7, module-safety-v8, module-conversions-v9, module-nominals-v10, module-compiler-operations-v11, module-branches-v12, module-bodies-v13, module-body-operations-v14, bootstrap-bodies-v15 or provided-contracts-v1".into(),
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
        self.validate_export_census(mode, output)?;
        if let Some(path) = &self.native_links {
            if mode != "--emit-lowir" || count {
                return Err("--native-links-output requires LowIR output without counting".into());
            }
            for other in output.into_iter().chain(self.metadata.as_deref()) {
                if absolute(path)? == absolute(other)? {
                    return Err("native link manifest and other output paths must differ".into());
                }
            }
        }
        self.validate_source_profile(mode, count)?;
        if self.source_core.is_some() && self.source_client.is_some() {
            return Err("--source-core and --source-client are mutually exclusive".into());
        }
        if self.census_progress.is_some() && self.census.is_none() {
            return Err("--census-progress requires --census-output".into());
        }
        if self.census.is_some()
            && (self.source_core.is_none()
                || !matches!(mode, "--emit-types" | "--emit-semantics")
                || count
                || output.is_some())
        {
            return Err("--census-output requires --source-core with --emit-types or --emit-semantics, without -o or counting".into());
        }
        if self.source_client.is_some() {
            if mode != "--emit-lowir" {
                return Err("--source-client requires --emit-lowir".into());
            }
            self.bindings_mode()?;
        }
        let enabled = self.is_library()
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
                    | "--emit-expanded-crate"
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
                && (self.is_library() || self.metadata.is_some() || self.dependency_count() != 0)
            {
                return Err("source-core artifact options require --emit-lowir".into());
            }
        }
        self.validate_guests(mode)?;
        if self.is_library() && self.identity()?.is_none() {
            return Err("library output requires --crate-name and --crate-disambiguator".into());
        }
        if self.metadata.is_some()
            && (mode != "--emit-lowir"
                || count
                || self.output.is_none()
                || self.identity()?.is_none())
        {
            return Err("--metadata-output requires named library or binary LowIR output without --count-lowir".into());
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
    fn validate_source_profile(&self, mode: &str, count: bool) -> Result<(), String> {
        let bootstrap = self.source_profile
            == Some(rustgm::lower::instances::SourcePublication::BootstrapModuleBodies);
        if self.source_profile.is_some()
            && (self.source_core.is_none() != bootstrap
                || self.source_client.is_some()
                || !self.is_library()
                || mode != "--emit-lowir"
                || self.metadata.is_none() && self.export_census.is_none()
                || count
                || self.identity()?.is_none())
        {
            if bootstrap {
                return Err("bootstrap-bodies-v15 requires ordinary library LowIR and metadata output with full identity".into());
            }
            return Err("--source-profile requires SourceCore library LowIR and metadata output with full identity".into());
        }
        Ok(())
    }

    fn validate_export_census(&self, mode: &str, output: Option<&str>) -> Result<(), String> {
        use rustgm::lower::instances::SourcePublication;
        if self.export_census.is_none() {
            return Ok(());
        }
        if self.source_profile != Some(SourcePublication::ModuleBodyOperations)
            || mode != "--emit-lowir"
            || self.metadata.is_some()
            || self.native_links.is_some()
            || output.is_some()
            || !self.guests.is_empty()
            || self.dependency_count() != 0
            || rustgm::support::bringup::selected().is_some()
        {
            return Err(
                "--export-census requires a strict v14 SourceCore library command without artifact paths"
                    .into(),
            );
        }
        Ok(())
    }

    /// Guest crates need a source core to depend on and an expanding view.
    /// An analysis-only census may observe a guest session: it records every
    /// crate's declarations and bodies and still never passes a build.
    fn validate_guests(&self, mode: &str) -> Result<(), String> {
        let option = if self.bringup_guests {
            "--bringup-guest"
        } else {
            "--source-guest"
        };
        if !self.guests.is_empty() {
            if self.source_core.is_none() || self.dependency_count() != 0 || mode == "--emit-names"
            {
                return Err(format!(
                    "{option} requires --source-core and a view that expands the crate, without external crates"
                ));
            }
            if rustgm::support::bringup::selected().is_none()
                && (self.is_library() || self.metadata.is_some())
            {
                return Err(
                    "a strict source graph produces one program without crate metadata".into(),
                );
            }
        }
        for cfg in &self.guest_cfgs {
            if !self.guests.iter().any(|guest| guest.name() == cfg.name) {
                return Err(format!(
                    "{} {}: no {option} names this crate",
                    cfg.option, cfg.name
                ));
            }
        }
        if self.bringup_guests && rustgm::support::bringup::selected().is_none() {
            return Err(
                "--bringup-guest requires the bring-up profile (--bringup-skip-list)".into(),
            );
        }
        Ok(())
    }

    /// Install source roots and cfg scopes after the session's own cfg is complete.
    pub fn configure_guests(
        &mut self,
        config: &mut rustgm::expand::config::Config,
    ) -> Result<(), String> {
        if !self.guests.is_empty() {
            config.set_guest_crates(std::mem::take(&mut self.guests));
        }
        for cfg in &self.guest_cfgs {
            let scope = config.crate_scope(&cfg.name);
            if let Some(setting) = &cfg.setting {
                scope.add(setting).map_err(|e| {
                    format!("{} {}: {}: {}", cfg.option, cfg.name, e.offset, e.message)
                })?;
            }
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
    pub fn is_library(&self) -> bool {
        self.output == Some(OutputKind::Library)
    }
    /// Describe the successfully lowered binary unit without publishing an extern interface.
    pub fn executable_metadata(
        &self,
        report: &rustgm::resolve::expansion::Report,
    ) -> Result<Option<ExecutableMetadata<'_>>, String> {
        use rustgm::support::edition::Edition;
        use std::fmt::Write as _;
        let (Some(path), Some(OutputKind::Binary)) = (self.metadata.as_deref(), self.output) else {
            return Ok(None);
        };
        let identity = self
            .identity()?
            .ok_or("binary metadata requires its actual crate identity")?;
        let edition = report
            .package
            .units
            .first()
            .ok_or("binary metadata requires its actual source root")?
            .edition();
        let edition = match edition {
            Edition::Rust2015 => 2015,
            Edition::Rust2018 => 2018,
            Edition::Rust2021 => 2021,
            Edition::Rust2024 => 2024,
        };
        let mut metadata = String::from(BINARY_METADATA_MAGIC);
        writeln!(
            metadata,
            "crate {}@{}",
            identity.name(),
            identity.disambiguator()
        )
        .map_err(|error| error.to_string())?;
        writeln!(metadata, "edition {edition}").map_err(|error| error.to_string())?;
        if let Some(imported) = report.resolution.crates.as_deref() {
            let dependencies = imported.dependencies();
            for artifact in dependencies.artifacts() {
                let identity = artifact.identity();
                writeln!(
                    metadata,
                    "dependency {}@{}",
                    identity.name(),
                    identity.disambiguator()
                )
                .map_err(|error| error.to_string())?;
            }
            for dependency in dependencies.aliases() {
                let identity = dependency.identity();
                writeln!(
                    metadata,
                    "extern {}{} {}@{}",
                    if dependency.in_prelude() {
                        ""
                    } else {
                        "noprelude:"
                    },
                    dependency.alias(),
                    identity.name(),
                    identity.disambiguator()
                )
                .map_err(|error| error.to_string())?;
            }
        }
        Ok(Some(ExecutableMetadata {
            path,
            bytes: metadata.into_bytes(),
        }))
    }

    pub fn context(&self, artifact: bool) -> Result<rustgm::resolve::bindings::Context, String> {
        let mode = self.bindings_mode()?;
        let artifacts = if !artifact {
            ArtifactContext::Inspection
        } else if self.source_profile
            == Some(rustgm::lower::instances::SourcePublication::BootstrapModuleBodies)
        {
            ArtifactContext::BootstrapLibrary
        } else if matches!(&mode, rustgm::support::bindings::Mode::Bootstrap) {
            ArtifactContext::Ordinary
        } else {
            ArtifactContext::Source {
                selected: self.output,
            }
        };
        Ok(rustgm::resolve::bindings::Context {
            crate_name: self.name.clone(),
            current: self.identity()?,
            dependencies: self.dependencies(&mode)?,
            mode,
            artifacts,
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
    fn dependencies(
        &self,
        mode: &rustgm::support::bindings::Mode,
    ) -> Result<Option<Arc<Dependencies>>, String> {
        if self.dependency_count() == 0 {
            return Ok(None);
        }
        // Source graphs can contain the complete pinned standard-library
        // metadata. Use the shared artifact ceiling for their aggregate input.
        let (limit, overflow) = if matches!(mode, rustgm::support::bindings::Mode::Bootstrap) {
            (16_000_000, "crate artifacts exceed 16 MB input budget")
        } else {
            (
                crates::MAX_ARTIFACT_BYTES,
                "crate artifacts exceed source metadata input budget",
            )
        };
        let mut paths = HashMap::default();
        let mut artifacts = Vec::new();
        let mut aliases = Vec::new();
        let mut total = 0usize;
        let named = self.externs.iter().map(|argument| {
            argument
                .split_once('=')
                .map(|(alias, path)| (Some(alias), path))
                .ok_or("--extern requires [noprelude:]ALIAS=METADATA")
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
                let remaining = limit.saturating_sub(total);
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
                    return Err(overflow.into());
                }
                total += bytes.len();
                if bytes.starts_with(BINARY_METADATA_MAGIC.as_bytes()) {
                    return Err("executable metadata cannot serve as a dependency".into());
                }
                let artifact = crates::decode(&bytes).map_err(str::to_owned)?;
                let identity = artifact.identity().clone();
                paths.insert(path, identity.clone());
                artifacts.push(artifact);
                identity
            };
            if let Some(alias) = alias {
                let (alias, in_prelude) = alias
                    .strip_prefix("noprelude:")
                    .map_or((alias, true), |alias| (alias, false));
                aliases.push(
                    Dependency::with_prelude(alias.to_owned(), identity, in_prelude)
                        .map_err(str::to_owned)?,
                );
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
