//! Typed synchronous outcomes preserve dependencies, profile evidence and real failures.
use super::*;

fn checked(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
) -> std::result::Result<Selection, selection::Failure> {
    let inputs = caller();
    selection::select_available_checked(
        facts,
        context,
        goal,
        selection::Caller {
            assumptions: &inputs.assumptions,
            environment: inputs.environment.as_ref(),
            first_region: inputs.first_region,
            retain_sized: inputs.retain_sized,
        },
        Availability::Executable,
    )
}

#[test]
fn checked_pending_profile_and_completion_never_publish_absence() -> Result {
    let (mut facts, owner, predicate, _) = setup()?;
    let mut context = Context::default();
    let goal = goal(&mut facts, &mut context, predicate)?;
    assert!(
        matches!(checked(&mut facts, &mut context, &goal), Err(selection::Failure::Pending(Dependency::ImplementationValidated(id))) if id == owner)
    );
    let mut state = crate::sema::nominals::traits::State::body(&mut facts, None, 0, 1)?;
    assert_eq!(
        state.copy_decision(&mut facts, predicate).err(),
        Some("canonical trait selection requires pending declarations")
    );
    let arena = facts.types.arena_identity();
    let registry = facts.generics.implementations.as_mut().ok_or("registry")?;
    let source = registry.header(owner).ok_or("actual header")?.source;
    registry.mark_early_unavailable(&facts.types, &arena, owner, source, predicate)?;
    assert!(!registry.is_validated(owner));
    assert!(registry.early_unavailable(&facts.types, owner)?);
    assert!(
        registry
            .early_unavailable(&crate::types::Store::default(), owner)
            .is_err()
    );
    assert!(
        matches!(checked(&mut facts, &mut context, &goal), Err(selection::Failure::Pending(Dependency::ImplementationValidated(id))) if id == owner)
    );
    assert_eq!(
        state.copy_decision(&mut facts, predicate).err(),
        Some("implementation validation is outside the early scalar profile")
    );
    let registry = facts.generics.implementations.as_mut().ok_or("registry")?;
    registry.complete(owner, Vec::new(), Vec::new(), Vec::new())?;
    assert!(!registry.early_unavailable(&facts.types, owner)?);
    assert!(registry.is_validated(owner));
    assert!(matches!(
        checked(&mut facts, &mut context, &goal),
        Ok(Selection::Proven(_))
    ));
    assert!(state.copy_decision(&mut facts, predicate)?);
    Ok(())
}

#[test]
fn profile_markers_reject_foreign_arena_source_predicate_and_completed_owner() -> Result {
    let (mut facts, owner, predicate, _) = setup()?;
    let arena = facts.types.arena_identity();
    let foreign = crate::types::Store::default();
    let registry = facts.generics.implementations.as_mut().ok_or("registry")?;
    let source = registry.header(owner).ok_or("header")?.source;
    assert!(
        registry
            .mark_early_unavailable(&foreign, &arena, owner, source, predicate)
            .is_err()
    );
    let wrong_source = crate::resolve::unit::NodeKey {
        node: crate::syntax::ast::NodeId::new(u32::MAX),
        ..source
    };
    assert!(
        registry
            .mark_early_unavailable(&facts.types, &arena, owner, wrong_source, predicate)
            .is_err()
    );
    assert!(
        registry
            .mark_early_unavailable(
                &facts.types,
                &arena,
                owner,
                source,
                Predicate {
                    ty: crate::types::TypeId::primitive(Primitive::Bool),
                    ..predicate
                }
            )
            .is_err()
    );
    assert!(
        registry
            .mark_early_unavailable(&facts.types, &arena, DefId(u32::MAX), source, predicate)
            .is_err()
    );
    assert!(!registry.early_unavailable(&facts.types, owner)?);
    registry.mark_early_unavailable(&facts.types, &arena, owner, source, predicate)?;
    registry.complete(owner, Vec::new(), Vec::new(), Vec::new())?;
    assert!(
        registry
            .mark_early_unavailable(&facts.types, &arena, owner, source, predicate)
            .is_err()
    );
    assert!(!registry.early_unavailable(&facts.types, owner)?);
    Ok(())
}

#[test]
fn checked_budget_and_invalid_goals_remain_real_failures() -> Result {
    let (mut facts, _, predicate, _) = setup()?;
    let mut context = Context::default();
    let goal = goal(&mut facts, &mut context, predicate)?;
    let mut foreign_context = Context::default();
    assert!(matches!(
        checked(&mut facts, &mut foreign_context, &goal),
        Err(selection::Failure::Invalid(_))
    ));
    context.steps = 1_000_000;
    assert!(matches!(
        checked(&mut facts, &mut context, &goal),
        Err(selection::Failure::Invalid(_))
    ));
    assert!(context.steps > 1_000_000);
    Ok(())
}
