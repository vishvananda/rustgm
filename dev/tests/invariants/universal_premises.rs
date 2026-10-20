//! A universal caller premise supplies a fresh instance, never a new universal fact.
use super::{Declarations, quantified_tests::valid, tests};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::traits::{
        assumptions::{Assumptions, ScopedSelection},
        implementations::Availability,
        selection::{self, Caller, Selection, Source},
    },
    types::{
        Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        lifetime_proof::Obligations,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::Predicate,
        },
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn parameter(report: &Report, facts: &Declarations, index: usize) -> TestResult<TypeId> {
    let owner = tests::definition(report, "caller")?;
    let Argument::Type(ty) = facts.generics.parameters[&owner][index].argument else {
        return Err("missing test type parameter".into());
    };
    Ok(ty)
}
fn predicate(
    facts: &mut Declarations,
    item: DefId,
    ty: TypeId,
    arguments: &[Argument],
) -> TestResult<Predicate> {
    let arguments = facts.types.intern_arguments(arguments)?;
    Ok(tests::predicate(facts, item, ty, arguments)?)
}
fn quantify(facts: &mut Declarations, count: u32, predicate: Predicate) -> TestResult<Clause> {
    Ok(Clause::ForAll(facts.types.intern_quantified(
        Quantified {
            bound_regions: count,
            predicate,
        },
    )?))
}
fn outer(count: u32) -> TestResult<Obligations> {
    let mut proof = Obligations::new(1);
    proof.regions.end = 1 + count;
    for index in 0..count {
        proof.equalities.push((
            RegionId(index + 1),
            RegionId::bound(0, u16::try_from(index)?)?,
        ));
    }
    Ok(proof)
}
fn select(
    facts: &mut Declarations,
    assumptions: &Assumptions,
    predicate: Predicate,
    proof: &mut Obligations,
) -> TestResult<Source> {
    let (mut context, goal) = tests::instantiate(facts, predicate)?;
    let Selection::Proven(evidence) = selection::select_available(
        facts,
        &mut context,
        &goal,
        Caller {
            assumptions,
            environment: None,
            first_region: proof.regions.end,
            retain_sized: false,
        },
        Availability::Validated,
    )?
    else {
        return Err("missing universal assumption instance".into());
    };
    let source = evidence.source;
    proof.extend(evidence.proof)?;
    proof.equalities.append(&mut context.region_equalities);
    Ok(source)
}

