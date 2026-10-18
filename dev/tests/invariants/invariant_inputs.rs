//! Early variance hints preserve only proved invariant inputs, never unknown slots.
use super::RigidInputs;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    sema::{self, model::Declarations},
    support::source::SourceFile,
    types::{ItemId, aggregate::Definitions, variance::Variance},
};
use std::{collections::HashMap, path::Path};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn fixture() -> TestResult<(Declarations, HashMap<String, ItemId>)> {
    let source = "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}struct Direct<'a,T:A<'a>>(T::Out);struct Tuple<'a,T:A<'a>>((T::Out,));struct Callable<'a,T:A<'a>>(fn()->T::Out);struct Mixed<'a,'b,T:A<'a>>(T::Out,&'b u8);struct Inner<T>(T);struct Wrapped<'a,T:A<'a>>(Inner<T::Out>);struct Cov<'a>(&'a u8);";
    let report = resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "invariants.rs".into(),
            path: "invariants.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)?;
    let facts = sema::declarations::analyze(&report)?;
    let mut owners = HashMap::new();
    for definition in &facts.aggregates {
        let name = report.resolution.definitions[definition.item.0 as usize].name;
        owners.insert(report.package.names.get(name).to_owned(), definition.item);
    }
    Ok((facts, owners))
}
#[test]
fn early_hints_are_a_conservative_subset_of_completed_variance() -> TestResult {
    let (facts, owners) = fixture()?;
    let mut hints = RigidInputs::default();
    for (name, expected) in [
        ("Direct", true),
        ("Tuple", true),
        ("Callable", true),
        ("Mixed", false),
        ("Wrapped", false),
        ("Cov", false),
    ] {
        let item = *owners.get(name).ok_or("missing nominal")?;
        let actual = hints.demand(
            &facts.types,
            &facts.aggregates,
            item,
            facts.definitions[item.0 as usize],
        )?;
        assert_eq!(actual, expected, "{name}");
        if actual || name == "Wrapped" {
            // Wrapped is ultimately invariant, but an early query must not assume
            // the external Inner declaration's variance to grant this hint.
            assert!(
                facts
                    .lifetimes
                    .variances
                    .get(item)
                    .ok_or("missing completed variance")?
                    .iter()
                    .all(|v| *v == Variance::Invariant)
            );
        }
    }
    Ok(())
}
#[test]
fn missing_templates_are_not_cached_and_completed_queries_share_work() -> TestResult {
    let (facts, owners) = fixture()?;
    let item = *owners.get("Direct").ok_or("missing nominal")?;
    let template = facts.definitions[item.0 as usize];
    let mut hints = RigidInputs::default();
    let mut definitions = Definitions::default();
    assert!(!hints.demand(&facts.types, &definitions, item, template)?);
    assert!(!hints.completed.contains_key(&item));
    let definition = facts.aggregates.get(item).ok_or("missing raw fields")?;
    definitions.insert(
        item,
        definition.kind,
        definition.repr,
        facts.aggregates.variants(definition),
    )?;
    assert!(hints.demand(&facts.types, &definitions, item, template)?);
    let before = hints.steps;
    for _ in 0..128 {
        assert!(hints.demand(&facts.types, &definitions, item, template)?);
    }
    assert_eq!(hints.steps - before, 128);
    Ok(())
}
