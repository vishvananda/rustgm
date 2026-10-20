//! Actual suspended selection exports older input witnesses without dropping universal debts.
#[path = "universal_inference/binders.rs"]
mod binders;
#[path = "universal_inference/calls.rs"]
mod calls;
#[path = "universal_inference/scaling.rs"]
mod scaling;
use super::{Context, Declarations, Goal, Selection, header_tests, suspension, tests};
use crate::{
    resolve::model::DefId,
    sema::traits::{
        assumptions::Assumptions, implementations::Availability, selection::OwnedCaller,
    },
    types::{
        ItemId, Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        infer::{Domain, Instantiation, TermId, arguments::Argument as Inferred},
        lifetime_proof::Obligations,
        outlives::Queries,
        parameters::{Parameter, ParameterKind},
        region_graph,
        relations::Constraints,
    },
};
use std::sync::Arc;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn goal(facts: &mut Declarations, item: DefId) -> TestResult<(Context, Goal, TermId)> {
    let parameter = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = facts.types.intern(Kind::Parameter(parameter))?;
    let args = facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 0)?),
        Argument::Type(ty),
    ])?;
    let predicate = tests::predicate(facts, item, BYTE, args)?;
    let mut context = Context::default();
    let unknown = context.variable(Domain::Any)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &facts.types,
        &[(parameter, Inferred::Type(unknown))],
    )?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)?;
    Ok((context, goal, unknown))
}

fn begin(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
) -> TestResult<suspension::Progress> {
    Ok(suspension::begin_refining(
        facts,
        context,
        goal,
        OwnedCaller {
            assumptions: Arc::new(Assumptions::default()),
            environment: None,
            first_region: 2,
            retain_sized: false,
        },
        Availability::Declarations,
        Some(1),
    )?)
}

fn prove(
    facts: &Declarations,
    proof: &Obligations,
    captured: bool,
    permission: bool,
) -> TestResult<bool> {
    let mut constraints = Constraints::new(
        &facts.types,
        &facts.lifetimes.variances,
        u32::from(captured),
        100_000,
    )?;
    // Region 1 predates the query, even when this particular goal does not use it.
    constraints.variable(RegionId(1))?;
    if permission {
        constraints.assume(RegionId::bound(0, 0)?, RegionId::STATIC)?;
    }
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    proof.require(&mut constraints, &mut queries)?;
    match constraints.prove() {
        Ok(_) => Ok(true),
        Err(region_graph::Error::Unsatisfied { .. } | region_graph::Error::Verification { .. }) => {
            Ok(false)
        }
        Err(error) => Err(error.into()),
    }
}

fn reference(facts: &mut Declarations, region: RegionId) -> TestResult<TypeId> {
    Ok(facts.types.intern(Kind::Reference {
        mutable: false,
        region,
        referent: BYTE,
    })?)
}

#[test]
fn completed_universal_selection_preserves_the_six_qualified_lifetime_outcomes() -> TestResult {
    for (implementation, captured, permission, expected) in [
        ("impl<'a,'b>A<'a,&'b u8>for u8{}", false, false, true),
        ("impl<'a,'b:'a>A<'a,&'b u8>for u8{}", false, false, true),
        ("impl<'a:'b,'b>A<'a,&'b u8>for u8{}", false, false, false),
        ("impl<'a,'b:'a>A<'a,&'b u8>for u8{}", true, false, false),
        ("impl<'a,'b:'a>A<'a,&'b u8>for u8{}", true, true, true),
        (
            "impl<'a:'b,'b:'c,'c:'a>A<'a,(&'b u8,&'c u8)>for u8{}",
            false,
            false,
            false,
        ),
    ] {
        let (report, mut facts) =
            tests::declarations(&format!("trait A<'a,T>{{}}{implementation}"))?;
        let (mut context, goal, unknown) = goal(&mut facts, tests::definition(&report, "A")?)?;
        let suspension::Progress::Ready(Selection::Proven(mut evidence)) =
            begin(&mut facts, &mut context, &goal)?
        else {
            return Err(format!("universal query did not complete: {implementation}").into());
        };
        assert_eq!(evidence.proof.scope_count(), 1);
        let selected = context.resolved(&mut facts.types, unknown)?;
        if let Some(Kind::Reference { region, .. }) = facts.types.kind(selected) {
            // Alias 2 and private impl witnesses 3/4 cannot be caller inputs.
            assert_eq!(region, RegionId(5));
            assert_eq!(evidence.proof.regions.end, 6);
        }
        if captured {
            let ty = reference(&mut facts, RegionId(1))?;
            let input = context.known(&facts.types, ty)?;
            context.equate(&facts.types, unknown, input)?;
            evidence
                .proof
                .equalities
                .push((RegionId(1), RegionId::bound(0, 0)?));
        }
        evidence
            .proof
            .equalities
            .append(&mut context.region_equalities);
        assert_eq!(
            prove(&facts, &evidence.proof, captured, permission)?,
            expected,
            "{implementation}; captured={captured}; permission={permission}"
        );
    }
    Ok(())
}