#[test]
fn repeated_uses_get_distinct_owned_instances_and_retain_original_provenance() -> TestResult {
    let (report, mut facts) = tests::declarations("trait A<'a>{}fn caller<T>(){}")?;
    let ty = parameter(&report, &facts, 0)?;
    let item = tests::definition(&report, "A")?;
    let value = predicate(
        &mut facts,
        item,
        ty,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let clause = quantify(&mut facts, 1, value)?;
    let mut assumptions = Assumptions::default();
    assumptions.insert_clause(&facts.types, clause)?;
    let mut proof = outer(2)?;
    for caller in [RegionId(1), RegionId(2), RegionId::STATIC] {
        let goal = predicate(&mut facts, item, ty, &[Argument::Lifetime(caller)])?;
        let first = proof.regions.end;
        let Source::QuantifiedAssumption {
            clause: original,
            subject,
            application,
        } = select(&mut facts, &assumptions, goal, &mut proof)?
        else {
            return Err("universal provenance was flattened".into());
        };
        assert_eq!(Clause::ForAll(original), clause);
        assert_eq!(subject, ty);
        let application = facts
            .types
            .trait_application(application)
            .ok_or("missing instance application")?;
        let arguments = facts
            .types
            .arguments(application.arguments)
            .ok_or("missing instance arguments")?
            .collect::<Vec<_>>();
        assert_eq!(arguments, [Argument::Lifetime(RegionId(first))]);
        assert_eq!(proof.regions.end, first + 1);
    }
    assert_eq!(proof.scope_count(), 0);
    assert!(valid(&facts, &proof, 2)?);
    assert!(std::mem::size_of::<Source>() <= std::mem::size_of::<Predicate>() + 4);
    Ok(())
}

#[test]
fn immutable_environments_preserve_universal_and_captured_permissions() -> TestResult {
    use crate::sema::nominals::traits::State;
    use crate::types::parameters::environment::{Abilities, Environment};
    use std::sync::Arc;
    for (capture, expected) in [(0, true), (1, false)] {
        let (report, mut facts) = tests::declarations("trait A<'a,'b>{}fn caller<T>(){}")?;
        let ty = parameter(&report, &facts, 0)?;
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("missing parameter".into());
        };
        let item = tests::definition(&report, "A")?;
        let value = predicate(
            &mut facts,
            item,
            ty,
            &[
                Argument::Lifetime(RegionId::bound(0, 0)?),
                Argument::Lifetime(RegionId::bound(1, capture)?),
            ],
        )?;
        let assumption = quantify(&mut facts, 1, value)?;
        let environment = Environment::new(&facts.types, &[(id, Abilities::default())])?
            .with_clauses(&facts.types, &[assumption])?;
        let goal = predicate(
            &mut facts,
            item,
            ty,
            &[
                Argument::Lifetime(RegionId::bound(0, 0)?),
                Argument::Lifetime(RegionId::bound(1, 0)?),
            ],
        )?;
        let goal = quantify(&mut facts, 1, goal)?;
        let mut state = State::new(&mut facts, Some(Arc::new(environment)), Some((2, 11)))?;
        assert!(state.prove_clause(&mut facts, goal, false)?);
        assert!(state.prove_clause(&mut facts, goal, false)?);
        assert!(!state.prove_clause(&mut facts, goal, true)?);
        let proof = state.finish();
        assert_eq!(proof.regions, 11..15);
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(valid(&facts, &proof, 2)?, expected);
        let mut foreign = State::instance(&mut facts, 2)?;
        assert!(!foreign.prove_clause(&mut facts, goal, false)?);
    }
    Ok(())
}

#[test]
fn exact_projection_shortcuts_do_not_drop_quantified_competitors() -> TestResult {
    use crate::types::{GenericArgs, ItemId, parameters::environment::Environment, projection};
    let (report, mut facts) = tests::declarations("trait Base{type P;}trait A<'a>{type Q;}")?;
    let base = tests::definition(&report, "Base")?;
    let a = tests::definition(&report, "A")?;
    let p = tests::definition(&report, "P")?;
    let q = tests::definition(&report, "Q")?;
    let parent = predicate(&mut facts, base, BYTE, &[])?;
    let crate::types::parameters::predicates::Bound::Trait(parent_application) = parent.bound
    else {
        return Err("missing parent trait".into());
    };
    let subject = facts
        .types
        .intern(Kind::Projection(projection::Application {
            item: ItemId(p.0),
            subject: BYTE,
            trait_application: parent_application,
            arguments: GenericArgs::EMPTY,
        }))?;
    let exact = predicate(
        &mut facts,
        a,
        subject,
        &[Argument::Lifetime(RegionId::STATIC)],
    )?;
    let crate::types::parameters::predicates::Bound::Trait(application) = exact.bound else {
        return Err("missing exact trait".into());
    };
    let universal = predicate(
        &mut facts,
        a,
        subject,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let universal = quantify(&mut facts, 1, universal)?;
    let request = tests::checked_outer_request(
        &mut facts,
        projection::Application {
            item: ItemId(q.0),
            subject,
            trait_application: application,
            arguments: GenericArgs::EMPTY,
        },
    )?;
    let environment =
        Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[exact])?;
    assert!(
        matches!(selection::known_premises::opaque_normalized(&mut facts, Some(&environment), request)?, Some(Source::Assumption(value)) if value == exact)
    );
    let competing = environment.with_clauses(&facts.types, &[universal])?;
    assert!(
        selection::known_premises::opaque_normalized(&mut facts, Some(&competing), request)?
            .is_none()
    );
    let only = Environment::new(&facts.types, &[])?.with_clauses(&facts.types, &[universal])?;
    assert!(
        selection::known_premises::opaque_normalized(&mut facts, Some(&only), request)?.is_none()
    );
    Ok(())
}

