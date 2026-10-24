//! Genuine provided originals retain exact defining facts, never execution authority.
use super::*;
use crate::body::templates::Template;
#[path = "direct_checks.rs"]
mod direct;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn original() -> Result<(Declarations, Template, DefId)> {
    named_original(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
        "clone_from",
    )
}

fn named_original(path: &str, name: &str) -> Result<(Declarations, Template, DefId)> {
    let source = crate::test_source::fixture(path)?;
    let report = crate::test_source::source_provider(&source, "defining_members")?;
    let crate::pipeline::Analysis {
        facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
        })
        .ok_or("actual clone_from")?;
    let definition = function.definition;
    let template = prepared.templates.function(&report, &facts, function)?;
    Ok((facts, template, definition))
}

fn statement(body: &Body) -> Result<usize> {
    body.statements.iter().position(|statement| {
        matches!(statement.action, Action::Evaluate(id) if matches!(body.values[id.0 as usize].kind, Rvalue::Call { .. }))
    }).ok_or_else(|| "actual abstract clone call".into())
}

#[test]
fn genuine_clone_defining_call_rejects_changed_site_value_and_result() -> Result {
    let (facts, template, definition) = original()?;
    let body = template.body;
    let index = statement(&body)?;
    let mut work = Work::new(1_000_000);
    let context = prepare(&facts, &body, definition, &mut work)?;
    check(&context, &facts, &body, &body.statements[index], &mut work)?;
    let mut changed = body.clone();
    changed.statements[index].source = changed.blocks.first().ok_or("block")?.source;
    assert!(
        check(
            &context,
            &facts,
            &changed,
            &changed.statements[index],
            &mut work
        )
        .is_err()
    );
    let Action::Evaluate(value) = body.statements[index].action else {
        return Err("statement".into());
    };
    let Rvalue::Call { callee, .. } = body.values[value.0 as usize].kind else {
        return Err("Call".into());
    };
    let mut changed = body.clone();
    changed.values[callee.0 as usize].kind = Rvalue::Function(definition);
    assert!(
        check(
            &context,
            &facts,
            &changed,
            &changed.statements[index],
            &mut work
        )
        .is_err()
    );
    let mut changed = body.clone();
    changed.values[value.0 as usize].ty = crate::types::TypeId::UNIT;
    assert!(
        check(
            &context,
            &facts,
            &changed,
            &changed.statements[index],
            &mut work
        )
        .is_err()
    );
    assert!(!facts.is_const_function(definition));
    Ok(())
}

#[test]
fn defining_call_work_counts_failures_and_reuses_one_local_context() -> Result {
    let (facts, template, definition) = original()?;
    let body = template.body;
    let index = statement(&body)?;
    let mut tiny = Work::new(1);
    assert!(prepare(&facts, &body, definition, &mut tiny).is_err());
    assert!(tiny.steps > 1);
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut work = Work::new(1_000_000);
        let context = prepare(&facts, &body, definition, &mut work)?;
        let setup = work.steps;
        for _ in 0..count {
            check(&context, &facts, &body, &body.statements[index], &mut work)?;
        }
        let calls = work.steps - setup;
        println!(
            "defining_calls={count} setup={setup} visits={calls} context={} input={}",
            size_of::<DefiningCalls<'_>>(),
            size_of::<Call>()
        );
        if let Some((width, visits)) = previous {
            assert_eq!(calls * width, visits * count);
        }
        previous = Some((count, calls));
    }
    Ok(())
}

#[test]
fn provided_function_ctfe_root_rejects_fresh_and_warm_code_even_with_forged_const_flag() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-default-effects/provider/pure-unused.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "provided_roots")?;
    let crate::pipeline::Analysis {
        mut facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let named = |name| {
        facts
            .functions
            .iter()
            .find(|function| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == name
            })
            .copied()
            .ok_or("function")
    };
    let provided = named("provided")?;
    let constant = named("constant")?;
    let body = prepared
        .templates
        .function(&report, &facts, &provided)?
        .body;
    let warm_body = prepared
        .templates
        .function(&report, &facts, &constant)?
        .body;
    assert!(!body.const_context);
    facts.constant_functions.insert(provided.definition);
    // These are forged CTFE entry attempts, not source-positive const execution.
    for warm in [false, true] {
        let mut catalog = crate::const_eval::code::Catalog::default();
        if warm {
            catalog.insert(&warm_body)?;
        }
        let index = catalog.insert(&body)?;
        catalog.definitions.insert(provided.definition, index);
        for direct in [false, true] {
            let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
            let mut machine = crate::const_eval::Machine::catalog(
                evaluator,
                &catalog,
                crate::const_eval::Limits::default(),
            );
            let result = if direct {
                machine.definition(provided.definition, &[]).map(|_| ())
            } else {
                machine.begin(index, &[])
            };
            let error = result
                .err()
                .ok_or("provided root acquired const permission")?;
            assert!(
                error
                    .message
                    .contains("const-trait default execution is not implemented"),
                "{}",
                error.message
            );
        }
    }
    Ok(())
}
