//! Closed source member requests retain exact typed facts; executable entry is separate.
use super::*;
#[test]
fn closed_const_member_requests_reject_forged_owner_site_signature_and_selection() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-add-sub/macro-const-with-location.rs")?;
    let report = crate::test_source::source_provider(&source, "const_member_initializers")?;
    let crate::pipeline::Analysis {
        facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let initializer = facts
        .initializers
        .iter()
        .find(|i| {
            report
                .package
                .names
                .get(report.resolution.definitions[i.definition.0 as usize].name)
                == "SHARED"
        })
        .ok_or("actual SHARED initializer")?;
    let body = prepared
        .templates
        .initializer(&report, &facts, initializer)?
        .body;
    let different = facts
        .initializers
        .iter()
        .find(|i| {
            report
                .package
                .names
                .get(report.resolution.definitions[i.definition.0 as usize].name)
                == "SUM"
        })
        .ok_or("different actual initializer")?;
    let index = statement(&body)?;
    let mut work = Work::new(1_000_000);
    assert!(closed_request(
        &report,
        &facts,
        &body,
        &body.statements[index],
        &mut work
    )?);
    let Action::Evaluate(id) = body.statements[index].action else {
        return Err("statement".into());
    };
    let Rvalue::Call {
        callee,
        signature: _,
        arguments,
    } = body.values[id.0 as usize].kind
    else {
        return Err("Call".into());
    };
    for mutation in 0..5 {
        let mut changed = body.clone();
        match mutation {
            0 => changed.values[id.0 as usize].ty = crate::types::TypeId::UNIT,
            1 => {
                changed.values[id.0 as usize].kind = Rvalue::Call {
                    callee,
                    signature: crate::types::TypeId::UNIT,
                    arguments,
                }
            }
            2 => changed.values[callee.0 as usize].kind = Rvalue::Function(initializer.definition),
            3 => changed.statements[index].source = different.semantic_source().node.into(),
            _ => {
                changed.owner = Owner::Initializer {
                    definition: different.definition,
                    kind: different.kind,
                }
            }
        }
        assert!(
            closed_request(
                &report,
                &facts,
                &changed,
                &changed.statements[index],
                &mut work
            )
            .is_err(),
            "changed actual owner/site/signature/value {mutation} was accepted"
        );
    }
    let mut tiny = Work::new(1);
    assert!(closed_request(&report, &facts, &body, &body.statements[index], &mut tiny).is_err());
    assert!(tiny.steps > 1);
    Ok(())
}
#[test]
fn runtime_member_calls_cannot_acquire_const_permission_from_a_body_flag() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-add-sub/explicit-shared-bodies.rs")?;
    let report = crate::test_source::source_provider(&source, "runtime_member_flags")?;
    let crate::pipeline::Analysis {
        facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "explicit_add"
        })
        .ok_or("function")?;
    let template = prepared.templates.function(&report, &facts, function)?;
    let mut body = template.body;
    body.const_context = true;
    let index = statement(&body)?;
    assert!(!closed_request(
        &report,
        &facts,
        &body,
        &body.statements[index],
        &mut Work::new(1_000_000)
    )?);
    Ok(())
}
