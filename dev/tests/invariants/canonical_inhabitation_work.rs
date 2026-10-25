//! Canonical preparation and contextual cache hits keep their actual work bound.
use super::{Cache, Inhabited};
use crate::{
    resolve::{expansion::Report, package::ModuleId},
    sema::model::Declarations,
    types::{Kind, TypeId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn prepared() -> Result<(Report, Declarations, TypeId, ModuleId)> {
    let source = crate::test_source::fixture("pa18/tests/spec/canonical-inhabitation/generic-visibility-query.rs")?;
    let report = crate::test_source::resolve(&source, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = facts.functions.iter().find(|function| report.package.names.get(report.resolution.definitions[function.definition.0 as usize].name) == "probe")
        .ok_or("missing actual probe declaration")?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else { return Err("missing probe signature".into()) };
    let ty = *facts.types.list(parameters).and_then(|parameters| parameters.first()).ok_or("missing probe input")?;
    let index = report.package.modules.iter().position(|module| report.package.names.get(module.name) == "owner").ok_or("missing owner module")?;
    let module = ModuleId(u32::try_from(index)?);
    facts.aggregates.prepare(&mut facts.types, ty)?;
    Ok((report, facts, ty, module))
}
#[test]
fn canonical_fields_share_preparation_but_keep_matching_module_in_the_cache_key() -> Result {
    let (report, facts, ty, owner) = prepared()?;
    let mut cache = Cache::default();
    let before = facts.aggregates.instantiation_steps();
    let outer = super::super::is_inhabited(&facts, &report, ModuleId(0), ty, &mut cache);
    assert_eq!(outer.result, Ok(true));
    let inner = super::super::is_inhabited(&facts, &report, owner, ty, &mut cache);
    assert_eq!(inner.result, Ok(false));
    let warm = super::super::is_inhabited(&facts, &report, ModuleId(0), ty, &mut cache);
    assert_eq!(warm.result, Ok(true));
    assert_eq!(warm.work, 1);
    assert_eq!(facts.aggregates.instantiation_steps(), before);
    assert!(inner.work > warm.work);
    Ok(())
}
#[test]
fn malformed_variant_identity_is_cached_as_error_and_warm_reads_are_charged() -> Result {
    let (report, mut facts, ty, owner) = prepared()?;
    let Some(Kind::Adt(item, _)) = facts.types.kind(ty) else { return Err("probe is not nominal".into()) };
    let variant = *facts.members.variants(item).first().ok_or("missing actual nominal variant")?;
    facts.members.variants[variant.0 as usize].ordinal = u32::MAX;
    let mut cache = Cache::default();
    let mut graph = super::super::decision::Graph::new(1_000_000);
    let first = Inhabited::new(&facts, &report, owner, &mut cache).get(ty, &mut graph);
    assert_eq!(first, Err("inhabitation variant owner or ordinal changed"));
    let work = graph.work();
    let misses = cache.misses();
    let second = Inhabited::new(&facts, &report, owner, &mut cache).get(ty, &mut graph);
    assert_eq!(second, first);
    assert_eq!(graph.work(), work + 1);
    assert_eq!(cache.misses(), misses);
    Ok(())
}