#[test]
fn inferred_arguments_commit_only_after_the_subject_and_alternative_are_known() -> TestResult {
    use crate::types::{
        arguments::Constant,
        infer::{Context, Domain, Instantiation, arguments::Argument as Inferred},
    };
    for (ambiguous, unknown_self) in [(false, false), (true, false), (false, true)] {
        let (report, mut facts) =
            tests::declarations("trait A<'a,U,const N:usize>{}fn caller<T,U,const N:usize>(){}")?;
        let owner = tests::definition(&report, "caller")?;
        let formals = facts.generics.parameters[&owner]
            .iter()
            .map(|p| p.argument)
            .collect::<Vec<_>>();
        let ids = formals
            .iter()
            .map(|&arg| crate::sema::generics::parameter_id(&facts.types, arg))
            .collect::<Result<Vec<_>, _>>()?;
        let ty = parameter(&report, &facts, 0)?;
        let item = tests::definition(&report, "A")?;
        let three = facts.types.intern_constant(Constant {
            ty: TypeId::primitive(Primitive::Usize),
            bits: 3,
        })?;
        let mut assumptions = Assumptions::default();
        for value in [BYTE, TypeId::primitive(Primitive::U16)]
            .into_iter()
            .take(if ambiguous { 2 } else { 1 })
        {
            let value = predicate(
                &mut facts,
                item,
                ty,
                &[
                    Argument::Lifetime(RegionId::bound(0, 0)?),
                    Argument::Type(value),
                    Argument::Const(three),
                ],
            )?;
            let clause = quantify(&mut facts, 1, value)?;
            assumptions.insert_clause(&facts.types, clause)?;
        }
        let goal = predicate(
            &mut facts,
            item,
            ty,
            &[Argument::Lifetime(RegionId(1)), formals[1], formals[2]],
        )?;
        let mut context = Context::default();
        let subject = if unknown_self {
            context.variable(Domain::Any)?
        } else {
            context.known(&facts.types, ty)?
        };
        let unknown = context.variable(Domain::Any)?;
        let constant = context.const_variable(&facts.types, TypeId::primitive(Primitive::Usize))?;
        let mut substitution = Instantiation::new(
            &mut context,
            &facts.types,
            &[
                (ids[0], Inferred::Type(subject)),
                (ids[1], Inferred::Type(unknown)),
                (ids[2], Inferred::Const(constant)),
            ],
        )?;
        let goal = context.instantiate_predicate(&mut facts.types, goal, &mut substitution)?;
        if ambiguous || unknown_self {
            assert!(matches!(
                assumptions.select_scoped(&mut facts.types, &mut context, &goal, 2)?,
                ScopedSelection::Deferred
            ));
            assert!(context.head(&facts.types, unknown)?.is_none());
            assert!(matches!(
                context.resolved_const(&mut facts.types, constant),
                Err(crate::types::infer::Error::Unresolved)
            ));
            assert!(context.region_equalities.is_empty());
            if unknown_self {
                assert!(context.head(&facts.types, subject)?.is_none());
                let known = context.known(&facts.types, ty)?;
                context.equate(&facts.types, subject, known)?;
            } else {
                let known = context.known(&facts.types, BYTE)?;
                context.equate(&facts.types, unknown, known)?;
            }
        }
        let ScopedSelection::Proven(evidence) =
            assumptions.select_scoped(&mut facts.types, &mut context, &goal, 2)?
        else {
            return Err("resolved caller assumption did not resume".into());
        };
        assert_eq!(context.resolved(&mut facts.types, unknown)?, BYTE);
        assert_eq!(context.resolved_const(&mut facts.types, constant)?, three);
        assert_eq!(evidence.regions, 2..3);
        let mut proof = outer(1)?;
        let mut selected = Obligations::new(evidence.regions.start);
        selected.regions.end = evidence.regions.end;
        selected.equalities = context.region_equalities;
        proof.extend(selected)?;
        assert!(valid(&facts, &proof, 1)?);
    }
    Ok(())
}

