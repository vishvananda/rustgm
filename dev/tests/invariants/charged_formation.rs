//! Failed formation rolls back its own visitation, retaining earlier completed dependencies.
use super::*;
use crate::{
    resolve::{expansion::Report, model::DefKind},
    types::{Primitive, traits::Application},
};
use std::cell::Cell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn owner(report: &Report, name: &str) -> TestResult<DefId> {
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            report.package.names.get(row.name) == name
                && matches!(row.kind, DefKind::Struct | DefKind::Trait)
        })
        .ok_or("actual local nominal or trait")?;
    Ok(DefId(u32::try_from(index)?))
}
fn nominal(facts: &mut Declarations, owner: DefId, argument: TypeId) -> TestResult<TypeId> {
    let arguments = facts.types.intern_arguments(&[Argument::Type(argument)])?;
    Ok(facts.types.intern(Kind::Adt(ItemId(owner.0), arguments))?)
}
#[test]
fn counted_formation_replays_real_clauses_and_rolls_back_only_failed_visitation() -> TestResult {
    let report = crate::test_source::resolve(
        "trait A{}struct Leaf<T:A>(T);struct Hold<T>(T);impl A for u8{}fn main(){}",
        true,
    )?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let leaf = owner(&report, "Leaf")?;
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U16);
    let complete = nominal(&mut facts, leaf, byte)?;
    let pending = nominal(&mut facts, leaf, word)?;
    let tuple = facts.types.intern_list(&[complete, pending])?;
    let tuple = facts.types.intern(Kind::Tuple(tuple))?;
    let root = nominal(&mut facts, owner(&report, "Hold")?, tuple)?;
    let mut checked = Requirements::default();
    assert!(!requirements_counted(&mut facts, complete, &mut checked, &|_, _| Ok(()))?.is_empty());
    let prior = checked.types.clone();
    let calls = Cell::new(0_usize);
    let attempted = Cell::new(0_usize);
    let failed = requirements_counted(&mut facts, root, &mut checked, &|steps, _| {
        calls.set(calls.get() + 1);
        attempted.set(attempted.get().saturating_add(steps));
        if calls.get() == 5 {
            Err("frozen formation failure")
        } else {
            Ok(())
        }
    });
    assert_eq!(failed, Err("frozen formation failure"));
    assert_eq!(
        checked.types, prior,
        "failed attempt erased a completed dependency or retained a partial root"
    );
    assert!(checked.work.is_empty());
    assert!(checked.unique.is_empty());
    let failed_work = attempted.get();
    let mut ordinary = Requirements::default();
    requirements(&mut facts, complete, &mut ordinary)?;
    let expected = requirements(&mut facts, root, &mut ordinary)?;
    let bytes = Cell::new(0_usize);
    let actual = requirements_counted(&mut facts, root, &mut checked, &|steps, retained| {
        attempted.set(attempted.get().saturating_add(steps));
        bytes.set(bytes.get().saturating_add(retained));
        Ok(())
    })?;
    assert_eq!(actual, expected);
    assert!(!actual.is_empty());
    assert!(attempted.get() > failed_work);
    assert!(prior.is_subset(&checked.types));
    assert!(
        requirements_counted(&mut facts, root, &mut checked, &|_, retained| {
            assert_eq!(retained, 0, "warm visitation unexpectedly allocated");
            Ok(())
        })?
        .is_empty()
    );
    substitution_and_trait_cache(&mut facts, &report, leaf, byte, word)?;
    eprintln!(
        "counted formation completed={} retained={} failed_work={failed_work} total_work={} logical_bytes={} Requirements={}",
        prior.len(),
        checked.types.len(),
        attempted.get(),
        bytes.get(),
        size_of::<Requirements>()
    );
    Ok(())
}
fn substitution_and_trait_cache(
    facts: &mut Declarations,
    report: &Report,
    leaf: DefId,
    byte: TypeId,
    word: TypeId,
) -> TestResult {
    let parameter = facts.generics.parameter_ids(&facts.types, leaf)?[0];
    let clause = facts.generics.predicates[&leaf][0].value;
    let mapping = [(parameter, Argument::Type(byte))];
    let mut plain = substitute::Bound::parameters_captured(&facts.types, &mapping, Vec::new())?;
    let charged = Cell::new(0_usize);
    let charge = |work, _| {
        charged.set(charged.get().saturating_add(work));
        Ok(())
    };
    let mut counted = substitute::Bound::parameters_captured_charged(
        &facts.types,
        &mapping,
        Vec::new(),
        &charge,
    )?;
    assert_eq!(
        plain.clause(&mut facts.types, clause)?,
        counted.clause_charged(&mut facts.types, clause, &charge)?
    );
    assert!(charged.get() > 0);
    let arguments = facts.types.intern_arguments(&[])?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(owner(report, "A")?.0),
        arguments,
    })?;
    let contracts = facts
        .generics
        .traits
        .as_deref_mut()
        .ok_or("real trait contracts")?;
    let predicate = Predicate {
        ty: byte,
        bound: Bound::Trait(application),
    };
    let ordinary = contracts.requirements(&mut facts.types, predicate)?;
    let counted = contracts.requirements_counted(&mut facts.types, predicate, &|work, bytes| {
        assert_eq!(work, 1);
        assert_eq!(bytes, 0);
        Ok(())
    })?;
    assert!(std::sync::Arc::ptr_eq(&ordinary, &counted));
    let predicate = Predicate {
        ty: word,
        ..predicate
    };
    assert!(
        contracts
            .requirements_counted(&mut facts.types, predicate, &|_, _| Err(
                "frozen trait failure"
            ))
            .is_err()
    );
    let retried = contracts.requirements_counted(&mut facts.types, predicate, &charge)?;
    let cached = contracts.requirements(&mut facts.types, predicate)?;
    assert!(std::sync::Arc::ptr_eq(&retried, &cached));
    Ok(())
}
