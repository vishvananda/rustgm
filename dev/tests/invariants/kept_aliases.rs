//! Only selected aliases owned by this Dispatcher may bypass source restoration.
use super::*;
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    sema::values::Instance,
    types::{
        Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        substitute::{Bound, RegionEquations},
    },
};
fn make_facts() -> Result<Declarations, Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    Ok(Declarations::new(&report)?)
}
fn selected(facts: &mut Declarations, region: RegionId) -> Result<Instance, &'static str> {
    Ok(Instance {
        definition: DefId(0),
        arguments: facts
            .types
            .intern_arguments(&[Argument::Lifetime(region)])?,
    })
}
fn source() -> Result<Bound, &'static str> {
    let mut fold = Bound::close_projection_regions(1, 0, HashMap::default())?;
    let mut equations = RegionEquations::default();
    equations.extend([(RegionId(1), RegionId::bound(0, 0)?)])?;
    fold.exchange_projection_equations(&mut equations)?;
    Ok(fold)
}
#[test]
fn selected_aliases_do_not_admit_foreign_gaps_or_result_only_regions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut facts = make_facts()?;
    let mut dispatcher = Dispatcher::new(1, 10)?;
    dispatcher.obligations.regions.end = 12;
    dispatcher.obligations.resume_at(20)?;
    dispatcher.obligations.regions.end = 22;
    let mut fold = source()?;
    let selected = selected(&mut facts, RegionId(11))?;
    let before = fold.retained_allocation().0;
    let upper = fold.projection_anchor_growth(1)?;
    dispatcher.retain_source_aliases(&mut facts, selected, &mut fold)?;
    assert!(fold.retained_allocation().0 <= before + upper);
    let bytes = fold.retained_allocation().0;
    dispatcher.retain_source_aliases(&mut facts, selected, &mut fold)?;
    assert_eq!(fold.retained_allocation().0, bytes);
    assert_eq!(fold.region(RegionId(11), 0)?, RegionId(11));
    assert_eq!(fold.region(RegionId(1), 0)?, RegionId::bound(0, 0)?);
    for region in [RegionId(2), RegionId(15), RegionId(20), RegionId(25)] {
        assert!(fold.region(region, 0).is_err());
    }
    let foreign = self::selected(&mut facts, RegionId(15))?;
    dispatcher.retain_source_aliases(&mut facts, foreign, &mut fold)?;
    assert!(dispatcher.source_instance(&mut facts, foreign).is_err());
    assert!(fold.region(RegionId(15), 0).is_err());
    assert!(facts.source_computations.is_none());
    let result_only = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId(20),
        referent: TypeId::primitive(Primitive::U8),
    })?;
    assert!(fold.apply(&mut facts.types, result_only).is_err());
    Ok(())
}
#[test]
fn selected_aliases_never_replace_original_source_mappings()
-> Result<(), Box<dyn std::error::Error>> {
    let mut facts = make_facts()?;
    let mut dispatcher = Dispatcher::new(1, 1)?;
    let selected = selected(&mut facts, RegionId(1))?;
    let mut fold = source()?;
    assert!(
        dispatcher
            .retain_source_aliases(&mut facts, selected, &mut fold)
            .is_err()
    );
    for region in [RegionId::STATIC, RegionId::bound(0, 0)?] {
        assert!(fold.retain_projection_anchor(region).is_err());
    }
    assert_eq!(fold.region(RegionId(1), 0)?, RegionId::bound(0, 0)?);
    assert!(fold.projection_anchor_growth(usize::MAX)? > 64 * 1024 * 1024);
    Ok(())
}

#[test]
fn selected_proof_aliases_compose_with_source_closing_without_paying_debts()
-> Result<(), Box<dyn std::error::Error>> {
    let mut facts = make_facts()?;
    let mut dispatcher = Dispatcher::new(1, 10)?;
    let mut proof = crate::types::lifetime_proof::Obligations::new(20);
    proof.regions.end = 21;
    proof.equalities.push((RegionId(1), RegionId(20)));
    proof.outlives.push((RegionId(1), RegionId::STATIC));
    let debts = proof.equalities.clone();
    let selected = self::selected(&mut facts, RegionId(20))?;
    let restored = super::super::implementation::restore_constant(
        &mut facts,
        &mut dispatcher.context,
        selected,
        &proof,
        &dispatcher.obligations,
    )?;
    assert_eq!(restored, self::selected(&mut facts, RegionId(1))?);
    let mut source = source()?;
    let canonical = restored.substitute(&mut facts.types, &mut source)?;
    assert_eq!(
        canonical,
        self::selected(&mut facts, RegionId::bound(0, 0)?)?
    );
    assert_eq!(
        dispatcher.source_instance(&mut facts, canonical)?,
        self::selected(&mut facts, RegionId(10))?
    );
    assert_eq!(proof.equalities, debts);
    assert_eq!(proof.outlives, [(RegionId(1), RegionId::STATIC)]);
    assert!(dispatcher.context.steps > 0);
    let mut missing = proof.clone();
    missing.equalities.clear();
    let unanchored = super::super::implementation::restore_constant(
        &mut facts,
        &mut dispatcher.context,
        selected,
        &missing,
        &dispatcher.obligations,
    )?;
    assert_eq!(unanchored, selected);
    assert!(
        unanchored
            .substitute(&mut facts.types, &mut source)
            .is_err()
    );
    let foreign = self::selected(&mut facts, RegionId(25))?;
    let before = dispatcher.context.steps;
    assert!(
        super::super::implementation::restore_constant(
            &mut facts,
            &mut dispatcher.context,
            foreign,
            &proof,
            &dispatcher.obligations,
        )
        .is_err()
    );
    assert!(dispatcher.context.steps > before);
    Ok(())
}

#[test]
fn selections_without_equalities_preserve_input_types_and_their_unpaid_debts()
-> Result<(), Box<dyn std::error::Error>> {
    let mut facts = make_facts()?;
    let mut dispatcher = Dispatcher::new(0, 10)?;
    let reference = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId(1),
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let selected = Instance {
        definition: DefId(0),
        arguments: facts.types.intern_arguments(&[Argument::Type(reference)])?,
    };
    let mut proof = crate::types::lifetime_proof::Obligations::new(10);
    proof.types.push((reference, RegionId::STATIC));
    let before = dispatcher.context.steps;
    assert_eq!(
        super::super::implementation::restore_constant(
            &mut facts,
            &mut dispatcher.context,
            selected,
            &proof,
            &dispatcher.obligations,
        )?,
        selected
    );
    assert_eq!(proof.types, [(reference, RegionId::STATIC)]);
    assert!(proof.equalities.is_empty());
    assert_eq!(dispatcher.context.steps, before);
    assert!(facts.source_computations.is_none());
    Ok(())
}
