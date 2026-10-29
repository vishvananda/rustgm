//! Completed contexts retain sparse obligations, with exact warm replay and failed work.
use super::*;
use crate::support::crates::generics::source::CallSafety;
fn source() -> Result<String> {
    Ok(crate::test_source::fixture(
        "pa31/tests/spec/source-call-safety/provider.rs",
    )?)
}
fn pending(checker: &mut Checker<'_>, solved: &Solved, source: SourceCallUse) -> Result {
    let mut pending = super::super::super::safety::Pending::default();
    for &(node, term) in &checker.state.calls {
        let kind = source.call_safety(
            checker.facts,
            checker.state.body,
            node,
            solved.types[term.0 as usize],
        )?;
        if kind != CallSafety::Safe {
            pending
                .rows
                .push(super::super::super::safety::Record { node, kind });
        }
    }
    checker.state.call_types.safety = Some(Box::new(pending));
    Ok(())
}
#[test]
fn actual_discharged_sites_keep_owner_signature_and_safe_target_identity() -> Result {
    let (report, facts) = analyze(&source()?)?;
    for (name, expected) in [
        ("explicit", CallSafety::ExplicitUnsafeBlock),
        ("implicit", CallSafety::UnsafeFunction),
        ("explicit_unsafe", CallSafety::ExplicitUnsafeBlock),
        ("safe_in_unsafe", CallSafety::Safe),
        ("const_explicit", CallSafety::ExplicitUnsafeBlock),
    ] {
        let owner = body(&report, &facts, name)?;
        let handle = use_for(&facts, owner)?;
        let (node, signature) = facts.nodes[owner.file.0 as usize]
            .iter()
            .enumerate()
            .find_map(|(index, node)| match node.operation {
                Some(Operation::Call {
                    signature,
                    source_use: Some(actual),
                }) if actual == handle => Some((index, signature)),
                _ => None,
            })
            .ok_or("actual checked call")?;
        let node = crate::syntax::ast::NodeId::new(u32::try_from(node)?);
        assert_eq!(
            handle.call_safety(&facts, owner, node, signature)?,
            expected
        );
        assert_eq!(
            handle.call_safety(&facts, owner, node, crate::types::TypeId(u32::MAX))?,
            CallSafety::Safe
        );
        assert!(
            handle
                .call_safety(
                    &facts,
                    NodeKey {
                        file: crate::resolve::unit::FileId(owner.file.0 + 1),
                        ..owner
                    },
                    node,
                    signature
                )
                .is_err()
        );
    }
    println!(
        "CallTypes={} Contexts={} Descriptor={} Pending={} Record={}",
        size_of::<crate::sema::bodies::calls::CallTypes>(),
        size_of::<CallContexts>(),
        size_of::<Descriptor>(),
        size_of::<super::super::super::safety::Pending>(),
        size_of::<super::super::super::safety::Record>()
    );
    assert_eq!(size_of::<crate::sema::bodies::calls::CallTypes>(), 720);
    assert_eq!(size_of::<CallContexts>(), 88);
    assert_eq!(size_of::<Descriptor>(), 12);
    Ok(())
}
#[test]
fn discharged_replay_and_failed_work_cannot_change_published_safety() -> Result {
    let (report, mut facts) = analyze(&source()?)?;
    let owner = body(&report, &facts, "explicit")?;
    let handle = use_for(&facts, owner)?;
    let retained = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.call_contexts.as_deref())
        .ok_or("completed call contexts")?
        .bytes(row_count(&facts))?;
    let (mut checker, solved) = replay(&report, &mut facts, owner)?;
    pending(&mut checker, &solved, handle)?;
    checker.publish_calls(&solved)?;
    let capacity = checker
        .facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.call_contexts.as_deref())
        .ok_or("warm call contexts")?
        .bytes(row_count(checker.facts))?;
    assert_eq!(retained, capacity);
    let saved = checker
        .state
        .call_types
        .safety
        .take()
        .ok_or("actual pending calls")?;
    assert_eq!(saved.rows.len(), 1);
    let mut changed = saved;
    changed.rows[0].kind = CallSafety::UnsafeFunction;
    checker.state.call_types.safety = Some(changed);
    assert!(checker.publish_calls(&solved).is_err());
    pending(&mut checker, &solved, handle)?;
    checker.state.steps = 1_000_000;
    assert!(checker.publish_calls(&solved).is_err());
    assert!(checker.state.steps > 1_000_000);
    assert!(checker.publish_calls(&solved).is_err());
    checker.state.steps = 0;
    pending(&mut checker, &solved, handle)?;
    checker.publish_calls(&solved)?;
    Ok(())
}
