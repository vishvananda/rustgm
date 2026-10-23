//! Public interpreter roots do not acquire another reached call's conditional proof.
use super::*;
use crate::{
    body::ir::Body,
    const_eval::{self, code::Catalog},
    sema::model::Declarations,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn checked() -> TestResult<(crate::resolve::expansion::Report, Declarations, Vec<Body>)> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-const-callable/original-native29.rs"),
    )?;
    let report = crate::test_source::source_provider(&source, "function_roots")?;
    let targets = ["apply", "id"].map(|name| {
        report
            .resolution
            .definitions
            .iter()
            .position(|definition| report.package.names.get(definition.name) == name)
            .and_then(|index| u32::try_from(index).ok())
            .map(crate::resolve::model::DefId)
            .ok_or("actual helper target")
    });
    let [apply, id] = targets;
    let (crate::pipeline::Analysis { facts, .. }, bodies) =
        crate::pipeline::execution::materialized_bodies::during(&[apply?, id?], || {
            crate::pipeline::analyze_source(&report)
        })?;
    Ok((report, facts, bodies))
}
fn named<'a>(
    report: &crate::resolve::expansion::Report,
    bodies: &'a [Body],
    name: &str,
) -> TestResult<&'a Body> {
    bodies.iter().find(|body| matches!(body.owner, Owner::Function { definition, .. }
        if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == name))
        .ok_or_else(|| format!("missing {name}").into())
}
#[test]
fn conditional_function_roots_reject_fresh_and_warm_entry_before_execution() -> TestResult {
    let (report, mut facts, bodies) = checked()?;
    let apply = named(&report, &bodies, "apply")?;
    let item = named(&report, &bodies, "id")?;
    for warm in [false, true] {
        let mut catalog = Catalog::default();
        if warm {
            catalog.insert(item)?;
        }
        let index = catalog.insert(apply)?;
        let Owner::Function { definition, .. } = apply.owner else {
            return Err("function".into());
        };
        catalog.definitions.insert(definition, index);
        for direct in [false, true] {
            let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
            let mut machine =
                const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default());
            let result = if direct {
                machine.definition(definition, &[]).map(|_| ())
            } else {
                machine.begin(index, &[])
            };
            let error = result.err().ok_or("unpaid function root executed")?;
            assert!(
                error.message.contains("requires a reached caller payment"),
                "{}",
                error.message
            );
        }
    }
    Ok(())
}
#[test]
fn provider_roots_require_their_actual_prepared_catalog_and_ordinary_mode_needs_none() -> TestResult
{
    let (report, mut facts, bodies) = checked()?;
    let item = named(&report, &bodies, "id")?;
    let Owner::Function { definition, .. } = item.owner else {
        return Err("id".into());
    };
    let mut catalog = Catalog::default();
    let index = catalog.insert(item)?;
    catalog.definitions.insert(definition, index);
    let saved = facts.source_bindings.take().ok_or("catalog")?;
    let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
    assert!(
        const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default())
            .begin(index, &[])
            .is_err()
    );
    facts.source_bindings = crate::sema::compiler_items::Catalog::reserve(&report)?;
    let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
    assert!(
        const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default())
            .begin(index, &[])
            .is_err()
    );
    facts.source_bindings = Some(saved);
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-const-callable/original-native29.rs"),
    )?;
    let another = crate::test_source::source_provider(&source, "another_function_roots_provider")?;
    assert!(
        crate::sema::values::Evaluator::new(&another, &mut facts)
            .check_source_call_policy()
            .is_err()
    );
    let argument = crate::sema::values::Constant::scalar(crate::sema::model::Scalar::Integer(29));
    let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
    let result = const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default())
        .definition(definition, &[argument])
        .map_err(|e| e.message)?;
    assert_eq!(result.as_scalar()?, crate::sema::model::Scalar::Integer(29));
    let ordinary = crate::test_source::resolve(
        "const fn value()->u8{7}#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{value()}",
        true,
    )?;
    assert!(
        crate::sema::values::Evaluator::new(&ordinary, &mut facts)
            .check_source_call_policy()
            .is_err()
    );
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&ordinary)?;
    assert!(facts.source_bindings.is_none());
    let body = crate::lower::instances::inspect_prepared(
        &ordinary,
        &mut facts,
        prepared,
        |_, checked| named(&ordinary, &checked.program().bodies, "value").cloned(),
    )??;
    let Owner::Function { definition, .. } = body.owner else {
        return Err("value owner".into());
    };
    let mut catalog = Catalog::default();
    let index = catalog.insert(&body)?;
    catalog.definitions.insert(definition, index);
    let evaluator = crate::sema::values::Evaluator::new(&ordinary, &mut facts);
    let result = const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default())
        .definition(definition, &[])
        .map_err(|e| e.message)?;
    assert_eq!(result.as_scalar()?, crate::sema::model::Scalar::Integer(7));
    Ok(())
}
