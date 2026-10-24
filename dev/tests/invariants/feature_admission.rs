//! Counters distinguish live configured syntax from templates and retired trees.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        bindings::Context,
        expansion,
        load::{InputFile, Loader, Sources},
    },
    support::{bindings::Mode, crates::CrateIdentity, hash::HashSet, source::SourceFile},
};
use std::{fmt::Write, path::Path};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const HEADER: &str = "#![feature(no_core,lang_items,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}#[lang=\"sized\"]trait Sized:MetaSized{}";
const INCLUDE: &str = "#[rustc_builtin_macro]macro_rules!include{($($t:tt)*)=>{}}";
struct Files;
impl Sources for Files {
    fn read(&mut self, path: &Path) -> std::result::Result<Option<InputFile>, String> {
        if path.extension().is_some_and(|extension| extension == "rs")
            && path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| {
                    stem.strip_prefix("items")
                        .is_some_and(|suffix| suffix.bytes().all(|b| b.is_ascii_digit()))
                })
        {
            Ok(Some(input(path, "type N=!;")?))
        } else if path == Path::new("unit.rs") {
            Ok(Some(input(path, "#[allow(unused)]const X:u8=1;")?))
        } else {
            Ok(None)
        }
    }
}
fn input(path: &Path, source: &str) -> std::result::Result<InputFile, String> {
    Ok(InputFile {
        identity: path.into(),
        path: path.into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    })
}
fn context() -> Result<Context> {
    let identity = CrateIdentity::new("minicore", "features_v1")?;
    Ok(Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    })
}
fn original(source: &str) -> Result<Package> {
    Ok(Loader::with_context(
        input(Path::new("lib.rs"), source)?,
        &mut Files,
        &Config::default(),
        &context()?,
    )?
    .package)
}
fn expanded(source: &str) -> Result<Package> {
    Ok(expansion::analyze_with_context(
        input(Path::new("lib.rs"), source)?,
        &mut Files,
        &Config::default(),
        &context()?,
    )?
    .package)
}
fn counted(package: &Package) -> Result<Work> {
    let mut work = Work::default();
    inspect(package, &mut work)?;
    Ok(work)
}
fn unique_nodes(package: &Package) -> usize {
    let mut seen = HashSet::default();
    let mut pending = Vec::new();
    let mut children = Vec::new();
    for (file, unit) in package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        pending.push(unit.configured.ast.root);
        while let Some(node) = pending.pop() {
            if !unit.configured.active[node.get() as usize] {
                continue;
            }
            if !seen.insert((file, node)) {
                continue;
            }
            children.clear();
            unit.configured.ast.push_children(node, &mut children);
            pending.append(&mut children);
        }
    }
    seen.len()
}

#[test]
fn ordinary_admission_inspects_no_syntax_and_language_requests_stay_packed() -> Result {
    let mut package = original(HEADER)?;
    package.source_core = None;
    // The empty ordinary path must not even demand a valid loaded root.
    package.units.clear();
    assert_eq!(counted(&package)?, Work::default());
    // Source auto-trait and negative-impl gates extend the packed request bits.
    assert_eq!(size_of::<super::super::policy::Features>(), 2);
    assert_eq!(size_of::<Position>(), 1);
    // Cold library-policy and token-mark pointers extend the 240-byte base.
    assert_eq!(
        size_of::<Package>(),
        240 + size_of::<Option<Box<crate::resolve::stability::Library>>>()
            + size_of::<crate::expand::marks::Marks>()
    );
    assert_eq!(size_of::<super::super::SourceCore>(), 72);
    Ok(())
}

#[test]
fn width_depth_and_expansion_counts_visit_each_live_node_once() -> Result {
    for n in [8, 32, 128] {
        let mut width = HEADER.to_owned();
        for index in 0..n {
            write!(width, "#[allow(unused)]const N{index}:u8=1;")?;
        }
        let package = original(&width)?;
        let work = counted(&package)?;
        assert_eq!(work.nodes, unique_nodes(&package));
        assert!(work.nodes < 12 * n + 64, "width={n}: {work:?}");
        assert_eq!(work.units, 1);
        // A source deeper than this axis reaches the existing parser limit,
        // before feature admission; retain that boundary below.
        let levels = n.min(64);
        let depth = format!(
            "{HEADER}{}const X:u8=1;{}",
            "mod m{".repeat(levels),
            "}".repeat(levels)
        );
        let package = original(&depth)?;
        let work = counted(&package)?;
        assert_eq!(work.nodes, unique_nodes(&package));
        assert!(work.nodes < 5 * levels + 64, "depth={levels}: {work:?}");
        eprintln!("feature admission depth={levels}: {work:?}");
        let mut source =
            format!("{HEADER}macro_rules!make{{($name:ident)=>{{const $name:u8=1;}}}}");
        for i in 0..n {
            write!(source, "make!(N{i});")?;
        }
        let package = expanded(&source)?;
        let work = counted(&package)?;
        assert_eq!(work.nodes, unique_nodes(&package));
        assert!(work.nodes < 12 * n + 64, "expansions={n}: {work:?}");
        assert_eq!(work.units, 1);
        eprintln!("feature admission expansions={n}: {work:?}");
    }
    let beyond = format!(
        "{HEADER}{}const X:u8=1;{}",
        "mod m{".repeat(128),
        "}".repeat(128)
    );
    let error = original(&beyond)
        .err()
        .ok_or("missing parser depth boundary")?;
    assert!(
        error
            .to_string()
            .contains("crate syntax nesting exceeds 128")
    );
    Ok(())
}

