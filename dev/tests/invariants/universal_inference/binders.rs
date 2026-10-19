//! Retained parent queries preserve complete binder ancestry and foreign allocation gaps.
use super::{
    Availability, BYTE, Context, Goal, Inferred, Kind, OwnedCaller, RegionId, Selection,
    TestResult, prove, suspension, tests,
};
use crate::{
    sema::traits::assumptions::Assumptions,
    types::{ItemId, infer::Domain},
};
use std::sync::Arc;

fn caller() -> OwnedCaller {
    OwnedCaller {
        assumptions: Arc::new(Assumptions::default()),
        environment: None,
        first_region: 2,
        retain_sized: false,
    }
}

fn input(
    facts: &mut super::Declarations,
    context: &mut Context,
    item: ItemId,
    regions: &[RegionId],
    actual: crate::types::infer::TermId,
) -> TestResult<Goal> {
    use crate::types::{
        arguments::Argument,
        infer::Instantiation,
        parameters::{Parameter, ParameterKind},
    };
    let parameter = facts.types.intern_parameter(Parameter {
        owner: ItemId(1_000_000),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = facts.types.intern(Kind::Parameter(parameter))?;
    let mut arguments: Vec<_> = regions.iter().copied().map(Argument::Lifetime).collect();
    arguments.push(Argument::Type(ty));
    let arguments = facts.types.intern_arguments(&arguments)?;
    let predicate = tests::predicate(facts, crate::resolve::model::DefId(item.0), BYTE, arguments)?;
    let mut fields = Instantiation::new(
        context,
        &facts.types,
        &[(parameter, Inferred::Type(actual))],
    )?;
    Ok(context.instantiate_predicate(&mut facts.types, predicate, &mut fields)?)
}

#[test]
fn nested_partial_parents_keep_private_witnesses_and_scope_correlations() -> TestResult {
    for counts in [vec![1, 1], vec![1, 0, 1]] {
        for (head, bound, accepted) in [
            ("'a,'b", "", true),
            ("'a,'b", "where 'w:'b", true),
            ("'a,'b", "where 'b:'w", false),
            ("'a,'a", "", false),
        ] {
            let source = format!(
                "trait A<'a,'b,T>{{}}impl<'a,'b,'w,T:Copy>A<{head},(&'w u8,T)>for u8 {bound}{{}}"
            );
            let (report, mut facts) = tests::declarations(&source)?;
            let item = ItemId(tests::definition(&report, "A")?.0);
            let mut context = Context::default();
            let unknown = context.variable(Domain::Any)?;
            let subject = context.known(&facts.types, BYTE)?;
            let input = input(
                &mut facts,
                &mut context,
                item,
                &[
                    RegionId::bound(u16::try_from(counts.len() - 1)?, 0)?,
                    RegionId::bound(0, 0)?,
                ],
                unknown,
            )?;
            let suspension::Progress::Inferred { query } = suspension::begin_scoped_refining(
                &mut facts,
                &mut context,
                &input,
                caller(),
                Availability::Declarations,
                &counts,
            )?
            else {
                return Err("nested parent did not retain its unfinished Copy premise".into());
            };
            let end = query.allocation_end();
            let value = context.child(&facts.types, unknown, 1)?;
            context.equate(&facts.types, value, subject)?;
            let suspension::Progress::Ready(Selection::Proven(mut evidence)) =
                query.resume(&mut facts, &mut context, &input, end + 17)?
            else {
                return Err("nested parent did not finish after its input settled".into());
            };
            evidence
                .proof
                .equalities
                .append(&mut context.region_equalities);
            assert_eq!(evidence.proof.scope_count(), counts.len());
            for id in end..end + 17 {
                assert!(!evidence.proof.owns_region(RegionId(id)));
            }
            assert_eq!(
                prove(&facts, &evidence.proof, false, false)?,
                accepted,
                "{source}; {counts:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn opening_deep_wide_inputs_traverses_the_type_graph_once() -> TestResult {
    let mut observations = Vec::new();
    for width in [16_u16, 64, 256] {
        let (report, mut facts) = tests::declarations("trait A<T>{}impl<T>A<T>for u8{}")?;
        let item = ItemId(tests::definition(&report, "A")?.0);
        let mut fields = Vec::new();
        for depth in 0..width {
            fields.push(facts.types.intern(Kind::Reference {
                region: RegionId::bound(depth, 0)?,
                mutable: false,
                referent: BYTE,
            })?);
        }
        let fields = facts.types.intern_list(&fields)?;
        let ty = facts.types.intern(Kind::Tuple(fields))?;
        let mut context = Context::default();
        let argument = context.known(&facts.types, ty)?;
        let input = input(&mut facts, &mut context, item, &[], argument)?;
        let before = context.steps;
        let selected = suspension::begin_scoped_refining(
            &mut facts,
            &mut context,
            &input,
            caller(),
            Availability::Declarations,
            &vec![1; usize::from(width)],
        )?;
        assert!(matches!(
            selected,
            suspension::Progress::Ready(Selection::Proven(_))
        ));
        let work = context.steps - before;
        eprintln!("nested input width={width} work={work}");
        observations.push((usize::from(width), work));
    }
    for (width, work) in observations {
        assert!(
            work <= 300 * width + 512,
            "nested input width={width} work={work}"
        );
    }
    Ok(())
}
