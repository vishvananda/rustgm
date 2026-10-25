//! Free constant declarations stay real while their placeholder has no binding.
use super::*;
use crate::{
    expand::config::Config,
    pipeline,
    resolve::{
        self,
        load::InputFile,
        model::{DefId, DefKind, Namespace},
    },
    support::source::SourceFile,
};
use std::path::Path;
#[path = "free_constant_bodies.rs"]
mod evaluation;
#[path = "visibility_policy_cases.rs"]
mod levels;
#[path = "visibility_cause_proofs.rs"]
mod proofs;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str) -> Result<Report, Error> {
    let expanded = resolve::expansion::analyze(
        InputFile {
            identity: "unnamed.rs".into(),
            path: "unnamed.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec()).map_err(Error::unlocated)?,
        },
        &mut NoFiles,
        &Config::default(),
    )?;
    resolve::analyze_expanded(expanded)
}
fn fixture(stage: u8, name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa{stage}/tests/spec/unnamed-constants/{name}.rs"
    ))?)
}
fn constants(report: &Report) -> Vec<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .filter_map(|(index, definition)| {
            if definition.kind == DefKind::Const && report.package.names.get(definition.name) == "_"
            {
                Some(DefId(u32::try_from(index).ok()?))
            } else {
                None
            }
        })
        .collect()
}
fn checked_rows(report: &Report, expected: usize) -> TestResult {
    let constants = constants(report);
    assert_eq!(constants.len(), expected);
    let mut nodes = HashSet::default();
    let mut bodies = HashSet::default();
    for id in constants {
        let definition = &report.resolution.definitions[id.0 as usize];
        let key = definition.node.ok_or("real constant source")?;
        let module = report.resolution.scopes[definition.scope.0 as usize].module;
        assert_eq!(
            report
                .resolution
                .item_definitions
                .get(&(module, key.node, Namespace::Value)),
            Some(&id)
        );
        let body = *report
            .resolution
            .node_scopes
            .get(&(module, key.node))
            .ok_or("constant body scope")?;
        assert_eq!(
            report.resolution.scopes[body.0 as usize].parent,
            Some(definition.scope)
        );
        assert!(nodes.insert(key) && bodies.insert(body));
    }
    assert!(
        report
            .resolution
            .bindings
            .iter()
            .all(|(_, name, _, _)| report.package.names.get(name) != "_")
    );
    Ok(())
}
#[test]
fn module_block_macro_and_inactive_items_preserve_real_namespace_identities() -> TestResult {
    for (name, count) in [
        ("module-repeat", 2),
        ("block-repeat", 2),
        ("nested-distinct-scopes", 2),
    ] {
        let text = fixture(6, name)?;
        for expanded in [false, true] {
            let found = crate::test_source::resolve(&text, expanded)?;
            checked_rows(&found, count)?;
        }
    }
    let text = crate::test_source::fixture("pa8/tests/spec/expansion/unnamed-constants.rs")?;
    checked_rows(&report(&text)?, 4)?;
    for name in ["named-duplicate", "trait-unnamed", "inherent-unnamed"] {
        assert!(report(&fixture(6, name)?).is_err(), "{name}");
    }
    assert!(report(&fixture(25, "unresolved-initializer")?).is_err());
    let associated = report(&fixture(6, "named-associated")?)?;
    let mut owners = HashSet::default();
    let mut names = Vec::new();
    for definition in &associated.resolution.definitions {
        if definition.kind != DefKind::Const {
            continue;
        }
        let Some(key) = definition.node else { continue };
        let unit = associated.package.unit(key.file);
        let crate::syntax::ast::Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
            return Err("constant definition lost its actual source item".into());
        };
        assert!(matches!(
            unit.configured.ast.items[index as usize].kind,
            ItemKind::Const { .. }
        ));
        assert!(owners.insert(key));
        names.push(associated.package.names.get(definition.name));
    }
    names.sort_unstable();
    assert_eq!(names, ["NAMED", "NAMED", "OTHER"]);
    assert_eq!(owners.len(), 3);
    for name in ["unexecuted-macro", "disabled-owner", "visibility-disabled"] {
        let found = report(&fixture(25, name)?)?;
        checked_rows(&found, 0)?;
        assert!(found.unused_visibility_warnings()?.is_empty());
        let analysis = pipeline::analyze_source(&found)?;
        assert_eq!(analysis.facts.constant_work.initializer_validations, 0);
    }
    Ok(())
}