#[test]
fn include_templates_and_separate_module_instances_are_not_replayed() -> Result {
    for n in [8, 32, 128] {
        let mut source = format!("#![feature(never_type)]{HEADER}{INCLUDE}");
        for i in 0..n {
            write!(source, "mod m{i}{{include!(\"items.rs\");}}")?;
        }
        let package = expanded(&source)?;
        let work = counted(&package)?;
        assert_eq!(work.nodes, unique_nodes(&package));
        assert_eq!(work.units, 1);
        assert_eq!(work.templates, 1);
        assert!(package.units.iter().any(|u| u.include_context.is_some()));
        assert!(work.nodes < 6 * n + 64, "includes={n}: {work:?}");
        let mut source = format!("#![feature(never_type)]{HEADER}{INCLUDE}");
        for i in 0..n {
            write!(source, "mod m{i}{{include!(\"items{i}.rs\");}}")?;
        }
        let package = expanded(&source)?;
        let work = counted(&package)?;
        assert_eq!(work.nodes, unique_nodes(&package));
        assert_eq!(work.units, 1);
        assert_eq!(work.templates, n);
        assert!(work.nodes < 6 * n + 64, "distinct includes={n}: {work:?}");
        let mut source = HEADER.to_owned();
        for i in 0..n {
            write!(source, "#[path=\"unit.rs\"]mod m{i};")?;
        }
        let package = expanded(&source)?;
        let work = counted(&package)?;
        assert_eq!(work.nodes, unique_nodes(&package));
        assert_eq!(work.units, n + 1);
        assert!(work.nodes < 16 * n + 64, "units={n}: {work:?}");
        eprintln!("feature admission units={n}: {work:?}");
    }
    Ok(())
}

#[test]
fn failing_uses_are_counted_and_cfg_disabled_owners_have_no_authority() -> Result {
    for tail in [
        "type N=!;",
        "const N:u8=#[allow(unused)]1;",
        "mod wrong{#![feature(never_type)]type N=!;}",
    ] {
        let package = original(&format!("{HEADER}{tail}"))?;
        let mut work = Work::default();
        assert!(inspect(&package, &mut work).is_err());
        assert!(work.nodes > 0 && work.nodes <= unique_nodes(&package));
        assert_eq!(work.units, 1);
    }
    let source =
        format!("{HEADER}#[cfg(any())]mod wrong{{#![feature(unknown_source_gate)]type N=!;}}");
    let package = original(&source)?;
    let work = counted(&package)?;
    assert_eq!(work.nodes, unique_nodes(&package));
    let package = original(&format!("#![feature(asm_experimental_arch)]{HEADER}"))?;
    let features = &package
        .source_core
        .as_deref()
        .ok_or("source owner")?
        .features;
    assert!(!features.has(Feature::NeverType));
    assert!(!features.has(Feature::ExpressionAttributes));
    Ok(())
}

#[test]
fn an_attribute_index_cannot_substitute_another_configured_owner() -> Result {
    let mut package = original(&format!("{HEADER}#[allow(unused)]const N:u8=1;"))?;
    let unit = &mut package.units[0];
    let index = unit
        .configured
        .owners
        .iter()
        .position(|owner| owner.node != unit.configured.ast.root)
        .ok_or("nonroot attribute owner")?;
    unit.configured.owners[index].node = unit.configured.ast.root;
    let mut work = Work::default();
    let error = inspect(&package, &mut work)
        .err()
        .ok_or("forged owner accepted")?;
    assert!(error.message.contains("different configured owner"));
    assert!(work.nodes > 0);
    Ok(())
}

#[test]
fn a_callable_bound_result_is_not_a_stable_function_result_position() -> Result {
    // This checks parent classification before callable name/binding proof.
    // It does not assert that this provider already supplies the Fn lang item.
    let package = original(&format!("{HEADER}fn f<F:Fn()->!>(){{}}"))?;
    let mut work = Work::default();
    let error = inspect(&package, &mut work)
        .err()
        .ok_or("ungated callable output")?;
    assert!(error.message.contains("never_type"));
    assert!(work.nodes > 0);
    Ok(())
}
