//! A shim's `pub use base::*` publishes one glob row whose size does not
//! depend on its target, and a consumer binds the target's public names once
//! per row, charging each visited binding once. Explicit shim bindings shadow
//! glob names, two glob targets make a name ambiguous where it is used, and
//! forged rows are rejected by name.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        load::{InputFile, Sources},
        model::Namespace,
    },
    source_packets,
    support::{
        bindings::Mode,
        crates::{self, CrateIdentity, Dependencies, Dependency, Interface, exported_macros},
        source::SourceFile,
    },
};
use std::{fmt::Write as _, path::Path, sync::Arc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const NO_CORE: &str = "#![feature(no_core)]#![no_core]";
const SPLIT: &str = "pub mod left{pub fn shared(value:u8)->u8{value}pub fn only_left(value:u8)->u8{value}}pub mod right{pub fn shared(value:u8)->u8{value}}mod sealed{pub fn hidden(value:u8)->u8{value}}pub(crate) fn internal(value:u8)->u8{value}";

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}

fn provider(extra: &str) -> Result<Interface> {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    source.push_str(extra);
    crate::module_packets::exported_profile(
        &source,
        crate::lower::instances::SourcePublication::ModuleBodies,
        "shim_expansion",
    )
}
/// A source client of `provider` published as a library; it names the
/// provider `base`.
fn shim(provider: &Interface, body: &str) -> Result<Interface> {
    let (
        report,
        crate::pipeline::Analysis {
            mut facts,
            prepared,
        },
        current,
    ) = source_packets::client_analysis(copy(provider)?, &format!("{NO_CORE}{body}"))?;
    let (_, interface) =
        crate::lower::instances::build_library_prepared(&report, &mut facts, current, prepared)?;
    Ok(crates::decode(&crates::encode(&interface)?)?)
}
/// Resolve a client that names `shim` by `extern crate shim` and reaches
/// the provider only through the dependency inventory.
fn client(
    provider: &Interface,
    shim: &Interface,
    body: &str,
) -> std::result::Result<resolve::expansion::Report, String> {
    let identity = CrateIdentity::new("glob_client", "shim_expansion")?;
    let dependencies = Dependencies::new(
        vec![copy(provider)?, copy(shim)?],
        vec![Dependency::new("shim".into(), shim.identity().clone())?],
    )?;
    let context = Context {
        crate_name: Some("glob_client".into()),
        current: Some(identity),
        dependencies: Some(Arc::new(dependencies)),
        mode: Mode::SourceClient {
            provider: provider.identity().clone(),
        },
    };
    let input = InputFile {
        identity: "client.rs".into(),
        path: "client.rs".into(),
        source: SourceFile::new(format!("{NO_CORE}extern crate shim;{body}").into_bytes())?,
    };
    let expanded =
        resolve::expansion::analyze_with_context(input, &mut NoFiles, &Config::default(), &context)
            .map_err(|error| error.to_string())?;
    resolve::analyze_expanded(expanded).map_err(|error| error.to_string())
}
/// An independent decoded copy; interfaces are not `Clone`.
fn copy(interface: &Interface) -> std::result::Result<Interface, &'static str> {
    crates::decode(&crates::encode(interface)?)
}
fn steps(report: &resolve::expansion::Report) -> Result<usize> {
    Ok(report
        .resolution
        .crates
        .as_deref()
        .ok_or("imported crates")?
        .glob_expansion_steps())
}
/// One glob row of a shim's table: (placement, owner, target).
type GlobRow = (Vec<String>, u32, Vec<String>);
fn glob_rows(shim: &Interface) -> Result<Vec<GlobRow>> {
    Ok(shim
        .macros()
        .ok_or("missing macro table")?
        .globs()
        .iter()
        .map(|row| (row.placement().to_vec(), row.owner(), row.target().to_vec()))
        .collect())
}

