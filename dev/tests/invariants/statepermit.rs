//! A generated descriptor cannot substitute another body, state place or source arena.
use super::*;
use crate::{
    body::ir::{
        Action, BlockId, Body, LocalId, Owner, Place, ProjectionKind, Rvalue, Terminator, ValueId,
    },
    pipeline, sema, test_source,
    types::{self, Kind, TypeId},
};
use crate::async_checks as checks;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn drop_permit_authenticates_the_full_generated_state_graph_and_current_arena() -> Result {
    let text = test_source::fixture("pa43/tests/spec/future-runtime-v1/construction-source.rs")?;
    let report = test_source::source_provider(&text, "drop_permit")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    pipeline::checked_future_storage(&mut facts, &checked)?;
    let future = checks::future(&report, &mut facts, "never", &[])?;
    let mut other = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let other_future = checks::future(&report, &mut other, "never", &[])?;
    assert_eq!(facts.types.kind(future), other.types.kind(other_future));
    for case in 0..11 {
        let Constructed {
            mut body,
            contract,
            permit,
        } = pipeline::future_drop(&mut facts, &checked, future)?;
        for _ in 0..8 {
            permit.require_current(&facts.types, &body)?;
            assert!(permit.require_current(&other.types, &body).is_err());
            assert!(
                require_ordinary_body(&body).is_err(),
                "descriptor supplied ordinary proof"
            );
        }
        assert!(permit.instance().is_none());
        assert_eq!(
            permit.entry().map(|key| key.operation),
            Some(types::future::Operation::Drop)
        );
        assert_eq!(contract.key, body.key());
        match case {
            0 => body.result = TypeId::primitive(types::Primitive::U8),
            1 => body.projections[1].kind = ProjectionKind::FutureField(1),
            2 => body.projections[1].base = Place::Local(LocalId(0)),
            3 => body.values[0].kind = Rvalue::Read(Place::Local(LocalId(0))),
            4 => body.scalars[1] = sema::model::Scalar::Integer(1),
            5 => body.statements[1].action = Action::Evaluate(ValueId(0)),
            6 => body.blocks[4].terminator = Terminator::Return(ValueId(5)),
            7 => body.blocks[0].terminator = Terminator::Jump(BlockId(1)),
            8 => {
                let Owner::FutureEntry {
                    mut key,
                    signature,
                    execution,
                } = body.owner
                else {
                    return Err("actual generated owner".into());
                };
                key.operation = types::future::Operation::Poll;
                body.owner = Owner::FutureEntry {
                    key,
                    signature,
                    execution,
                };
            }
            9 => body.const_context = true,
            10 => body.storage.as_mut().ok_or("actual storage")?.exits.clear(),
            _ => return Err("invalid test case".into()),
        }
        assert!(
            permit.require_current(&facts.types, &body).is_err(),
            "forgery {case}"
        );
    }
    assert_eq!(size_of::<Permit>(), 40);
    assert_eq!(size_of::<crate::borrow::copy_view::CopyView<'_>>(), 32);
    assert_eq!(size_of::<Owner>(), 20);
    assert_eq!(size_of::<Body>(), 456);
    Ok(())
}

#[test]
fn nonempty_actual_capture_families_cannot_use_the_empty_state_drop_profile() -> Result {
    let text = test_source::fixture("pa43/tests/spec/future-runtime-v1/source-shape.rs")?;
    let report = test_source::source_provider(&text, "captured_drop_profile")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    pipeline::checked_future_storage(&mut facts, &checked)?;
    let future = checks::future(
        &report,
        &mut facts,
        "generic",
        &[types::arguments::Argument::Type(TypeId::primitive(
            types::Primitive::U8,
        ))],
    )?;
    assert!(matches!(
        facts.types.kind(future),
        Some(Kind::Future { .. })
    ));
    let error = pipeline::future_drop(&mut facts, &checked, future)
        .err()
        .ok_or("captured family acquired an empty cleanup implementation")?;
    assert!(error.contains("capture"), "{error}");
    Ok(())
}
