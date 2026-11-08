//! `#![compiler_builtins]` is published as bit 0 of a crate loader flags byte
//! that leads only the outermost artifact, and a `no_std` source client
//! requires such a crate in its graph after name resolution, as rustc 1.98's
//! crate loader injects one.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        load::{InputFile, Sources},
    },
    support::{
        bindings::Mode,
        crates::{
            self, CrateIdentity, Dependencies, Dependency, Interface, LoaderFlags, exported_macros,
        },
        source::SourceFile,
    },
};
use std::{path::Path, sync::Arc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const BUILTINS: &str = "#![feature(compiler_builtins)]#![compiler_builtins]#![no_std]";
const CLIENT: &str = "#![no_std]pub fn answer()->u32{match Option::Some(core::base()){Option::Some(_)=>forty_two!(),Option::None=>0}}";

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}

fn input(source: &str) -> Result<InputFile> {
    Ok(InputFile {
        identity: "lib.rs".into(),
        path: "lib.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    })
}
/// The `no_std` controls' source-core provider, named `core`.
fn core() -> Result<Interface> {
    let identity = CrateIdentity::new("core", "loader_flags")?;
    let context = Context {
        crate_name: Some("core".into()),
        current: Some(identity.clone()),
        mode: Mode::Source {
            provider: identity.clone(),
        },
        ..Default::default()
    };
    let source = crate::test_source::fixture("pa29/tests/spec/no-std-clients/provider.rs")?;
    let expanded = resolve::expansion::analyze_with_context(
        input(&source)?,
        &mut NoFiles,
        &Config::default(),
        &context,
    )?;
    let report = resolve::analyze_expanded(expanded)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let (_, interface) = crate::lower::instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        crate::lower::instances::SourcePublication::ModuleBodies,
    )?;
    Ok(copy(&interface)?)
}
/// Resolve a source client of `core` named `name`, with `artifacts` as its
/// inventory and `aliases` as its `--extern` entries.
fn resolved(
    core: &Interface,
    name: &str,
    artifacts: &[&Interface],
    aliases: &[(&str, &Interface)],
    source: &str,
) -> std::result::Result<resolve::expansion::Report, String> {
    let identity = CrateIdentity::new(name, "loader_flags")?;
    let mut inventory = vec![copy(core)?];
    for artifact in artifacts {
        inventory.push(copy(artifact)?);
    }
    let mut entries = Vec::new();
    for (alias, artifact) in aliases {
        entries.push(Dependency::new(
            (*alias).into(),
            artifact.identity().clone(),
        )?);
    }
    let context = Context {
        crate_name: Some(name.into()),
        current: Some(identity),
        dependencies: Some(Arc::new(Dependencies::new(inventory, entries)?)),
        mode: Mode::SourceClient {
            provider: core.identity().clone(),
        },
    };
    let input = input(source).map_err(|error| error.to_string())?;
    let expanded =
        resolve::expansion::analyze_with_context(input, &mut NoFiles, &Config::default(), &context)
            .map_err(|error| error.to_string())?;
    resolve::analyze_expanded(expanded).map_err(|error| error.to_string())
}
/// Publish a resolved source client as a library.
fn published(report: &resolve::expansion::Report, name: &str) -> Result<Interface> {
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(report)?;
    let (_, interface) = crate::lower::instances::build_library_prepared(
        report,
        &mut facts,
        CrateIdentity::new(name, "loader_flags")?,
        prepared,
    )?;
    Ok(copy(&interface)?)
}
/// An independent decoded copy; interfaces are not `Clone`.
fn copy(interface: &Interface) -> std::result::Result<Interface, &'static str> {
    crates::decode(&crates::encode(interface)?)
}
fn rejected(bytes: &[u8], message: &str) -> Result {
    let error = crates::decode(bytes)
        .err()
        .ok_or_else(|| format!("forged loader flags were accepted: {message}"))?;
    assert!(error.contains(message), "{error} != {message}");
    Ok(())
}
fn failure(
    result: std::result::Result<resolve::expansion::Report, String>,
    message: &str,
) -> Result {
    let error = result.err().ok_or_else(|| format!("accepted: {message}"))?;
    assert!(error.contains(message), "{error} != {message}");
    Ok(())
}

