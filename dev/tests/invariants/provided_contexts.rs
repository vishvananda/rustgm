//! Indexed defining context does not mint executable const capability.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn actual_default_context_keeps_nested_runtime_and_rejects_mismatched_bodies() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-default-effects/provider/nested-ordinary.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "default_contexts")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let function = |name| {
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
            .ok_or("actual function")
    };
    let provided = function("provided")?;
    let nested = function("nested")?;
    let constant = function("constant")?;
    let source = provided.body.ok_or("default body")?;
    assert!(
        matches!(facts.function_context(provided.definition, source)?,
        FunctionContext::ConstTraitDefault { trait_definition }
        if facts.associated.constant_trait(trait_definition) == Some(true))
    );
    assert!(!facts.is_const_function(provided.definition));
    actual_abstract_self_clause(&facts, provided, nested)?;
    assert_eq!(
        facts.function_context(nested.definition, nested.body.ok_or("nested body")?)?,
        FunctionContext::Runtime
    );
    assert_eq!(
        facts.function_context(constant.definition, constant.body.ok_or("const body")?)?,
        FunctionContext::ConstFunction
    );
    assert!(facts.function_context(nested.definition, source).is_err());
    let index = facts
        .function_indices
        .remove(&provided.definition)
        .ok_or("index")?;
    assert!(facts.function_context(provided.definition, source).is_err());
    facts.function_indices.insert(provided.definition, index);
    facts.functions[index].definition = nested.definition;
    assert!(facts.function_context(provided.definition, source).is_err());
    Ok(())
}

fn actual_abstract_self_clause(
    facts: &crate::sema::model::Declarations,
    provided: crate::sema::model::Function,
    nested: crate::sema::model::Function,
) -> Result {
    let source = provided.body.ok_or("actual provided body")?;
    let trait_definition = facts
        .associated
        .trait_owner(provided.definition)
        .ok_or("trait")?;
    let trait_owner = facts
        .associated
        .owner(trait_definition)
        .and_then(|owner| facts.associated.get(owner))
        .ok_or("trait owner")?;
    let ordinary = facts.generics.trait_self_bounds[&trait_definition];
    let defining = facts.defining_self_clause(trait_definition)?;
    assert_eq!(
        facts.default_self_clause(provided.definition, source)?,
        Some(defining)
    );
    assert!(matches!(
        defining,
        Clause::Predicate(Predicate {
            ty,
            bound: Bound::MaybeConstTrait(application),
        }) if ty == ordinary.ty && ordinary.bound == Bound::Trait(application)
    ));
    assert!(facts.defining_self_clause(nested.definition).is_err());
    assert!(
        facts
            .default_self_clause(nested.definition, source)
            .is_err()
    );
    let input = crate::sema::declaration_inputs::Input {
        source: trait_owner.source_owner()?.source.into(),
        value: ordinary.into(),
    };
    let projected = facts.member_use_clause(
        provided.definition,
        input,
        crate::support::constness::Constness::Conditional,
    )?;
    assert!(matches!(
        projected,
        Clause::Predicate(Predicate {
            bound: Bound::MaybeConstTrait(_),
            ..
        })
    ));
    let unrelated = crate::sema::declaration_inputs::Input {
        source: provided.node.into(),
        ..input
    };
    assert_eq!(
        facts.member_use_clause(
            provided.definition,
            unrelated,
            crate::support::constness::Constness::Conditional
        )?,
        input.value
    );
    assert_eq!(
        facts.generics.trait_self_bounds[&trait_definition],
        ordinary
    );
    Ok(())
}

#[test]
fn provided_self_premise_is_local_and_required_for_abstract_call_checking() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "provided_premise")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "clone_from"
        })
        .copied()
        .ok_or("actual provided clone_from")?;
    let body = function.body.ok_or("actual body")?;
    let mut visits = 0;
    DefiningCalls::new(&facts, function.definition, body, |n| {
        visits += n;
        Ok(())
    })?;
    assert!(visits > 0);
    let environment = facts
        .generics
        .body_environments
        .remove(&function.definition)
        .ok_or("local environment")?;
    let mut failed = 0;
    assert!(
        DefiningCalls::new(&facts, function.definition, body, |n| {
            failed += n;
            Ok(())
        })
        .is_err()
    );
    assert!(failed > 0);
    facts
        .generics
        .body_environments
        .insert(function.definition, environment);
    DefiningCalls::new(&facts, function.definition, body, |_| Ok(()))?;
    assert!(!facts.is_const_function(function.definition));
    Ok(())
}

#[test]
fn ordinary_nonprovider_defining_check_keeps_zero_added_work() -> Result {
    let report = crate::test_source::resolve("fn ordinary()->u8{7}", true)?;
    let crate::pipeline::Analysis {
        mut facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = *facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "ordinary"
        })
        .ok_or("ordinary function")?;
    let template = prepared.templates.function(&report, &facts, &function)?;
    assert!(!template.body.const_context);
    assert!(report.package.source_core().is_none());
    let before = facts.body_work.canonical_steps;
    crate::body::constants::eligibility_charged(&report, &mut facts, &template.body)?;
    assert_eq!(facts.body_work.canonical_steps, before);
    Ok(())
}

#[test]
fn nonprovider_const_defaults_do_not_skip_actual_defining_checks() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-default-effects/default-nonconst-definition.rs",
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    assert!(report.package.source_core().is_none());
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let result =
        crate::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ());
    let error = result
        .err()
        .ok_or("ordinary-profile invalid default accepted")?;
    assert!(
        error
            .to_string()
            .contains("non-const calls are not allowed"),
        "{error}"
    );
    Ok(())
}
