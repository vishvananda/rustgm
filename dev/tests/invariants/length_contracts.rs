//! Canonical slice originals keep declaration ownership through both collectors.
use super::*;
use crate::{body, borrow, sema, types};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(source: &str) -> Result<(crate::resolve::expansion::Report, Declarations)> {
    let report = crate::test_source::resolve(source, true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
fn function(facts: &Declarations) -> Result<Function> {
    let length = facts
        .methods
        .demanded_length()
        .ok_or("missing demanded method")?;
    Ok(*facts
        .functions
        .get(
            *facts
                .function_indices
                .get(&length.definition)
                .ok_or("missing method index")?,
        )
        .ok_or("missing method")?)
}
#[test]
fn source_free_originals_use_exact_declaration_keys_in_both_collectors() -> Result {
    let (report, facts) = fixture("fn main(){let a=[1u8,2];let _=a.len();}")?;
    let function = function(&facts)?;
    let key = body::ir::BodyKey::Declaration(function.definition);
    let batch = body::regions::collect(&facts)?;
    let batch = batch
        .bodies
        .iter()
        .filter(|c| c.key == key)
        .collect::<Vec<_>>();
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0].signature, Some(function.signature));
    assert_eq!(batch[0].universals, 1);
    assert_eq!(batch[0].parameters.len(), 1);
    assert!(batch[0].assumptions.is_empty() && batch[0].obligations.is_empty());
    let incremental = body::regions::Collector::default().finish(&facts)?;
    assert_eq!(
        incremental.bodies.iter().filter(|c| c.key == key).count(),
        1
    );
    let templates = body::templates::build(&report, &facts)?;
    assert_eq!(
        templates
            .bodies
            .iter()
            .filter(|t| t.body.key() == key)
            .count(),
        1
    );
    let mut copies = types::copy::Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut copies,
    };
    borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &incremental,
        &templates,
        &[],
        |ty, field| body::drops::nominal_position(&facts, ty, field),
    )?;
    Ok(())
}
#[test]
fn library_contract_cache_rejects_unregistered_or_changed_originals() -> Result {
    let (_, facts) = fixture("fn main(){let _=[1u8].len();}")?;
    let original = function(&facts)?;
    let length = facts.methods.demanded_length().ok_or("registered method")?;
    let mut forged_length = length;
    forged_length.owner = original.definition;
    assert!(forged_length.validate(&facts, &original).is_err());
    forged_length = length;
    forged_length.receiver = TypeId::UNIT;
    assert!(forged_length.validate(&facts, &original).is_err());
    let mut collector = body::regions::Collector::default();
    collector.function(&facts, &original)?;
    let mut forged = original;
    forged.signature = TypeId::UNIT;
    assert!(contract(&facts, &forged).is_err());
    assert!(collector.function(&facts, &forged).is_err());
    forged = original;
    forged.body = Some(original.node);
    assert!(build(&facts, &forged).is_err());
    assert!(collector.function(&facts, &forged).is_err());
    forged = original;
    forged.definition = facts
        .functions
        .iter()
        .find(|f| f.definition != original.definition)
        .ok_or("ordinary function")?
        .definition;
    assert!(contract(&facts, &forged).is_err());
    assert!(build(&facts, &forged)?.is_none());
    assert!(collector.function(&facts, &forged).is_err());
    Ok(())
}
#[test]
fn library_spelling_does_not_demand_unused_originals() -> Result {
    let (report, facts) =
        fixture("struct S;impl S{fn len(&self)->usize{7}}fn main(){let _=S.len();}")?;
    assert!(report.resolution.library_types.slice_length().is_some());
    assert!(facts.methods.demanded_length().is_none());
    assert!(body::templates::build(&report, &facts)?.bodies.is_empty());
    Ok(())
}
#[test]
fn generic_method_original_is_shared_across_unrelated_source_and_repeated_calls() -> Result {
    use std::fmt::Write;
    for width in [8, 32, 128] {
        let mut source = String::from("fn main(){let a=[1u8,2];");
        for _ in 0..width {
            source.push_str("let _=a.len();");
        }
        source.push('}');
        for i in 0..width {
            write!(source, "fn unrelated{i}()->u8{{1}}")?;
        }
        let (report, facts) = fixture(&source)?;
        let original = function(&facts)?;
        let templates = body::templates::build(&report, &facts)?;
        assert_eq!(templates.bodies.len(), 1);
        assert!(
            matches!(templates.bodies[0].body.owner,body::ir::Owner::Function{definition,..}if definition==original.definition)
        );
        assert_eq!(
            facts
                .methods
                .calls
                .values()
                .filter(|c| c.method == original.definition)
                .count(),
            width
        );
        assert!(facts.methods.selection_probes <= width * 8);
    }
    Ok(())
}
#[test]
fn length_metadata_keeps_compact_ordinary_records() {
    println!(
        "length records: TypeId={} Kind={} Value={} NodeFacts={} Methods={} Collector={} Resolver={} Length={}",
        size_of::<types::TypeId>(),
        size_of::<types::Kind>(),
        size_of::<body::ir::Value>(),
        size_of::<sema::model::NodeFacts>(),
        size_of::<sema::methods::Methods>(),
        size_of::<body::regions::Collector>(),
        size_of::<crate::resolve::model::LibrarySliceLength>(),
        size_of::<sema::methods::slices::Length>()
    );
    assert_eq!(size_of::<types::TypeId>(), 4);
    assert_eq!(size_of::<types::Kind>(), 24);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    assert_eq!(size_of::<sema::model::NodeFacts>(), 80);
}
#[test]
fn earlier_array_trait_winner_does_not_publish_unused_slice_types() -> Result {
    let (report, facts) = fixture(
        "trait L{fn len(&self)->usize;}impl L for [u8;3]{fn len(&self)->usize{7}}fn main(){let _=[1u8,2,3].len();}",
    )?;
    let declaration = report
        .resolution
        .library_types
        .slice_length()
        .ok_or("reserved metadata")?;
    assert!(
        facts
            .methods
            .length_definition(declaration.definition)
            .is_none()
    );
    assert!(facts.definitions[declaration.definition.0 as usize].is_none());
    assert!(!facts.generics.parameters.contains_key(&declaration.owner));
    Ok(())
}