#[test]
fn loader_flags_lead_only_the_outermost_artifact_and_keep_every_other_byte() -> Result {
    let owner = CrateIdentity::new("builtins", "loader_flags")?;
    let dependency = CrateIdentity::new("core", "loader_flags")?;
    let reexport = exported_macros::ReexportMacro::new(
        Vec::new(),
        "here".into(),
        1,
        Vec::new(),
        "there".into(),
    )?;
    let plain = || -> Result<Interface> {
        Ok(
            Interface::new(owner.clone(), vec![dependency.clone()], Vec::new())?.with_macros(
                exported_macros::Macros::with_reexports(
                    Vec::new(),
                    Vec::new(),
                    vec![reexport.clone()],
                )?,
            )?,
        )
    };
    let unflagged = crates::encode(&plain()?)?;
    let flagged = plain()?.with_loader_flags(LoaderFlags::COMPILER_BUILTINS)?;
    let bytes = crates::encode(&flagged)?;
    // `[15][1]` is spliced in at byte 12; the rest is the unflagged artifact.
    assert_eq!(bytes[12..14], [15, 1]);
    assert_eq!([&bytes[..12], &bytes[14..]].concat(), unflagged);
    let decoded = crates::decode(&bytes)?;
    assert!(decoded.loader_flags().compiler_builtins());
    assert_eq!(decoded, flagged);
    assert_eq!(crates::encode(&decoded)?, bytes);
    assert!(crates::decode(&unflagged)?.loader_flags().is_empty());
    for (flags, message) in [
        (2, "unknown crate loader flag"),
        (0x81, "unknown crate loader flag"),
        (0, "crate loader flags envelope requires a flag"),
    ] {
        let mut forged = bytes.clone();
        forged[13] = flags;
        rejected(&forged, message)?;
    }
    let repeated = [&bytes[..14], &bytes[12..]].concat();
    rejected(&repeated, "repeated crate loader flags")?;
    // An artifact wrapped by the macro envelope carries no flags of its own.
    let inner = unflagged
        .windows(8)
        .skip(8)
        .position(|window| window == b"RGMETA\r\n")
        .ok_or("missing wrapped artifact")?
        + 8;
    let length = u32::from_le_bytes(unflagged[inner - 4..inner].try_into()?);
    let mut nested = unflagged[..inner - 4].to_vec();
    nested.extend((length + 2).to_le_bytes());
    nested.extend(&unflagged[inner..inner + 12]);
    nested.extend([15, 1]);
    nested.extend(&unflagged[inner + 12..]);
    rejected(
        &nested,
        "crate loader flags belong to the outermost artifact",
    )?;
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    assert!(
        plain()?
            .with_loader_flags(LoaderFlags::default())?
            .loader_flags()
            .is_empty()
    );
    assert!(
        flagged
            .with_loader_flags(LoaderFlags::COMPILER_BUILTINS)
            .is_err()
    );
    assert!(LoaderFlags::from_bits(1)?.compiler_builtins());
    Ok(())
}

#[test]
fn no_std_clients_need_a_compiler_builtins_crate_in_their_graph() -> Result {
    let core = core()?;
    assert!(core.loader_flags().is_empty());
    // A `#![compiler_builtins]` crate needs no injection and publishes bit 0.
    let builtins = published(
        &resolved(&core, "compiler_builtins", &[], &[], BUILTINS)?,
        "compiler_builtins",
    )?;
    assert!(builtins.loader_flags().compiler_builtins());
    assert_eq!(crates::encode(&builtins)?[12..14], [15, 1]);
    // A no_std client without one fails as rustc's E0463 does.
    failure(
        resolved(&core, "client", &[], &[], CLIENT),
        "can't find crate for `compiler_builtins`",
    )?;
    // One in the inventory suffices, also when `--extern` names it.
    let client = published(
        &resolved(&core, "client", &[&builtins], &[], CLIENT)?,
        "client",
    )?;
    assert!(client.loader_flags().is_empty());
    assert_ne!(crates::encode(&client)?[12], 15);
    resolved(
        &core,
        "client",
        &[&builtins],
        &[("compiler_builtins", &builtins)],
        CLIENT,
    )?;
    // A no_core client is never injected.
    resolved(
        &core,
        "client",
        &[],
        &[],
        "#![feature(no_core)]#![no_core]pub fn zero()->u32{0}",
    )?;
    // The crate the name resolves to must be `#![compiler_builtins]`.
    let plain = published(
        &resolved(
            &core,
            "compiler_builtins",
            &[],
            &[],
            "#![feature(no_core)]#![no_core]",
        )?,
        "compiler_builtins",
    )?;
    assert!(plain.loader_flags().is_empty());
    failure(
        resolved(
            &core,
            "client",
            &[&plain],
            &[("compiler_builtins", &plain)],
            CLIENT,
        ),
        "the crate resolved as `compiler_builtins` is not `#![compiler_builtins]`",
    )?;
    Ok(())
}

#[test]
fn the_compiler_builtins_attribute_is_one_gated_crate_root_word() -> Result {
    let core = core()?;
    for (source, message) in [
        (
            "#![compiler_builtins]#![no_std]",
            "the `compiler_builtins` attribute requires its internal feature",
        ),
        (
            "#![feature(compiler_builtins)]#![compiler_builtins]#![compiler_builtins]#![no_std]",
            "multiple `compiler_builtins` attributes",
        ),
        (
            "#![feature(compiler_builtins)]#![compiler_builtins = \"x\"]#![no_std]",
            "malformed `compiler_builtins` attribute input",
        ),
        (
            "#![feature(compiler_builtins)]#![no_std]mod m{#![compiler_builtins]}",
            "the `#![compiler_builtins]` attribute can only be used at the crate root",
        ),
    ] {
        failure(
            resolved(&core, "compiler_builtins", &[], &[], source),
            message,
        )?;
    }
    Ok(())
}
