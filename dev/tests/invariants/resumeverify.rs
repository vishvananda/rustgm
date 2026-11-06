//! Only the actual proved source family authorizes its Pin/state/Ready graph.
use crate::{
    body::{
        coroutines,
        ir::{
            Action, BlockId, Body, Owner, Place, ProjectionId, ProjectionKind, Rvalue, Terminator,
            ValueId,
        },
    },
    borrow::incremental::Bridge,
    pipeline, test_source,
    types::{self, TypeId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn actual_poll_permit_rejects_forged_state_edges_members_and_foreign_store() -> Result {
    let text = test_source::fixture("pa43/tests/spec/future-runtime-v1/pollready-source.rs")?;
    let report = test_source::source_provider(&text, "poll_permit_graph")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let other = pipeline::analyze_source(&report)?.facts;
    let observe = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "observe"
        })
        .ok_or("actual observer declaration")?
        .definition;
    super::super::inspect_roots(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[observe],
        },
        prepared,
        |facts, assembled, _| -> Result {
            let (mut program, _, forwarding) = assembled.into_lowering();
            let index=program.bodies.iter().position(|body|matches!(body.owner,Owner::FutureEntry{key,..} if key.operation==types::future::Operation::Poll))
                .ok_or("actual retained poll body")?;
            let Some(Bridge::Factory(factory)) = forwarding.get(&index) else {
                return Err("actual poll lost its moved generated-family permit".into());
            };
            let permit = factory.future().ok_or("actual future permit")?;
            let body = &mut program.bodies[index];
            for _ in 0..8 {
                permit.require_current(&facts.types, body)?;
                assert!(permit.require_current(&other.types, body).is_err());
                assert!(coroutines::require_ordinary_body(body).is_err());
                assert!(!permit.state_read(
                    &facts.types,
                    body,
                    Place::Projection(ProjectionId(0))
                )?);
                assert!(permit.state_read(
                    &facts.types,
                    body,
                    Place::Projection(ProjectionId(2))
                )?);
            }
            reject_forged_graph(&facts.types, body, permit)?;
            assert_eq!(size_of::<coroutines::Permit>(), 48);
            assert_eq!(size_of::<Owner>(), 20);
            assert_eq!(size_of::<Body>(), 456);
            Ok(())
        },
    )??;
    Ok(())
}

fn reject_forged_graph(
    types: &types::Store,
    body: &mut Body,
    permit: &coroutines::Permit,
) -> Result {
    let pointer = body.projections[0];
    body.projections[0].kind = ProjectionKind::Field(crate::sema::members::Selection::Tuple(0));
    assert!(permit.require_current(types, body).is_err());
    body.projections[0] = pointer;
    let state = body.projections[2];
    body.projections[2].kind = ProjectionKind::FutureField(1);
    assert!(permit.require_current(types, body).is_err());
    body.projections[2] = state;
    let ready = body.values[6];
    body.values[6].kind = Rvalue::Unit;
    assert!(permit.require_current(types, body).is_err());
    body.values[6] = ready;
    let poison = body.statements[10];
    body.statements[10].action = Action::Evaluate(ValueId(0));
    assert!(permit.require_current(types, body).is_err());
    body.statements[10] = poison;
    let complete = body.statements[13];
    body.statements[13].action = Action::Store {
        place: Place::Projection(ProjectionId(2)),
        value: ValueId(4),
    };
    assert!(permit.require_current(types, body).is_err());
    body.statements[13] = complete;
    let branch = body.blocks[0].terminator;
    body.blocks[0].terminator = Terminator::Jump(BlockId(1));
    assert!(permit.require_current(types, body).is_err());
    body.blocks[0].terminator = branch;
    let output = body.result;
    body.result = TypeId::UNIT;
    assert!(permit.require_current(types, body).is_err());
    body.result = output;
    let instance = body.instance.take();
    assert!(permit.require_current(types, body).is_err());
    body.instance = instance;
    let storage = body.storage.take().ok_or("actual poll storage")?;
    assert!(permit.require_current(types, body).is_err());
    body.storage = Some(storage);
    permit.require_current(types, body)?;
    Ok(())
}
