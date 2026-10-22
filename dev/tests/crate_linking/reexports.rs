//! Public source bindings retain defining crate identities across real artifacts.
use super::{artifacts, crates, exports, imports};
use crates::{CrateIdentity, Dependencies, Dependency, ExternalDef, ExternalKind};
use rustgm::resolve::{
    expansion::Report,
    model::{DefId, Namespace, ScopeId},
};
use std::{
    fmt::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn qualify(native: bool) -> Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root.join(format!(
        "obj/parallel/pa29-public-names/maintainer-{}-{native}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let mut command = Command::new("python3");
    command
        .current_dir(root)
        .arg(root.join("scripts/check_public_names.py"))
        .arg("--app")
        .arg(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--evidence-root")
        .arg(&folder);
    if native {
        command.arg("--native");
    }
    artifacts::checked("harness", &folder, &mut command)?;
    Ok(folder)
}

fn module(report: &Report, owner: &CrateIdentity, path: &[&str]) -> Result<ScopeId> {
    let external = ExternalDef::new(
        owner.clone(),
        path.iter().map(|p| (*p).into()).collect(),
        ExternalKind::Module,
    )?;
    let definition = report
        .resolution
        .crates
        .as_deref()
        .ok_or("missing imports")?
        .definition(&external)
        .ok_or("missing module identity")?;
    Ok(report.resolution.definitions[definition.0 as usize]
        .members
        .ok_or("missing module scope")?)
}

fn binding(report: &Report, scope: ScopeId, name: &str, namespace: Namespace) -> Result<DefId> {
    let (_, _, binding) = report
        .resolution
        .bindings
        .scope(scope)
        .find(|(id, ns, _)| *ns == namespace && report.package.names.get(*id) == name)
        .ok_or("missing public spelling")?;
    Ok(binding.target)
}

#[test]
fn reexports_share_definition_tables_and_preserve_hidden_paths() -> Result {
    let folder = qualify(false)?;
    let first = crates::decode(&std::fs::read(folder.join("first.rgm"))?)?;
    let second = crates::decode(&std::fs::read(folder.join("second.rgm"))?)?;
    let facade = crates::decode(&std::fs::read(folder.join("facade.rgm"))?)?;
    for artifact in [&first, &second] {
        assert_eq!(crates::encode(artifact)?[12], 6);
        assert_eq!(artifact.functions().len(), 2);
        assert_eq!(artifact.records().len(), 1);
        assert_eq!(artifact.constants().len(), 1);
        assert_eq!(
            artifact.storage().ok_or("missing storage")?.statics().len(),
            1
        );
        assert_eq!(
            artifact.records()[0].definition().path(),
            ["implementation", "Handle"]
        );
        assert!(artifact.records()[0].public());
        assert_eq!(
            artifact
                .public_names()
                .ok_or("missing public names")?
                .modules()
                .len(),
            3
        );
    }
    assert!(facade.functions().is_empty());
    assert!(facade.records().is_empty());
    assert!(facade.constants().is_empty());
    assert!(facade.storage().is_none());
    let owners = [
        first.identity().clone(),
        second.identity().clone(),
        facade.identity().clone(),
    ];
    let aliases = ["one", "two", "facade"]
        .into_iter()
        .zip(&owners)
        .map(|(name, identity)| Dependency::new(name.into(), identity.clone()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let dependencies = Arc::new(Dependencies::new(vec![facade, second, first], aliases)?);
    let report = imports::resolve("fn main(){let _=facade::alias();}", dependencies)?;
    let imported = report
        .resolution
        .crates
        .as_deref()
        .ok_or("missing imports")?;
    assert_eq!(imported.functions().count(), 4);
    assert_eq!(imported.records().count(), 2);
    assert_eq!(imported.constants().count(), 2);
    assert_eq!(imported.statics().count(), 2);
    let original = module(&report, &owners[0], &[])?;
    let exported = module(&report, &owners[2], &[])?;
    for (name, namespace) in [
        ("make", Namespace::Value),
        ("DATA", Namespace::Value),
        ("Handle", Namespace::Type),
    ] {
        assert_eq!(
            binding(&report, original, name, namespace)?,
            binding(&report, exported, name, namespace)?
        );
    }
    assert_eq!(
        binding(&report, original, "make", Namespace::Value)?,
        binding(&report, exported, "alias", Namespace::Value)?
    );
    assert_ne!(
        binding(&report, exported, "Handle", Namespace::Type)?,
        binding(&report, exported, "Other", Namespace::Type)?
    );
    let surface = module(&report, &owners[0], &["surface"])?;
    let again = binding(&report, surface, "again", Namespace::Type)?;
    assert_eq!(
        report.resolution.definitions[again.0 as usize].members,
        Some(surface)
    );
    assert!(binding(&report, original, "implementation", Namespace::Type).is_err());
    rustgm::pipeline::analyze_source(&report)?;
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM object backend and pinned Rust toolchain"]
fn public_function_and_storage_aliases_keep_native_address_identity() -> Result {
    qualify(true).map(|_| ())
}

#[test]
fn module_alias_graphs_share_tables_and_export_deterministically() -> Result {
    for width in [8, 32, 128] {
        let mut source = String::from(
            "mod hidden{pub fn run()->u8{7}}pub mod table{pub use super::hidden::run;pub use super::table as again;}",
        );
        for index in 0..width {
            write!(source, "pub use table as alias{index};")?;
        }
        crate::qualify::qualify(
            &format!("parallel/pa29-public-names/aliases-{width}"),
            &source,
            true,
        )?;
        let (_, interface) = exports::compile(&source, "alias_graph")?;
        let names = interface.public_names().ok_or("missing names")?;
        assert_eq!(names.modules().len(), 2);
        assert_eq!(names.modules()[0].bindings().len(), width + 1);
        assert_eq!(interface.functions().len(), 1);
        let first = crates::encode(&interface)?;
        assert!(first.len() <= 128 * width + 1024);
        let (_, repeated) = exports::compile(&source, "alias_graph")?;
        assert_eq!(first, crates::encode(&repeated)?);
        assert_eq!(crates::decode(&first)?, interface);
    }
    Ok(())
}

#[test]
fn empty_modules_and_type_value_homonyms_survive_publication() -> Result {
    for (index, source) in [
        "pub mod empty{}",
        "mod hidden{pub fn f(n:u32)->u32{n}}pub use hidden::f;",
    ]
    .into_iter()
    .enumerate()
    {
        crate::qualify::qualify(
            &format!("parallel/pa29-public-names/former-boundary-{index}"),
            source,
            true,
        )?;
        let (_, interface) = exports::compile(source, "former_boundary")?;
        assert!(interface.public_names().is_some());
    }
    let source =
        "mod hidden{pub struct Same{pub n:u8}pub fn Same()->Same{Same{n:7}}}pub use hidden::Same;";
    crate::qualify::qualify("parallel/pa29-public-names/namespaces", source, true)?;
    let (_, interface) = exports::compile(source, "namespaces")?;
    let names = interface.public_names().ok_or("missing names")?;
    assert_eq!(names.modules()[0].bindings().len(), 2);
    assert_eq!(interface.functions().len(), 1);
    assert_eq!(interface.records().len(), 1);
    assert_eq!(crates::decode(&crates::encode(&interface)?)?, interface);
    Ok(())
}
