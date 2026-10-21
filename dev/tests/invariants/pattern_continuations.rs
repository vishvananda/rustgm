//! Value and pattern ownership survive a typed declaration-query continuation.
use super::*;
use crate::sema::{constants::PatternScope, declarations, dispatch::Dispatcher};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn continuation_restores_exact_pattern_and_value_owners() -> TestResult {
    let report = crate::test_source::resolve("const N:u8=7;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let initializer = facts.initializers[0];
    let body = initializer.semantic_source();
    let mut queries = Queries::new(&report, &mut facts);
    queries.pattern_scope = Some(PatternScope {
        body,
        owner: Some(initializer.definition),
        binders: 1,
        end: 3,
    });
    queries.value_scope = Some(instances::ValueScope::Initializer { body, index: 0 });
    queries.pattern_dispatch = Some(Dispatcher::new(1, 1)?);
    let (scratch, continuation) = queries.suspend()?;
    let mut resumed = Queries::with_scratch(&report, &mut facts, None, scratch);
    resumed.resume(*continuation);
    let scope = resumed.pattern_scope.ok_or("lost pattern owner")?;
    assert_eq!(
        (scope.body, scope.owner, scope.binders, scope.end),
        (body, Some(initializer.definition), 1, 3)
    );
    assert!(
        matches!(resumed.value_scope, Some(instances::ValueScope::Initializer { body: actual, index: 0 }) if actual == body)
    );
    let proof = resumed
        .pattern_dispatch
        .take()
        .ok_or("lost dispatcher")?
        .finish();
    assert_eq!(proof.equalities.len(), 1);
    Ok(())
}
#[test]
fn oversized_dispatcher_is_rejected_before_continuation_publication() -> TestResult {
    let report = crate::test_source::resolve("const N:u8=7;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let mut queries = Queries::new(&report, &mut facts);
    queries.pattern_dispatch = Some(Dispatcher::new(0, 1)?);
    assert!(matches!(queries.retain_pattern_dispatcher(0), Err(error)
        if error.to_string().contains("suspended pattern dispatcher exceeds")));
    assert!(queries.pattern_dispatch.is_some());
    Ok(())
}