#[test]
fn glob_rows_stay_fixed_and_expansion_work_grows_linearly_with_the_target() -> Result {
    let (mut encoded, mut work) = (Vec::new(), Vec::new());
    for width in [6usize, 24, 96] {
        let mut functions = String::new();
        for index in 0..width {
            write!(functions, "pub fn f{index:03}(value:u8)->u8{{value}}")?;
        }
        let provider = provider(&functions)?;
        let shim = shim(&provider, "pub use base::*;")?;
        assert_eq!(glob_rows(&shim)?, [(Vec::new(), 1, Vec::new())]);
        let bytes = crates::encode(&shim)?;
        assert_eq!(bytes[12..14], [14, 5]);
        encoded.push(bytes);
        let last = width - 1;
        let report = client(
            &provider,
            &shim,
            &format!("pub fn f()->u8{{shim::f{last:03}(1)}}"),
        )?;
        work.push(steps(&report)?);
    }
    // The shim's table names its target and never copies its names.
    assert!(encoded.windows(2).all(|pair| pair[0] == pair[1]));
    // One unit per row and per visited target binding: 18 and 72 more names.
    let [small, middle, large] = <[usize; 3]>::try_from(work.as_slice())?;
    assert_eq!((middle - small, large - middle), (18, 72), "{work:?}");
    Ok(())
}

#[test]
fn explicit_shim_names_shadow_and_two_glob_targets_are_ambiguous_only_when_used() -> Result {
    let provider = provider(SPLIT)?;
    let ambiguous = shim(
        &provider,
        "pub use base::*;pub use base::left::*;pub use base::right::*;",
    )?;
    assert_eq!(
        glob_rows(&ambiguous)?,
        [
            (Vec::new(), 1, Vec::new()),
            (Vec::new(), 1, vec!["left".to_owned()]),
            (Vec::new(), 1, vec!["right".to_owned()]),
        ]
    );
    let report = client(
        &provider,
        &ambiguous,
        "pub fn f()->u8{shim::only_left(1)}pub fn g()->u8{shim::right::shared(1)}",
    )?;
    let imported = report
        .resolution
        .crates
        .as_deref()
        .ok_or("imported crates")?;
    let root = imported
        .definition(&crates::ExternalDef::new(
            ambiguous.identity().clone(),
            Vec::new(),
            crates::ExternalKind::Module,
        )?)
        .ok_or("shim root")?;
    let scope = report.resolution.definitions[root.0 as usize]
        .members
        .ok_or("shim root scope")?;
    let binding = |name: &str, namespace| {
        report
            .resolution
            .bindings
            .scope(scope)
            .find(|&(id, found, _)| found == namespace && report.package.names.get(id) == name)
            .map(|(_, _, binding)| *binding)
    };
    assert!(binding("shared", Namespace::Value).is_some_and(|b| b.ambiguous));
    assert!(binding("only_left", Namespace::Value).is_some_and(|b| !b.ambiguous));
    // Only names visible outside the provider travel.
    assert!(binding("internal", Namespace::Value).is_none());
    assert!(binding("sealed", Namespace::Type).is_none());
    let used = client(&provider, &ambiguous, "pub fn f()->u8{shim::shared(1)}")
        .err()
        .ok_or("a used glob ambiguity was accepted")?;
    assert!(used.contains("ambiguous value name `shared`"), "{used}");
    // The shim's own `left` shadows the glob's.
    let shadowed = shim(&provider, "pub use base::*;pub mod left{}")?;
    assert_eq!(glob_rows(&shadowed)?, [(Vec::new(), 1, Vec::new())]);
    let error = client(
        &provider,
        &shadowed,
        "pub fn f()->u8{shim::left::shared(1)}",
    )
    .err()
    .ok_or("a glob name replaced an explicit shim module")?;
    assert!(error.contains("unresolved value name"), "{error}");
    let plain = shim(&provider, "pub use base::*;")?;
    client(&provider, &plain, "pub fn f()->u8{shim::left::shared(1)}")?;
    Ok(())
}

