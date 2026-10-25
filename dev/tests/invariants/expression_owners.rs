//! Completed anonymous values cannot replace missing or corrupted canonical roots.
use super::*;
use crate::sema::values::execution::Request;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn check_completed(name: &str) -> TestResult {
    let source =
        crate::test_source::fixture(&format!("pa31/tests/spec/const-eval-select-ctfe/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "completed_anonymous_root")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let context = prepared.constants.as_ref().ok_or("context")?;
    let mut engine = context.engine.try_borrow_mut()?;
    let (target, index) = engine
        .roots
        .iter()
        .find_map(|(&target, &index)| {
            matches!(target, Target::Expression { .. }).then_some((target, index))
        })
        .ok_or("actual anonymous root")?;
    let request = Request {
        target,
        arguments: Vec::new(),
    };
    let value = *engine
        .completed
        .get(&request)
        .ok_or("actual completed expression request")?;
    let before = (
        engine.construction.work(),
        engine.catalog.bytes,
        engine
            .const_calls
            .as_deref()
            .ok_or("Inputs")?
            .retained_bytes(),
    );
    engine.roots.remove(&target);
    let error = engine
        .advance(&report, &mut facts, &request)
        .err()
        .ok_or("missing root admitted cached value")?;
    assert!(
        error
            .to_string()
            .contains("completed expression lost its canonical Code root")
    );
    let other = *engine
        .catalog
        .functions
        .values()
        .next()
        .ok_or("actual F Code")?;
    engine.roots.insert(target, other);
    assert!(engine.advance(&report, &mut facts, &request).is_err());
    engine.roots.insert(target, index);
    let Target::Expression { source, result, .. } = target else {
        return Err("expression".into());
    };
    let crate::body::ir::Owner::Function { definition, .. } =
        engine.catalog.code(other).ok_or("F")?.owner
    else {
        return Err("F owner".into());
    };
    let wrong = Target::Expression {
        source,
        result,
        context: Some(crate::sema::values::Instance {
            definition,
            arguments: crate::types::GenericArgs::EMPTY,
        }),
    };
    let wrong_request = Request {
        target: wrong,
        arguments: Vec::new(),
    };
    engine.roots.insert(wrong, index);
    engine.completed.insert(wrong_request.clone(), value);
    assert!(engine.root(&report, &mut facts, wrong).is_err());
    assert!(engine.advance(&report, &mut facts, &wrong_request).is_err());
    engine.roots.remove(&wrong);
    engine.completed.remove(&wrong_request);
    assert!(matches!(
        engine.advance(&report, &mut facts, &request)?,
        crate::sema::values::execution::Progress::Complete(_)
    ));
    assert_eq!(
        before,
        (
            engine.construction.work(),
            engine.catalog.bytes,
            engine
                .const_calls
                .as_deref()
                .ok_or("Inputs")?
                .retained_bytes()
        )
    );
    Ok(())
}

#[test]
fn completed_anonymous_requests_reject_missing_stale_and_contextual_root_entries() -> TestResult {
    for name in ["anonymous-empty-selector", "reference-anonymous-selector"] {
        check_completed(name)?;
    }
    Ok(())
}
