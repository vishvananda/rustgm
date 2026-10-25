//! Anonymous selectors retain their actual expression owner and proved Row.
use super::*;
use crate::{
    resolve::{
        expansion::Report,
        unit::{FileId, NodeKey},
    },
    sema::values::execution::{Request, Target},
    syntax::ast::{Kind as Syntax, NodeId},
    types::{GenericArgs, TypeId},
};

fn fixture(name: &str) -> TestResult<(Report, crate::pipeline::Analysis, Target)> {
    let source =
        crate::test_source::fixture(&format!("pa31/tests/spec/const-eval-select-ctfe/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "anonymous_scalar_owner")?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let context = analysis
        .prepared
        .constants
        .as_ref()
        .ok_or("source context")?;
    let mut selected = Vec::new();
    for (index, node) in report
        .package
        .unit(FileId(0))
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
    {
        if !matches!(node.kind, Syntax::Call { .. } | Syntax::Cast { .. }) {
            continue;
        }
        let source = NodeKey {
            file: FileId(0),
            node: NodeId::new(u32::try_from(index)?),
        };
        let Some(result) = analysis
            .facts
            .node(source)
            .and_then(|n| n.adjustment.map(|a| a.target).or(n.ty))
        else {
            continue;
        };
        let target = Target::Expression {
            source,
            result,
            context: None,
        };
        if context.inspect_const_call(target, |_, _, _| ()).is_ok() {
            selected.push(target);
        }
    }
    let [target] = selected.as_slice() else {
        return Err("one actual anonymous constant selector root".into());
    };
    let target = *target;
    Ok((report, analysis, target))
}

fn identity(
    context: &crate::pipeline::execution::Context,
    facts: &Declarations,
    target: Target,
) -> TestResult<(
    usize,
    *const Code,
    *const super::super::super::use_inputs::Row,
    usize,
    usize,
)> {
    let Target::Expression { source, result, .. } = target else {
        return Err("expression target".into());
    };
    context.inspect_const_call(target, |catalog, inputs, index| -> TestResult<_> {
        let code = catalog.expression(facts, index, source, result)?;
        let row = inputs.checked(catalog, index, code)?;
        assert!(matches!(code.owner, Owner::AnonymousConstant));
        assert_eq!(
            row.contract().key,
            crate::body::ir::BodyKey::Anonymous(source)
        );
        assert!(!catalog.definitions.values().any(|&named| named == index));
        Ok((
            index,
            std::ptr::from_ref(code),
            std::rc::Rc::as_ptr(row),
            catalog.bytes,
            inputs.retained_bytes(),
        ))
    })?
}

fn check_anonymous_scalar_warm_queries_reuse_exact_code_row_and_completed_proof(
    name: &str,
) -> TestResult {
    let (report, mut analysis, target) = fixture(name)?;
    assert!(analysis.facts.array_lengths.values().any(|&n| n == 7));
    let context = analysis
        .prepared
        .constants
        .as_ref()
        .ok_or("context")?
        .clone();
    let before = identity(&context, &analysis.facts, target)?;
    let work = (
        analysis.facts.body_work.cfg_functions.get(),
        analysis.facts.body_work.cfg_expressions.get(),
        analysis.facts.body_work.steps,
        analysis.facts.body_work.canonical_steps,
    );
    let mut executor = context.executor();
    for _ in 0..2 {
        executor.check(&report, &mut analysis.facts, target)?;
        let Progress::Complete(value) = executor.advance(
            &report,
            &mut analysis.facts,
            &Request {
                target,
                arguments: Vec::new(),
            },
        )?
        else {
            return Err("warm scalar constant did not complete".into());
        };
        assert_eq!(value.as_scalar()?, crate::sema::model::Scalar::Integer(7));
    }
    assert_eq!(before, identity(&context, &analysis.facts, target)?);
    assert_eq!(
        work,
        (
            analysis.facts.body_work.cfg_functions.get(),
            analysis.facts.body_work.cfg_expressions.get(),
            analysis.facts.body_work.steps,
            analysis.facts.body_work.canonical_steps
        )
    );
    Ok(())
}

fn check_anonymous_scalar_canonical_roots_reject_wrong_source_result_context_and_stale_types(
    name: &str,
) -> TestResult {
    let (report, mut analysis, target) = fixture(name)?;
    let Target::Expression { source, result, .. } = target else {
        return Err("expression".into());
    };
    let context = analysis
        .prepared
        .constants
        .as_ref()
        .ok_or("context")?
        .clone();
    let callback = context.inspect_const_call(target, |catalog, _, index| -> TestResult<_> {
        let function = *catalog.functions.values().next().ok_or("actual F")?;
        let body = catalog.code(function).ok_or("F Code")?;
        let other = body.source.source().ok_or("F source")?;
        for _ in 0..2 {
            assert!(
                catalog
                    .expression(&analysis.facts, index, other, result)
                    .is_err()
            );
            assert!(
                catalog
                    .expression(&analysis.facts, index, source, TypeId::UNIT)
                    .is_err()
            );
            assert!(
                catalog
                    .expression(&analysis.facts, function, source, result)
                    .is_err()
            );
            assert!(
                catalog
                    .expression(&analysis.facts, usize::MAX, source, result)
                    .is_err()
            );
        }
        let Owner::Function { definition, .. } = body.owner else {
            return Err("actual F owner".into());
        };
        Ok(definition)
    })??;
    let mut executor = context.executor();
    let wrong = Target::Expression {
        source,
        result,
        context: Some(crate::sema::values::Instance {
            definition: callback,
            arguments: GenericArgs::EMPTY,
        }),
    };
    for _ in 0..2 {
        assert!(executor.check(&report, &mut analysis.facts, wrong).is_err());
    }
    let node = &mut analysis.facts.nodes[source.file.0 as usize][source.node.get() as usize];
    let saved = *node;
    node.ty = Some(TypeId::UNIT);
    node.adjustment = None;
    assert!(
        executor
            .check(&report, &mut analysis.facts, target)
            .is_err()
    );
    assert!(
        executor
            .advance(
                &report,
                &mut analysis.facts,
                &Request {
                    target,
                    arguments: Vec::new()
                }
            )
            .is_err()
    );
    analysis.facts.nodes[source.file.0 as usize][source.node.get() as usize] = saved;
    executor.check(&report, &mut analysis.facts, target)?;
    Ok(())
}

fn check_anonymous_scalar_rows_require_the_actual_catalog_code_and_inputs(
    name: &str,
) -> TestResult {
    let (report, mut first, target) = fixture(name)?;
    let (_, second, other) = fixture(name)?;
    let context = first
        .prepared
        .constants
        .as_ref()
        .ok_or("first context")?
        .clone();
    let other_context = second.prepared.constants.as_ref().ok_or("second context")?;
    context.inspect_const_call(target, |catalog, inputs, index| -> TestResult {
        let code = catalog.code(index).ok_or("Code")?;
        inputs.checked(catalog, index, code)?;
        assert!(
            super::super::super::use_inputs::Inputs::default()
                .checked(catalog, index, code)
                .is_err()
        );
        other_context.inspect_const_call(
            other,
            |foreign, foreign_inputs, other_index| -> TestResult {
                let other_code = foreign.code(other_index).ok_or("foreign Code")?;
                assert_eq!(code.source, other_code.source);
                assert_eq!(code.result, other_code.result);
                assert!(!std::ptr::eq(code, other_code));
                assert!(inputs.checked(foreign, other_index, other_code).is_err());
                assert!(foreign_inputs.checked(catalog, index, code).is_err());
                assert!(inputs.checked(catalog, index, other_code).is_err());
                Ok(())
            },
        )??;
        let mut machine = Machine::catalog(
            Evaluator::new(&report, &mut first.facts),
            catalog,
            Limits::default(),
        );
        machine.begin(index, &[]).map_err(|e| e.message)?;
        let error = machine
            .advance()
            .err()
            .ok_or("missing Inputs admitted anonymous selector")?;
        assert!(
            error
                .message
                .contains(if name == "reference-anonymous-selector" {
                    "source const_eval_select callback binders require proved caller inputs"
                } else {
                    "anonymous selector requires its actual proved caller inputs"
                })
        );
        Ok(())
    })??;
    Ok(())
}

#[test]
fn anonymous_scalar_warm_queries_reuse_exact_code_row_and_completed_proof() -> TestResult {
    for name in ["anonymous-empty-selector", "reference-anonymous-selector"] {
        check_anonymous_scalar_warm_queries_reuse_exact_code_row_and_completed_proof(name)?;
    }
    Ok(())
}

#[test]
fn anonymous_scalar_canonical_roots_reject_wrong_source_result_context_and_stale_types()
-> TestResult {
    for name in ["anonymous-empty-selector", "reference-anonymous-selector"] {
        check_anonymous_scalar_canonical_roots_reject_wrong_source_result_context_and_stale_types(
            name,
        )?;
    }
    Ok(())
}

#[test]
fn anonymous_scalar_rows_require_the_actual_catalog_code_and_inputs() -> TestResult {
    for name in ["anonymous-empty-selector", "reference-anonymous-selector"] {
        check_anonymous_scalar_rows_require_the_actual_catalog_code_and_inputs(name)?;
    }
    Ok(())
}
