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
fn nominal_counted(
    facts: &mut Declarations,
    root: TypeId,
    checked: &mut Requirements,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<Vec<Clause>, &'static str> {
    requirements_inside_observed::<true, false>(facts, root, 0, checked, charge)
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
    assert!(!nominal_counted(&mut facts, complete, &mut checked, &|_, _| Ok(()))?.is_empty());
    let prior = checked.types.clone();
    let calls = Cell::new(0_usize);
    let attempted = Cell::new(0_usize);
    let failed = nominal_counted(&mut facts, root, &mut checked, &|steps, _| {
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
    let actual = nominal_counted(&mut facts, root, &mut checked, &|steps, retained| {
        attempted.set(attempted.get().saturating_add(steps));
        bytes.set(bytes.get().saturating_add(retained));
        Ok(())
    })?;
    assert_eq!(actual, expected);
    assert!(!actual.is_empty());
    assert!(attempted.get() > failed_work);
    assert!(prior.is_subset(&checked.types));
    assert!(
        nominal_counted(&mut facts, root, &mut checked, &|_, retained| {
            assert_eq!(retained, 0, "warm visitation unexpectedly allocated");
            Ok(())
        })?
        .is_empty()
    );
    substitution_and_trait_cache(&mut facts, &report, leaf, byte, word)?;
    structural_input_clauses(&mut facts, leaf)?;
    mapped_environment_failure_and_index_retry()?;
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

fn structural_input_clauses(facts: &mut Declarations, leaf: DefId) -> TestResult {
    let parameter = facts.generics.parameters[&leaf][0].argument;
    let Argument::Type(ty) = parameter else {
        return Err("actual Leaf type parameter".into());
    };
    let byte = TypeId::primitive(Primitive::U8);
    let sized: Clause = Predicate {
        ty,
        bound: Bound::Sized,
    }
    .into();
    let mut roots = vec![(ty, false)];
    for kind in [
        Kind::Array {
            element: ty,
            length: 1,
        },
        Kind::Slice(ty),
    ] {
        roots.push((facts.types.intern(kind)?, true));
    }
    for (elements, required) in [
        (&[ty, byte][..], true),
        (&[byte, ty][..], false),
        (&[ty][..], false),
    ] {
        let list = facts.types.intern_list(elements)?;
        roots.push((facts.types.intern(Kind::Tuple(list))?, required));
    }
    let parameters = facts.types.intern_list(&[ty])?;
    roots.push((
        facts.types.intern(Kind::Function {
            parameters,
            result: ty,
            bound_regions: 0,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?,
        false,
    ));
    roots.push((
        facts.types.intern(Kind::Pointer {
            mutable: false,
            pointee: ty,
        })?,
        false,
    ));
    for (root, required) in roots {
        let values = formation_requirements_counted(
            facts,
            root,
            &mut FormationRequirements::default(),
            &|_, _| Ok(()),
        )?;
        assert_eq!(
            values.contains(&sized),
            required,
            "actual formation root {root:?}"
        );
    }
    let reference = facts.types.intern(Kind::Reference {
        mutable: false,
        referent: ty,
        region: crate::types::RegionId::STATIC,
    })?;
    let values = formation_requirements_counted(
        facts,
        reference,
        &mut FormationRequirements::default(),
        &|_, _| Ok(()),
    )?;
    assert!(!values.contains(&sized));
    assert!(
        values.contains(
            &Predicate {
                ty,
                bound: Bound::Outlives(crate::types::RegionId::STATIC)
            }
            .into()
        )
    );
    let local = facts.types.intern(Kind::Reference {
        mutable: false,
        referent: ty,
        region: crate::types::RegionId::bound(0, 0)?,
    })?;
    let parameters = facts.types.intern_list(&[local])?;
    let callable = facts.types.intern(Kind::Function {
        parameters,
        result: byte,
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let values = formation_requirements_counted(
        facts,
        callable,
        &mut FormationRequirements::default(),
        &|_, _| Ok(()),
    )?;
    assert!(
        values.is_empty(),
        "callable-local lifetime cannot become a caller assumption"
    );
    assert_eq!(
        size_of::<FormationRequirements>(),
        size_of::<Requirements>()
    );
    Ok(())
}

fn mapped_environment_failure_and_index_retry() -> TestResult {
    use crate::types::{RegionId, parameters::environment::Retained};
    let report = crate::test_source::resolve(
        "trait A<U>{}trait B<U>{}fn held<'a,T:A<&'a u8>+B<&'a u16>>(_: &'a T)where for<'b>fn(&'b &'a u8):Copy{}fn main(){}",
        true,
    )?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.kind == DefKind::Function && report.package.names.get(row.name) == "held"
        })
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or("actual held function")?;
    let environment = std::sync::Arc::clone(
        facts
            .generics
            .environments
            .get(&definition)
            .ok_or("actual declared caller")?,
    );
    let original: Vec<_> = environment.trait_clauses()?.collect();
    let mut ordinary_fold = substitute::Bound::new(vec![RegionId(1)]);
    let expected = environment.substitute_regions(&mut facts.types, &mut ordinary_fold)?;
    let calls = Cell::new(0_usize);
    let work = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let failing = Cell::new(true);
    let charge = |steps, retained| {
        calls.set(calls.get() + 1);
        work.set(work.get().saturating_add(steps));
        bytes.set(bytes.get().saturating_add(retained));
        if failing.get() && calls.get() == 4 {
            Err("frozen environment allocation failure")
        } else {
            Ok(())
        }
    };
    let mut fold = substitute::Bound::new(vec![RegionId(1)]);
    assert!(
        environment
            .substitute_declared_regions_charged(&mut facts.types, &mut fold, &charge)
            .is_err()
    );
    let failed = work.get();
    let failed_fold = fold.steps;
    assert_eq!(environment.trait_clauses()?.collect::<Vec<_>>(), original);
    failing.set(false);
    let mapped =
        environment.substitute_declared_regions_charged(&mut facts.types, &mut fold, &charge)?;
    assert_eq!(
        mapped.trait_clauses()?.collect::<Vec<_>>(),
        expected.trait_clauses()?.collect::<Vec<_>>()
    );
    assert!(work.get() > failed);
    assert!(fold.steps >= failed_fold);
    let (failed_index_work, index_work) = fresh_index_retries(&facts.types, &mapped, &bytes)?;
    let retained = mapped.retain_owned_from(&environment, &mut Retained::default());
    assert!(bytes.get() >= retained.bytes);
    eprintln!(
        "mapped environment Environment={} retained={} logical_bytes={} work={} fold_work={} failed_index_work={} index_work={}",
        size_of::<crate::types::parameters::environment::Environment>(),
        retained.bytes,
        bytes.get(),
        work.get(),
        fold.steps,
        failed_index_work,
        index_work
    );
    Ok(())
}

fn fresh_index_retries(
    types: &Store,
    mapped: &crate::types::parameters::environment::Environment,
    bytes: &Cell<usize>,
) -> TestResult<(usize, usize)> {
    let attempts = Cell::new(0_usize);
    let index_work = Cell::new(0_usize);
    let failed_index = mapped.trait_index_charged(types, &|steps, _| {
        attempts.set(attempts.get() + 1);
        index_work.set(index_work.get().saturating_add(steps));
        if attempts.get() == 6 {
            Err("frozen caller index failure")
        } else {
            Ok(())
        }
    });
    assert!(failed_index.is_err());
    let failed_index_work = index_work.get();
    let index = mapped
        .trait_index_charged(types, &|steps, retained| {
            index_work.set(index_work.get().saturating_add(steps));
            bytes.set(bytes.get().saturating_add(retained));
            Ok(())
        })?
        .ok_or("completed mapped trait index")?;
    assert!(index_work.get() > failed_index_work);
    let warm = mapped
        .trait_index_charged(types, &|_, retained| {
            assert_eq!(retained, 0);
            Ok(())
        })?
        .ok_or("warm mapped trait index")?;
    assert!(std::sync::Arc::ptr_eq(&index, &warm));
    assert_eq!(
        index.seen.len(),
        2,
        "failed partial index became cached absence"
    );
    Ok((failed_index_work, index_work.get()))
}
