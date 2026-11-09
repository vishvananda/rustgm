//! A client validates an imported impl on demand only outside every consumer
//! specialization relation. The actual `min_specialization` client relates no
//! imported impl; each forged endpoint stops that demand by name.
use super::{Builder, Comparison, Graph, PairKey, Relation, State, VertexId};
use crate::{
    lower::instances::SourcePublication, module_packets, resolve::model::DefId,
    sema::model::Declarations, source_packets,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const GATE: &str = "imported implementation validation before its specialization conformance order";
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa29/tests/spec/scheduling-context/{name}.rs"
    ))?)
}
fn gate(facts: &Declarations, owner: DefId) -> Option<&'static str> {
    super::super::require_unordered_import(facts, owner).err()
}
fn graph(facts: &mut Declarations) -> Result<&mut Graph> {
    Ok(facts
        .generics
        .implementations
        .as_deref_mut()
        .and_then(|registry| registry.specialization_graph_mut())
        .ok_or("actual client graph")?)
}
#[test]
fn forged_imported_specialization_endpoints_stop_demanded_conformance() -> Result {
    let provider = module_packets::exported_profile(
        &fixture("usize-provider")?,
        SourcePublication::ModuleBodies,
        "usize_provider",
    )?;
    let (_, analysis, _) = source_packets::client_analysis(provider, &fixture("minspec")?)?;
    let mut facts = analysis.facts;
    let registry = facts
        .generics
        .implementations
        .as_deref()
        .ok_or("actual client registry")?;
    let imported: Vec<_> = registry
        .headers()
        .filter(|header| header.source.is_none())
        .map(|header| header.owner)
        .collect();
    assert!(imported.len() >= 3, "imported impls {imported:?}");
    let keys = imported
        .iter()
        .map(|&owner| registry.entry_key(owner))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for &owner in &imported {
        assert_eq!(
            gate(&facts, owner),
            None,
            "unforged imported impl {owner:?}"
        );
    }
    let (State::Sealed(forest) | State::Conforming(forest)) = &mut graph(&mut facts)?.state else {
        return Err("the analyzed client graph is closed".into());
    };
    forest.by_entry.insert(keys[0], VertexId::new(0)?);
    assert_eq!(
        gate(&facts, imported[0]),
        Some(GATE),
        "forged forest vertex"
    );
    assert_eq!(
        gate(&facts, imported[1]),
        None,
        "an unrelated imported impl"
    );
    graph(&mut facts)?.state = State::Building(Box::default());
    assert_eq!(gate(&facts, imported[0]), None, "an empty building graph");
    let mut builder = Box::<Builder>::default();
    builder.relations.push(Relation {
        child: keys[1],
        parent: keys[2],
        parent_arguments: crate::types::GenericArgs::EMPTY,
    });
    graph(&mut facts)?.state = State::Building(builder);
    assert_eq!(
        gate(&facts, imported[1]),
        Some(GATE),
        "forged relation child"
    );
    assert_eq!(
        gate(&facts, imported[2]),
        Some(GATE),
        "forged relation parent"
    );
    assert_eq!(
        gate(&facts, imported[0]),
        None,
        "outside the forged relation"
    );
    let mut builder = Box::<Builder>::default();
    builder
        .attempts
        .insert(PairKey::new(keys[0], keys[2])?, Comparison::Running);
    graph(&mut facts)?.state = State::Building(builder);
    assert_eq!(gate(&facts, imported[0]), Some(GATE), "forged comparison");
    assert_eq!(
        gate(&facts, imported[1]),
        None,
        "outside the forged comparison"
    );
    Ok(())
}
