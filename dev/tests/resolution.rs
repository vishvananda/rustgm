use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
        model::{DefKind, Namespace, Target},
        package::Package,
    },
    support::source::SourceFile,
};
use std::path::{Path, PathBuf};
struct Empty;
impl Sources for Empty {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn package(text: &str) -> Result<Package, String> {
    resolve::load::load(
        InputFile {
            identity: PathBuf::from("source.rs"),
            source: SourceFile::new(text.as_bytes().to_vec())?,
        },
        &mut Empty,
        &Config::default(),
    )
    .map_err(|e| format!("{e:?}"))
}
#[test]
fn local_references_retain_the_definition_visible_at_their_position() -> Result<(), String> {
    let mut package = package("fn f(x:i32){let x=x;{let x=x;x;}x;}")?;
    let facts = resolve::analyze(&mut package).map_err(|e| format!("{e:?}"))?;
    let locals: Vec<_> = facts
        .definitions
        .iter()
        .enumerate()
        .filter(|(_, d)| matches!(d.kind, DefKind::Parameter | DefKind::Local))
        .map(|(i, _)| u32::try_from(i))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    assert_eq!(locals.len(), 3);
    let targets: Vec<_> = facts
        .references
        .iter()
        .filter(|r| r.namespace == Namespace::Value)
        .filter_map(|r| {
            if let Target::Definition(id) = r.target {
                Some(id.0)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(targets, vec![locals[0], locals[1], locals[2], locals[1]]);
    Ok(())
}
#[test]
fn namespace_and_glob_aliases_preserve_target_identity() -> Result<(), String> {
    let mut package = package(
        "mod a{pub struct S{x:i32}pub fn S(){}}mod b{pub use crate::a::*;}use b::*;fn f(x:S){S();}",
    )?;
    let facts = resolve::analyze(&mut package).map_err(|e| format!("{e:?}"))?;
    let name = package.names.intern("S")?;
    let root = &facts.scopes[facts.module_scopes[0].0 as usize];
    let ty = root.bindings[&(name, Namespace::Type)].target;
    let value = root.bindings[&(name, Namespace::Value)].target;
    assert_ne!(ty, value);
    assert_eq!(facts.definitions[ty.0 as usize].kind, DefKind::Struct);
    assert_eq!(facts.definitions[value.0 as usize].kind, DefKind::Function);
    for module in 1..3 {
        let scope = &facts.scopes[facts.module_scopes[module].0 as usize];
        assert_eq!(scope.bindings[&(name, Namespace::Type)].target, ty);
        assert_eq!(scope.bindings[&(name, Namespace::Value)].target, value);
    }
    Ok(())
}
#[test]
fn long_left_associative_chains_do_not_consume_the_call_stack() -> Result<(), String> {
    let text = format!("fn f(x:i32)->i32{{x{}}}", "+x".repeat(20_000));
    let mut package = package(&text)?;
    let facts = resolve::analyze(&mut package).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        facts
            .references
            .iter()
            .filter(|r| r.namespace == Namespace::Value)
            .count(),
        20_001
    );
    Ok(())
}
#[test]
fn random_hash_seeds_do_not_change_the_text_view() -> Result<(), String> {
    let text = "mod a{pub struct X;pub fn f(){}}mod b{pub use crate::a::*;}use b::*;fn g(){X;f();}";
    let mut expected = None;
    for _ in 0..10 {
        let mut package = package(text)?;
        let facts = resolve::analyze(&mut package).map_err(|e| format!("{e:?}"))?;
        let mut output = Vec::new();
        resolve::render::write(&package, &facts, &mut output).map_err(|e| e.to_string())?;
        if let Some(expected) = &expected {
            assert_eq!(expected, &output);
        } else {
            expected = Some(output);
        }
    }
    Ok(())
}
#[test]
fn unresolved_names_are_explicitly_pending_only_where_expansion_can_add_bindings()
-> Result<(), String> {
    let mut package = package("macro_rules! m{()=>{}}fn f(){m!();generated();}")?;
    let facts = resolve::analyze(&mut package).map_err(|e| format!("{e:?}"))?;
    assert_eq!(facts.expansions.len(), 1);
    assert!(
        facts
            .references
            .iter()
            .any(|r| matches!(r.target, Target::Expansion(_)))
    );
    Ok(())
}

#[test]
fn a_late_explicit_import_retargets_downstream_aliases_and_globs() -> Result<(), String> {
    for root in ["pub use a::X;", "pub use a::*;"] {
        let source = format!(
            "{root}mod a{{pub use crate::b::*;pub use crate::d::X;}}mod b{{pub struct X;}}mod d{{pub use crate::e::X;}}mod e{{pub struct X;}}"
        );
        let mut package = package(&source)?;
        let facts = resolve::analyze(&mut package).map_err(|e| format!("{e:?}"))?;
        let name = package.names.intern("X")?;
        let origin = package
            .modules
            .iter()
            .position(|m| package.names.get(m.name) == "e")
            .ok_or("missing module")?;
        let expected = &facts.scopes[facts.module_scopes[origin].0 as usize];
        let root = &facts.scopes[facts.module_scopes[0].0 as usize];
        for ns in [Namespace::Type, Namespace::Value] {
            assert_eq!(
                root.bindings[&(name, ns)].target,
                expected.bindings[&(name, ns)].target
            );
            assert!(!root.bindings[&(name, ns)].ambiguous);
        }
    }
    Ok(())
}
