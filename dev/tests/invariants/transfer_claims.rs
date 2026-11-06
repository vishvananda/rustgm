//! Cold conversion data cannot authorize another source, operation, instance or arena.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn original() -> Result<(Declarations, crate::body::ir::Program)> {
    let text = crate::test_source::fixture("pa31/tests/spec/nominal-coercions/values/native.rs")?;
    let report = crate::test_source::source_provider(&text, "conversion_claims")?;
    let crate::pipeline::Analysis { facts, prepared } = crate::pipeline::analyze_source(&report)?;
    // Generic originals remain templates; consume any concrete CFGs already
    // retained by source preparation instead of rebuilding the whole fixture.
    let program = crate::body::concrete_selected(
        &report,
        &facts,
        |owner| crate::body::templates::owns(&facts, owner),
        prepared.concrete,
        crate::body::Selection::Complete,
    )?;
    Ok((facts, program))
}

#[test]
fn nominal_rows_reject_foreign_source_body_instance_operand_and_store() -> Result {
    let (facts, mut program) = original()?;
    let (foreign, _) = original()?;
    let index = program
        .bodies
        .iter()
        .position(|body| !body.nominal_uses().is_empty())
        .ok_or("actual nominal body")?;
    let alternate = program
        .bodies
        .iter()
        .find(|body| body.key() != program.bodies[index].key())
        .ok_or("another actual body")?;
    let other_key = alternate.key();
    let other_source = alternate.source;
    let body = &mut program.bodies[index];
    let saved = body
        .resolved_callables
        .as_ref()
        .and_then(|inputs| inputs.nominal.as_deref())
        .ok_or("actual nominal input owner")?
        .clone();
    let operation = saved.uses[0].operation;
    for ty in [saved.uses[0].from, saved.uses[0].to] {
        assert_eq!(facts.types.kind(ty), foreign.types.kind(ty));
    }
    let other_site = facts
        .unsizing_adjustments
        .keys()
        .find(|site| **site != saved.uses[0].source)
        .copied()
        .ok_or("another actual nominal site")?;
    let callable = body
        .values
        .iter()
        .find_map(|value| Instance::new(&facts.types, value.ty).ok())
        .ok_or("actual source callable")?;
    for _ in 0..8 {
        body.validate_nominal_inputs(&facts)?;
        body.nominal_use(&facts.types, operation)?;
        assert!(body.nominal_use(&foreign.types, operation).is_err());
    }
    for case in 0..11 {
        let inputs = body
            .resolved_callables
            .as_deref_mut()
            .and_then(|inputs| inputs.nominal.as_deref_mut())
            .ok_or("nominal inputs")?;
        match case {
            0 => inputs.key = other_key,
            1 => inputs.source = other_source,
            2 => inputs.instance = Some(callable),
            3 => {
                inputs.arena = foreign
                    .types
                    .existing_arena_identity()
                    .ok_or("foreign arena")?
                    .clone();
            }
            4 => inputs.uses[0].source = other_site,
            5 => inputs.uses[0].operation = inputs.uses[0].operand,
            6 => inputs.uses[0].operand = inputs.uses[0].operation,
            7 => inputs.uses[0].from = inputs.uses[0].to,
            8 => inputs.uses[0].to = inputs.uses[0].from,
            9 => inputs.uses.push(inputs.uses[0]),
            10 => inputs.uses.clear(),
            _ => return Err("invalid forgery case".into()),
        }
        assert!(
            body.validate_nominal_inputs(&facts).is_err(),
            "forgery {case}"
        );
        *body
            .resolved_callables
            .as_deref_mut()
            .and_then(|inputs| inputs.nominal.as_deref_mut())
            .ok_or("retained nominal inputs")? = saved.clone();
    }
    let statement = body.statements.iter().position(|statement|
        matches!(statement.action, crate::body::ir::Action::Evaluate(id) if id == operation))
        .ok_or("actual Evaluate")?;
    let source = body.statements[statement].source;
    body.statements[statement].source = other_site.node.into();
    assert!(body.validate_nominal_inputs(&facts).is_err());
    body.statements[statement].source = source;
    body.validate_nominal_inputs(&facts)?;
    Ok(())
}
