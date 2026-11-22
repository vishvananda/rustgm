//! Public interpreter roots do not acquire another reached call's conditional proof.
use super::*;
use crate::{
    body::ir::Body,
    const_eval::{self, code::Catalog},
    sema::{
        model::Declarations,
        values::{PromotedFunction, PromotionCaller},
    },
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

fn promoted_generic() -> TestResult<(crate::resolve::expansion::Report, Declarations, Body)> {
    let source = crate::test_source::fixture("pa31/tests/spec/promoted-calls/generic-callee.rs")?;
    let report = crate::test_source::source_provider(&source, "promoted_generic_root")?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "word")
        .and_then(|index| u32::try_from(index).ok())
        .map(crate::resolve::model::DefId)
        .ok_or("generic marked word")?;
    let (facts, mut bodies) =
        crate::pipeline::execution::materialized_bodies::during(&[definition], || {
            let crate::pipeline::Analysis {
                mut facts,
                prepared,
            } = crate::pipeline::analyze_source(&report)?;
            crate::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                crate::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            Ok(facts)
        })?;
    let body = bodies.pop().ok_or("checked marked generic body")?;
    Ok((report, facts, body))
}

#[test]
fn generic_marked_promotions_do_not_open_ordinary_function_roots() -> TestResult {
    let (report, mut facts, body) = promoted_generic()?;
    let target = body.instance.ok_or("complete marked generic instance")?;
    let mut catalog = Catalog::default();
    let index = catalog.insert(&body)?;
    catalog.functions.insert(target, index);
    for warm in [false, true] {
        let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
        let mut machine =
            const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default());
        if warm {
            machine.begin_promoted(&report, index, PromotedFunction::unscoped(target), &[])?;
            let crate::sema::values::execution::Progress::Complete(value) =
                machine.advance().map_err(|error| error.message)?
            else {
                return Err("closed marked promotion did not finish".into());
            };
            assert_eq!(value.as_scalar()?, crate::sema::model::Scalar::Integer(7));
        }
        let error = machine
            .begin(index, &[])
            .err()
            .ok_or("ordinary generic root executed")?;
        assert!(
            error
                .message
                .contains("source generic function root execution without a reached caller"),
            "{}",
            error.message
        );
    }
    Ok(())
}

#[test]
fn promoted_roots_reject_open_unmarked_wrong_instances_and_argument_arity() -> TestResult {
    use crate::types::{Kind, arguments::Argument, instance::Instance};
    let (report, mut facts, body) = promoted_generic()?;
    let target = body.instance.ok_or("complete marked generic instance")?;
    let Kind::FunctionItem {
        item, signature, ..
    } = facts.types.kind(target.ty()).ok_or("function item")?
    else {
        return Err("function item".into());
    };
    let missing = Instance::new(
        &facts.types,
        facts.definitions[item.0 as usize].ok_or("open item")?,
    )?;
    let parameter = facts
        .generics
        .parameters_in_scope(crate::resolve::model::DefId(item.0))
        .next()
        .map(|parameter| parameter.argument)
        .ok_or("generic parameter")?;
    let open_arguments = facts.types.intern_arguments(&[parameter])?;
    let open_type = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments: open_arguments,
    })?;
    let open = Instance::new(&facts.types, open_type)?;
    let unmarked = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "unmarked")
        .and_then(|index| facts.definitions.get(index).copied().flatten())
        .ok_or("unmarked function")?;
    let unmarked = Instance::new(&facts.types, unmarked)?;
    let arguments =
        facts
            .types
            .intern_arguments(&[Argument::Type(crate::types::TypeId::primitive(
                crate::types::Primitive::I8,
            ))])?;
    let wrong = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments,
    })?;
    let wrong = Instance::new(&facts.types, wrong)?;
    let mut catalog = Catalog::default();
    let index = catalog.insert(&body)?;
    catalog.functions.insert(target, index);
    catalog.functions.insert(wrong, index);
    for (instance, expected) in [
        (open, "closed complete instance"),
        (missing, "another callable body or complete instance"),
        (unmarked, "authentic source const promise"),
        (wrong, "another callable body or complete instance"),
    ] {
        let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
        let mut machine =
            const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default());
        let error = machine
            .begin_promoted(&report, index, PromotedFunction::unscoped(instance), &[])
            .err()
            .ok_or("invalid promoted root executed")?;
        assert!(error.message.contains(expected), "{error}");
    }
    let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
    let mut machine =
        const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default());
    let argument = crate::sema::values::Constant::scalar(crate::sema::model::Scalar::Integer(1));
    let error = machine
        .begin_promoted(
            &report,
            index,
            PromotedFunction::unscoped(target),
            &[argument],
        )
        .err()
        .ok_or("extra promoted argument executed")?;
    assert!(
        error
            .message
            .contains("constant call argument arity mismatch"),
        "{error}"
    );
    Ok(())
}

#[test]
fn scoped_promotions_reject_foreign_bound_slots() -> TestResult {
    use crate::types::{
        Kind, Primitive, RegionId, TypeId, arguments::Argument, instance::Instance,
    };
    let (report, mut facts, body) = promoted_generic()?;
    let target = body.instance.ok_or("complete marked generic instance")?;
    let Kind::FunctionItem {
        item, signature, ..
    } = facts.types.kind(target.ty()).ok_or("function item")?
    else {
        return Err("function item".into());
    };
    let foreign_reference = facts.types.intern(Kind::Reference {
        mutable: false,
        referent: TypeId::primitive(Primitive::U8),
        region: RegionId::bound(0, 1)?,
    })?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Type(foreign_reference)])?;
    let foreign_type = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments,
    })?;
    let foreign = Instance::new(&facts.types, foreign_type)?;
    let mut evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
    let error = evaluator
        .promoted_call(
            foreign,
            PromotionCaller {
                body: body.source,
                instance: body.instance,
                signature: body.owner.signatures().map(|(_, execution)| execution),
            },
            Vec::new(),
        )
        .err()
        .ok_or("foreign lifetime was accepted as a caller slot")?;
    assert!(
        error
            .message
            .contains("lifetime outside its caller's binder"),
        "{error}"
    );

    let error = evaluator
        .check_promoted_instance(PromotedFunction::unscoped(foreign))
        .err()
        .ok_or("unscoped promotion accepted a bound lifetime")?;
    assert!(
        error.message.contains("closed complete instance"),
        "{error}"
    );
    Ok(())
}