#[test]
fn assumption_instances_after_header_waits_exclude_foreign_allocation_gaps() -> TestResult {
    use super::{Decision, Frame, Search, continuation::Progress, header_tests};
    let (report, mut facts) = tests::declarations(
        "trait A<'a>{}trait B<'a>{}impl<'a,T:A<'a>>B<'a>for T{}fn caller<T>(){}",
    )?;
    let ty = parameter(&report, &facts, 0)?;
    let a = tests::definition(&report, "A")?;
    let b = tests::definition(&report, "B")?;
    let value = predicate(
        &mut facts,
        a,
        ty,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let clause = quantify(&mut facts, 1, value)?;
    let mut assumptions = Assumptions::default();
    assumptions.insert_clause(&facts.types, clause)?;
    let goal = predicate(&mut facts, b, ty, &[Argument::Lifetime(RegionId(1))])?;
    let (mut context, goal) = tests::instantiate(&mut facts, goal)?;
    let mut delayed = header_tests::delay(&mut facts, &[b])?;
    let header = delayed.pop().ok_or("missing delayed implementation")?;
    let mut search = Search::new(
        Caller {
            assumptions: &assumptions,
            environment: None,
            first_region: 2,
            retain_sized: false,
        },
        Availability::Declarations,
    );
    let frame = Frame::new(&facts, &mut context, &goal, 2)?;
    assert_eq!(
        search.begin(&mut facts, frame)?,
        Progress::Pending(crate::sema::traits::selection::Dependency::Header(
            header.owner
        ))
    );
    header_tests::stable_poll(&mut search, &mut facts, header.owner)?;
    let first = search.query.allocations.end();
    let foreign = first..first + 7;
    search.resume_at(foreign.end)?;
    header_tests::publish(&mut facts, header)?;
    let Progress::Ready(Decision::Proven(root)) = search.poll(&mut facts)? else {
        return Err("universal assumption child did not resume".into());
    };
    let Selection::Proven(evidence) =
        search.result(&mut facts, &mut context, &goal, Decision::Proven(root))?
    else {
        return Err("missing resumed proof".into());
    };
    assert!(evidence.proof.regions.end > foreign.end);
    let mut restore = evidence.proof.body_restoration()?;
    for region in foreign {
        assert!(!evidence.proof.owns_region(RegionId(region)));
        assert!(restore.region(RegionId(region), 0).is_err());
    }
    let mut proof = outer(1)?;
    proof.extend(evidence.proof)?;
    proof.equalities.extend(context.region_equalities);
    assert!(valid(&facts, &proof, 1)?);
    Ok(())
}

#[test]
fn correlated_slots_and_enclosing_captures_are_preserved_at_each_use() -> TestResult {
    for (count, second, expected) in [
        (2, RegionId::bound(0, 1)?, true),
        (1, RegionId::bound(0, 0)?, false),
        (1, RegionId(2), true),
        (1, RegionId(1), false),
    ] {
        let (report, mut facts) = tests::declarations("trait A<'a,'b>{}fn caller<T>(){}")?;
        let ty = parameter(&report, &facts, 0)?;
        let item = tests::definition(&report, "A")?;
        let value = predicate(
            &mut facts,
            item,
            ty,
            &[
                Argument::Lifetime(RegionId::bound(0, 0)?),
                Argument::Lifetime(second),
            ],
        )?;
        let clause = quantify(&mut facts, count, value)?;
        let mut assumptions = Assumptions::default();
        assumptions.insert_clause(&facts.types, clause)?;
        let goal = predicate(
            &mut facts,
            item,
            ty,
            &[
                Argument::Lifetime(RegionId(1)),
                Argument::Lifetime(RegionId(2)),
            ],
        )?;
        let mut proof = outer(2)?;
        select(&mut facts, &assumptions, goal, &mut proof)?;
        assert_eq!(valid(&facts, &proof, 2)?, expected, "{count}; {second:?}");
    }
    Ok(())
}

#[test]
fn reference_subjects_and_trait_inputs_share_one_instantiation() -> TestResult {
    for (requested, expected) in [(RegionId(1), true), (RegionId(2), false)] {
        let (report, mut facts) = tests::declarations("trait A<'a>{}fn caller<T>(){}")?;
        let ty = parameter(&report, &facts, 0)?;
        let item = tests::definition(&report, "A")?;
        let local = RegionId::bound(0, 0)?;
        let subject = facts.types.intern(Kind::Reference {
            mutable: false,
            region: local,
            referent: ty,
        })?;
        let value = predicate(&mut facts, item, subject, &[Argument::Lifetime(local)])?;
        let clause = quantify(&mut facts, 1, value)?;
        let mut assumptions = Assumptions::default();
        assumptions.insert_clause(&facts.types, clause)?;
        let subject = facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId(1),
            referent: ty,
        })?;
        let goal = predicate(&mut facts, item, subject, &[Argument::Lifetime(requested)])?;
        let mut proof = outer(2)?;
        select(&mut facts, &assumptions, goal, &mut proof)?;
        assert_eq!(valid(&facts, &proof, 2)?, expected);
    }
    Ok(())
}

