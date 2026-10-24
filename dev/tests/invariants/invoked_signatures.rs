//! Entry consistency is separate from the reached caller's reference permission.
use super::*;
use crate::{
    body::ir::Owner,
    const_eval::{Limits, code::Catalog},
    pipeline::execution::materialized_bodies,
    sema::values::Evaluator,
    types::{TypeId, callable::Convention},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn actual_reference_callback_entry_keeps_both_signatures_result_and_full_instance() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/const-eval-select-ctfe/reference-binder.rs")?;
    let report = crate::test_source::source_provider(&source, "reference_entry_pair")?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "ct")
        .ok_or("ct")?;
    let definition = DefId(u32::try_from(index)?);
    let (analysis, bodies) =
        materialized_bodies::during(&[definition], || crate::pipeline::analyze_source(&report))?;
    let mut facts = analysis.facts;
    let original = bodies.first().ok_or("actual proved callback")?;
    let (public, _) = original.owner.signatures().ok_or("paired signatures")?;
    let target = Instance::new(&facts.types, facts.definitions[index].ok_or("full F")?)?;
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        ..
    }) = facts.types.kind(public)
    else {
        return Err("actual public signature".into());
    };
    assert_ne!(bound_regions, 0);
    let wrong_owned = facts.types.intern(Kind::Function {
        bound_regions,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    let wrong_binder = facts.types.intern(Kind::Function {
        bound_regions: bound_regions + 1,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    for opponent in 0..6 {
        let mut body = original.clone();
        match opponent {
            1 => body.owner = Owner::Function { definition, signature: TypeId::UNIT, execution: None },
            2 => body.owner = Owner::Function { definition, signature: public, execution: Some(wrong_owned) },
            3 => body.result = TypeId::UNIT,
            4 => body.instance = Some(Instance::new(&facts.types, facts.definitions.iter().flatten().copied().find(|&ty| matches!(facts.types.kind(ty), Some(Kind::FunctionItem { item, .. }) if item.0 != definition.0)).ok_or("different complete instance")?)?),
            5 => body.owner = Owner::Function { definition, signature: public, execution: Some(wrong_binder) },
            _ => {}
        }
        let mut catalog = Catalog::default();
        let code = catalog.insert(&body)?;
        catalog.functions.insert(target, code);
        let machine = Machine::catalog(
            Evaluator::new(&report, &mut facts),
            &catalog,
            Limits::default(),
        );
        let result = machine
            .validate_invoked_entry(code, target)
            .and_then(|()| machine.validate_selected_entry(code, target));
        if opponent == 0 {
            result?;
        } else {
            assert!(result.is_err(), "entry consistency opponent {opponent}");
        }
    }
    Ok(())
}
