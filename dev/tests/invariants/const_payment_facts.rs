//! Semantic success is scoped and still needs one actual reached interpreter use.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn closed_const_requirements_preserve_effects_and_environment_identity() -> Result {
    let source = crate::provider_cases::UNUSED_CONDITIONAL_CONST_PAID;
    let report = crate::test_source::source_provider(source, "const_payment_facts")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let definition = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "unused")
        .ok_or("unused")?;
    let definition = DefId(u32::try_from(definition)?);
    let signature =
        facts.functions[*facts.function_indices.get(&definition).ok_or("function")?].signature;
    let arguments = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(TypeId::primitive(
            Primitive::U8,
        ))])?;
    let ty = facts.types.intern(Kind::FunctionItem {
        item: crate::types::ItemId(definition.0),
        signature,
        arguments,
    })?;
    let instance = Instance::new(&facts.types, ty)?;
    assert!(prepare(&mut facts, instance, 0).is_err());
    let requirements = prepare(&mut facts, instance, 64 * 1024 * 1024)?;
    assert!(
        requirements
            .goals
            .iter()
            .all(|g| matches!(g.bound, Bound::ConstTrait(_)))
    );
    assert_eq!(
        crate::sema::compiler_items::calls::declared_effect_for_test(&mut facts, definition)?,
        Effect::Conditional
    );
    let payment = pay(&mut facts, &requirements, None)?;
    payment.validate(&requirements, None)?;
    let first = Arc::new(Environment::new(&facts.types, &[])?);
    let second = Arc::new(Environment::new(&facts.types, &[])?);
    assert!(payment.validate(&requirements, Some(&first)).is_err());
    let payment = pay(&mut facts, &requirements, Some(&first))?;
    assert!(payment.validate(&requirements, Some(&second)).is_err());
    let arguments = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(TypeId::primitive(
            Primitive::U16,
        ))])?;
    let other = facts.types.intern(Kind::FunctionItem {
        item: crate::types::ItemId(definition.0),
        signature,
        arguments,
    })?;
    assert!(
        requirements
            .matches(&facts, Instance::new(&facts.types, other)?)
            .is_err()
    );
    facts.generics.environments.insert(definition, second);
    assert!(requirements.matches(&facts, instance).is_err());
    Ok(())
}