/// The bytes with the unique run `from` replaced by `to`.
fn respelled(bytes: &[u8], from: &[u8], to: &[u8]) -> Result<Vec<u8>> {
    let mut found = bytes
        .windows(from.len())
        .enumerate()
        .filter(|(_, w)| *w == from);
    let (at, _) = found.next().ok_or("forged bytes not found")?;
    if found.next().is_some() {
        return Err("forged bytes are not unique".into());
    }
    let mut copy = bytes.to_vec();
    copy.splice(at..at + from.len(), to.iter().copied());
    Ok(copy)
}
fn rejected(bytes: &[u8], message: &str) -> Result {
    let error = crates::decode(bytes)
        .err()
        .ok_or_else(|| format!("forged glob row was accepted: {message}"))?;
    assert!(error.contains(message), "{error} != {message}");
    Ok(())
}

#[test]
fn forged_glob_rows_and_relabelled_tables_are_rejected_by_name() -> Result {
    let provider = provider(SPLIT)?;
    let shim = shim(&provider, "pub use base::*;")?;
    let bytes = crates::encode(&shim)?;
    // One row: kind 4, the root placement, owner 1 and the provider's root.
    let row: &[u8] = b"\x01\x00\x00\x00\x04\x00\x00\x01\x00\x00\x00\x00\x00";
    let owner = |ordinal: u8| [&row[..7], &[ordinal], &row[8..]].concat();
    rejected(
        &respelled(&bytes, row, &owner(0))?,
        "exported glob re-export requires a dependency owner",
    )?;
    rejected(
        &respelled(&bytes, row, &owner(2))?,
        "exported glob re-export names an unknown owner",
    )?;
    // A version-four reader has no kind 4; a version-six table is unknown.
    let mut relabelled = bytes.clone();
    relabelled[13] = 4;
    rejected(&relabelled, "unknown exported macro row kind")?;
    relabelled[13] = 6;
    rejected(&relabelled, "unknown exported macro metadata version")?;
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    // A version-five label on a table without glob rows.
    let inner = bytes
        .windows(8)
        .skip(8)
        .position(|window| window == b"RGMETA\r\n")
        .ok_or("missing wrapped artifact")?;
    let inner = crates::decode(&bytes[inner + 8..])?;
    let reexport = exported_macros::ReexportMacro::new(
        Vec::new(),
        "here".into(),
        1,
        Vec::new(),
        "there".into(),
    )?;
    let plain = inner.with_macros(exported_macros::Macros::with_reexports(
        Vec::new(),
        Vec::new(),
        vec![reexport],
    )?)?;
    let mut five = crates::encode(&plain)?;
    assert_eq!(five[12..14], [14, 2]);
    five[13] = 5;
    rejected(
        &five,
        "version-five exported macro table has no glob re-export row",
    )?;
    // Targets and placements the owner or shim does not export decode, then
    // fail in the consumer by name.
    let target = |path: &[u8]| [&row[..11], path].concat();
    for (forged, message) in [
        (
            target(b"\x01\x00\x06\x00\x00\x00sealed"),
            "glob re-export target is not a public module of its owner",
        ),
        (
            target(b"\x01\x00\x07\x00\x00\x00missing"),
            "glob re-export target is not a public module of its owner",
        ),
        (
            [
                &row[..5],
                b"\x01\x00\x04\x00\x00\x00left".as_slice(),
                &row[7..],
            ]
            .concat(),
            "glob re-export placement has no public imported module",
        ),
    ] {
        let forged = crates::decode(&respelled(&bytes, row, &forged)?)?;
        let error = client(&provider, &forged, "pub fn f(value:u8)->u8{value}")
            .err()
            .ok_or("a forged glob target was accepted")?;
        assert!(error.contains(message), "{error} != {message}");
    }
    Ok(())
}