#[test]
fn flat_shortcuts_and_ambiguous_trials_cannot_drop_quantified_candidates() -> TestResult {
    let (report, mut facts) = tests::declarations("trait A<'a>{}trait B{}fn caller<T,U>(){}")?;
    let ty = parameter(&report, &facts, 0)?;
    let foreign = parameter(&report, &facts, 1)?;
    let item = tests::definition(&report, "A")?;
    let ordinary = predicate(&mut facts, item, ty, &[Argument::Lifetime(RegionId(1))])?;
    let value = predicate(
        &mut facts,
        item,
        ty,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let clause = quantify(&mut facts, 1, value)?;
    let mut assumptions = Assumptions::default();
    assumptions.insert(&facts.types, ordinary)?;
    let Some(Kind::Parameter(parameter)) = facts.types.kind(ty) else {
        return Err("missing parameter".into());
    };
    assert_eq!(assumptions.parameter_traits(parameter).len(), 1);
    assumptions.insert_clause(&facts.types, clause)?;
    assert_eq!(assumptions.parameter_traits(parameter).len(), 1);
    assert!(assumptions.exact(&facts.types, ordinary)?.is_none());
    let (mut context, goal) = tests::instantiate(&mut facts, ordinary)?;
    assert!(
        assumptions
            .select(&mut facts.types, &mut context, &goal)
            .is_err()
    );
    assert!(
        assumptions
            .unambiguous_declaration(&mut facts.types, &mut context, ordinary)
            .is_err()
    );
    assert!(matches!(
        assumptions.select_scoped(&mut facts.types, &mut context, &goal, 2)?,
        ScopedSelection::Deferred
    ));
    assert!(context.region_equalities.is_empty());
    let foreign = predicate(
        &mut facts,
        item,
        foreign,
        &[Argument::Lifetime(RegionId(1))],
    )?;
    let (mut context, goal) = tests::instantiate(&mut facts, foreign)?;
    assert!(matches!(
        assumptions.select_scoped(&mut facts.types, &mut context, &goal, 2)?,
        ScopedSelection::Absent
    ));
    let value = predicate(&mut facts, tests::definition(&report, "B")?, ty, &[])?;
    let clause = quantify(&mut facts, 0, value)?;
    assumptions.insert_clause(&facts.types, clause)?;
    assert_eq!(assumptions.parameter_traits(parameter).len(), 2);
    let mut proof = outer(0)?;
    select(&mut facts, &assumptions, value, &mut proof)?;
    assert_eq!(proof.regions.end, 1);
    assert!(valid(&facts, &proof, 0)?);
    Ok(())
}

#[test]
fn implementation_conditions_retain_the_assumed_child_interval() -> TestResult {
    let (report, mut facts) = tests::declarations(
        "trait A<'a>{}trait B<'a>{}impl<'a,V:A<'a>>B<'a>for V{}fn caller<T>(){}",
    )?;
    let ty = parameter(&report, &facts, 0)?;
    let value = predicate(
        &mut facts,
        tests::definition(&report, "A")?,
        ty,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let clause = quantify(&mut facts, 1, value)?;
    let mut assumptions = Assumptions::default();
    assumptions.insert_clause(&facts.types, clause)?;
    let goal = predicate(
        &mut facts,
        tests::definition(&report, "B")?,
        ty,
        &[Argument::Lifetime(RegionId(1))],
    )?;
    let mut proof = outer(1)?;
    let environment = facts.generics.environments[&tests::definition(&report, "caller")?].clone();
    let (mut context, goal) = tests::instantiate(&mut facts, goal)?;
    let Selection::Proven(evidence) = selection::select_available(
        &mut facts,
        &mut context,
        &goal,
        Caller {
            assumptions: &assumptions,
            environment: Some(&environment),
            first_region: proof.regions.end,
            retain_sized: false,
        },
        Availability::Validated,
    )?
    else {
        return Err("missing assumed implementation condition".into());
    };
    assert!(matches!(evidence.source, Source::Implementation { .. }));
    proof.extend(evidence.proof)?;
    proof.equalities.append(&mut context.region_equalities);
    assert!(proof.regions.end >= 4);
    assert!(valid(&facts, &proof, 1)?);
    Ok(())
}

#[test]
fn competing_clauses_share_opening_work_for_growing_common_tails() -> TestResult {
    let mut previous = 0;
    for width in [64, 128, 256, 512] {
        let (report, mut facts) = tests::declarations("trait A<'a,X>{}fn caller<T>(){}")?;
        let ty = parameter(&report, &facts, 0)?;
        let item = tests::definition(&report, "A")?;
        let goal = predicate(
            &mut facts,
            item,
            ty,
            &[Argument::Lifetime(RegionId(1)), Argument::Type(BYTE)],
        )?;
        let mut assumptions = Assumptions::default();
        assumptions.insert(&facts.types, goal)?;
        let local = RegionId::bound(0, 0)?;
        let mut tail = BYTE;
        for _ in 0..width {
            tail = facts.types.intern(Kind::Reference {
                mutable: false,
                region: local,
                referent: tail,
            })?;
            let value = predicate(
                &mut facts,
                item,
                ty,
                &[Argument::Lifetime(local), Argument::Type(tail)],
            )?;
            let clause = quantify(&mut facts, 1, value)?;
            assumptions.insert_clause(&facts.types, clause)?;
        }
        let before = facts.types.len();
        let (mut context, goal) = tests::instantiate(&mut facts, goal)?;
        let ScopedSelection::Proven(evidence) =
            assumptions.select_scoped(&mut facts.types, &mut context, &goal, 2)?
        else {
            return Err("failed quantified alternatives hid the ordinary assumption".into());
        };
        assert!(matches!(evidence.clause, Clause::Predicate(_)));
        assert_eq!(evidence.regions, 2..2);
        assert!(context.region_equalities.is_empty());
        assert!(facts.types.len() - before <= width + 4);
        assert!(context.steps < width * 200, "{width}: {}", context.steps);
        if previous != 0 {
            assert!(
                context.steps <= previous * 2 + 64,
                "{width}: {previous} -> {}",
                context.steps
            );
        }
        previous = context.steps;
    }
    Ok(())
}
