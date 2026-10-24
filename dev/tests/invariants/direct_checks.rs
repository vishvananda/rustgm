//! Direct member receiver publication keeps exact typed identity without authority.
use super::*;

#[test]
fn direct_defining_member_keeps_full_receiver_and_rejects_missing_or_changed_facts() -> Result {
    let (mut facts, template, definition) = named_original(
        "pa31/tests/spec/trait-default-effects/members/provider-implicit-unused.rs",
        "provided",
    )?;
    let body = template.body;
    let index = statement(&body)?;
    let source = body
        .source
        .proof_site(body.statements[index].source)?
        .source()
        .ok_or("source")?;
    let retained = *facts
        .invocations
        .get(&source)
        .ok_or("actual direct receiver")?;
    assert!(retained.kind.is_none() && retained.borrow.is_none() && retained.dereferences == 0);
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut work = Work::new(1_000_000);
        let context = prepare(&facts, &body, definition, &mut work)?;
        let setup = work.steps;
        for _ in 0..count {
            check(&context, &facts, &body, &body.statements[index], &mut work)?;
        }
        let visits = work.steps - setup;
        if let Some((width, old)) = previous {
            assert_eq!(visits * width, old * count);
        }
        previous = Some((count, visits));
        println!("direct_defining_calls={count} setup={setup} visits={visits}");
    }
    for receiver in [
        None,
        Some(crate::sema::invocations::Invocation {
            receiver: crate::types::TypeId::UNIT,
            ..retained
        }),
        Some(crate::sema::invocations::Invocation {
            kind: Some(crate::support::call_traits::CallTrait::FnOnce),
            ..retained
        }),
    ] {
        if let Some(receiver) = receiver {
            facts.invocations.insert(source, receiver);
        } else {
            facts.invocations.remove(&source);
        }
        let mut work = Work::new(1_000_000);
        let context = prepare(&facts, &body, definition, &mut work)?;
        let before = work.steps;
        assert!(check(&context, &facts, &body, &body.statements[index], &mut work).is_err());
        assert!(work.steps > before);
    }
    Ok(())
}

#[test]
fn direct_defining_member_rejects_foreign_trait_and_incomplete_receiver() -> Result {
    let (mut facts, template, definition) = named_original(
        "pa31/tests/spec/trait-default-effects/members/provider-implicit-unused.rs",
        "provided",
    )?;
    let body = template.body;
    let index = statement(&body)?;
    let source = body
        .source
        .proof_site(body.statements[index].source)?
        .source()
        .ok_or("source")?;
    let retained = *facts
        .invocations
        .get(&source)
        .ok_or("actual direct receiver")?;
    let trait_definition = facts
        .associated
        .trait_owner(definition)
        .ok_or("actual trait")?;
    let mut changed = body.clone();
    let Action::Evaluate(value) = changed.statements[index].action else {
        return Err("Evaluate".into());
    };
    let Rvalue::Call { callee, .. } = changed.values[value.0 as usize].kind else {
        return Err("Call".into());
    };
    // A rigid Self parameter is not a completed function-item receiver.
    changed.values[callee.0 as usize].ty = facts
        .associated
        .schema(trait_definition)
        .ok_or("schema")?
        .self_type;
    let mut work = Work::new(1_000_000);
    let context = prepare(&facts, &body, definition, &mut work)?;
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
    assert!(
        prepare(
            &facts,
            &body,
            crate::resolve::model::DefId(u32::MAX),
            &mut work
        )
        .is_err()
    );
    let other = facts
        .functions
        .iter()
        .find(|function| {
            facts
                .associated
                .trait_owner(function.definition)
                .is_some_and(|owner| {
                    owner != trait_definition
                        && facts.associated.constant_trait(owner) == Some(true)
                })
        })
        .ok_or("another actual const trait member")?
        .definition;
    let Some(crate::types::Kind::FunctionItem {
        signature,
        arguments,
        ..
    }) = facts.types.kind(retained.receiver)
    else {
        return Err("full receiver".into());
    };
    let receiver = facts.types.intern(crate::types::Kind::FunctionItem {
        item: crate::types::ItemId(other.0),
        signature,
        arguments,
    })?;
    changed.values[callee.0 as usize].ty = receiver;
    changed.values[callee.0 as usize].kind = Rvalue::Function(other);
    let context = prepare(&facts, &body, definition, &mut work)?;
    let error = check(
        &context,
        &facts,
        &changed,
        &changed.statements[index],
        &mut work,
    )
    .err()
    .ok_or("foreign trait accepted")?;
    assert!(error.contains("another actual trait member"), "{error}");
    Ok(())
}
