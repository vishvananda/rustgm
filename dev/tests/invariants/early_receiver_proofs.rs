//! Actual source receiver publication preserves full validation boundaries.
use super::*;

#[test]
fn exact_source_self_publishes_only_after_receiver_proof_and_keeps_distinct_arguments_pending()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture("pa31/tests/spec/source-location/caller-location.rs")?;
    let report = crate::test_source::source_provider(&source, "receiver_publication")?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let function = checker
        .facts
        .functions
        .iter()
        .copied()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "line"
        })
        .ok_or("actual Location line method")?;
    let (key, original, target) = checker
        .actual_method_receiver(&function)
        .ok_or("actual typed receiver")?;
    assert_eq!(original.base, target);
    checker.facts.methods.receivers.remove(&function.definition);
    checker.publish_exact_method_receiver(&function)?;
    assert_eq!(
        checker.facts.methods.receivers.get(&function.definition),
        Some(&(original.ty, original.references))
    );
    let Some(Type::Adt(item, _)) = checker.facts.types.kind(target) else {
        return Err("actual nominal Self".into());
    };
    let arguments =
        checker
            .facts
            .types
            .intern_arguments(&[crate::types::arguments::Argument::Lifetime(
                crate::types::RegionId::STATIC,
            )])?;
    let distinct = checker.facts.types.intern(Type::Adt(item, arguments))?;
    assert_ne!(distinct, target);
    checker.facts.methods.receivers.remove(&function.definition);
    checker.facts.receivers.insert(
        key,
        crate::sema::associated::Receiver {
            base: distinct,
            ..original
        },
    );
    checker.publish_exact_method_receiver(&function)?;
    assert!(
        !checker
            .facts
            .methods
            .receivers
            .contains_key(&function.definition)
    );
    checker.facts.receivers.insert(
        key,
        crate::sema::associated::Receiver {
            base: TypeId::primitive(crate::types::Primitive::U8),
            ..original
        },
    );
    assert!(checker.validate_method_receiver(&function).is_err());
    assert!(
        !checker
            .facts
            .methods
            .receivers
            .contains_key(&function.definition)
    );
    checker.facts.receivers.insert(key, original);
    checker.validate_method_receiver(&function)?;
    assert_eq!(
        checker.facts.methods.receivers.get(&function.definition),
        Some(&(original.ty, original.references))
    );
    Ok(())
}