#[test]
fn known_caller_lifetimes_keep_their_older_scope_without_allocating_exports() -> TestResult {
    for permission in [false, true] {
        let (report, mut facts) =
            tests::declarations("trait A<'a,T>{}impl<'a,'b:'a>A<'a,&'b u8>for u8{}")?;
        let (mut context, goal, unknown) = goal(&mut facts, tests::definition(&report, "A")?)?;
        let ty = reference(&mut facts, RegionId(1))?;
        let input = context.known(&facts.types, ty)?;
        context.equate(&facts.types, unknown, input)?;
        let suspension::Progress::Ready(Selection::Proven(mut evidence)) =
            begin(&mut facts, &mut context, &goal)?
        else {
            return Err("known quantified inputs did not complete".into());
        };
        assert_eq!(context.resolved(&mut facts.types, unknown)?, ty);
        assert_eq!(evidence.proof.regions.end, 5);
        evidence
            .proof
            .equalities
            .push((RegionId(1), RegionId::bound(0, 0)?));
        evidence
            .proof
            .equalities
            .append(&mut context.region_equalities);
        assert_eq!(
            prove(&facts, &evidence.proof, true, permission)?,
            permission
        );
    }
    Ok(())
}

#[test]
fn partial_universal_inputs_survive_header_waits_without_claiming_foreign_allocations() -> TestResult
{
    for (where_, expected) in [("", true), ("where 'a:'b", false)] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a,T>{{}}trait B{{}}impl B for u16{{}}impl<'a,'b,T:B>A<'a,(&'b u8,T)>for u8 {where_}{{}}"
        ))?;
        let (mut context, goal, unknown) = goal(&mut facts, tests::definition(&report, "A")?)?;
        let mut delayed = header_tests::delay(&mut facts, &[tests::definition(&report, "B")?])?;
        let suspension::Progress::Inferred { query } = begin(&mut facts, &mut context, &goal)?
        else {
            return Err("quantified input did not publish a partial tuple".into());
        };
        let reference = context.child(&facts.types, unknown, 0)?;
        let original = context.resolved(&mut facts.types, reference)?;
        assert!(matches!(
            facts.types.kind(original),
            Some(Kind::Reference {
                region: RegionId(5),
                ..
            })
        ));
        let inner = context.child(&facts.types, unknown, 1)?;
        let value = context.known(&facts.types, TypeId::primitive(Primitive::U16))?;
        context.equate(&facts.types, inner, value)?;
        let end = query.allocation_end();
        let suspension::Progress::Pending(owner, mut query) =
            query.resume(&mut facts, &mut context, &goal, end + 7)?
        else {
            return Err("quantified candidate did not wait for its exact condition header".into());
        };
        let steps = context.steps;
        for _ in 0..3 {
            let suspension::Progress::Pending(same, retained) =
                query.resume(&mut facts, &mut context, &goal, end + 7)?
            else {
                return Err("unprepared universal header stopped waiting".into());
            };
            assert_eq!(same, owner);
            assert_eq!(context.steps, steps);
            query = retained;
        }
        let header = delayed.pop().ok_or("missing delayed header")?;
        assert_eq!(
            owner,
            crate::sema::traits::selection::Dependency::Header(header.owner)
        );
        header_tests::publish(&mut facts, header)?;
        let suspension::Progress::Ready(Selection::Proven(mut evidence)) =
            query.resume(&mut facts, &mut context, &goal, end + 10)?
        else {
            return Err("retained universal candidate did not complete".into());
        };
        assert_eq!(context.resolved(&mut facts.types, reference)?, original);
        assert_eq!(evidence.proof.regions.end, end + 10);
        for id in end..end + 10 {
            assert!(!evidence.proof.owns_region(RegionId(id)));
        }
        evidence
            .proof
            .equalities
            .append(&mut context.region_equalities);
        assert_eq!(prove(&facts, &evidence.proof, false, false)?, expected);
    }
    Ok(())
}

#[test]
fn resumed_children_export_new_witnesses_without_replacing_existing_inputs() -> TestResult {
    for (bound, expected) in [("", true), ("where 'a:'c", false)] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a,T>{{}}trait B<'a,U>{{}}impl<'a,'c>B<'a,&'c u8>for u16 {bound}{{}}impl<'a,'b,V,W>A<'a,(&'b u8,V,W)>for u8 where V:B<'a,W>{{}}"
        ))?;
        let (mut context, goal, unknown) = goal(&mut facts, tests::definition(&report, "A")?)?;
        let suspension::Progress::Inferred { query } = begin(&mut facts, &mut context, &goal)?
        else {
            return Err("candidate did not publish its first private witness".into());
        };
        let first = context.child(&facts.types, unknown, 0)?;
        let original = context.resolved(&mut facts.types, first)?;
        let subject = context.child(&facts.types, unknown, 1)?;
        let last = context.child(&facts.types, unknown, 2)?;
        assert!(matches!(
            context.resolved(&mut facts.types, last),
            Err(crate::types::infer::Error::Unresolved)
        ));
        let value = context.known(&facts.types, TypeId::primitive(Primitive::U16))?;
        context.equate(&facts.types, subject, value)?;
        let end = query.allocation_end();
        let suspension::Progress::Ready(Selection::Proven(mut evidence)) =
            query.resume(&mut facts, &mut context, &goal, end + 17)?
        else {
            return Err("newly selected child did not complete the parent".into());
        };
        assert_eq!(context.resolved(&mut facts.types, first)?, original);
        let result = context.resolved(&mut facts.types, last)?;
        assert!(
            matches!(facts.types.kind(result), Some(Kind::Reference { region, .. }) if region == RegionId(end + 19))
        );
        assert_eq!(evidence.proof.regions.end, end + 20);
        for id in end..end + 17 {
            assert!(!evidence.proof.owns_region(RegionId(id)));
        }
        evidence
            .proof
            .equalities
            .append(&mut context.region_equalities);
        assert_eq!(prove(&facts, &evidence.proof, false, false)?, expected);
    }
    Ok(())
}
