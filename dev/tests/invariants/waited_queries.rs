//! Count the actual retained source query before and after semantic validation.
use super::{Context, Goal, Pending, Progress};
use crate::{
    resolve::model::DefId,
    sema::{
        model::Declarations,
        traits::{
            assumptions::Assumptions,
            implementations::{Availability, Identity, Implementation, Implementations},
            selection::{self, Dependency, OwnedCaller, Selection},
        },
    },
    types::{ItemId, Kind, Primitive, infer::Instantiation, parameters::predicates::Predicate},
};
use std::sync::Arc;
#[path = "checked_queries.rs"]
mod checked;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn setup() -> Result<(Declarations, DefId, Predicate, Box<Implementations>)> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/early-copy-readiness/runtime-read.rs")?;
    let report = crate::test_source::source_provider(&source, "source_traits_v1")?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let copy = facts
        .source_bindings
        .as_deref()
        .ok_or("missing actual source catalog")?
        .validated_language(crate::support::bindings::LangItem::Copy)?
        .ok_or("missing actual Copy declaration")?
        .definition;
    let registry = facts
        .generics
        .implementations
        .as_ref()
        .ok_or("missing actual registry")?;
    let header = registry
        .headers()
        .find(|header| {
            header
                .predicate
                .bound
                .trait_parts()
                .and_then(|(id, _)| facts.types.trait_application(id))
                .is_some_and(|app| app.item.0 == copy.0)
                && facts.types.kind(header.predicate.ty) == Some(Kind::Primitive(Primitive::U8))
        })
        .ok_or("missing actual scalar Copy implementation")?;
    assert!(registry.is_validated(header.owner));
    let owner = header.owner;
    let predicate = header.predicate;
    let input = Implementation {
        owner,
        source: header.source,
        predicate,
        methods: Vec::new(),
        types: Vec::new(),
        constants: Vec::new(),
    };
    let actual = facts
        .associated
        .owner(owner)
        .and_then(|id| facts.associated.get(id))
        .copied()
        .ok_or("missing actual source owner")?;
    let mut waiting = Implementations::default();
    waiting.prepare_source(
        &mut facts.types,
        &facts.associated,
        Identity::User(ItemId(copy.0)),
        actual,
        input,
    )?;
    waiting.finish_constant_headers()?;
    assert!(!waiting.is_validated(owner));
    let completed = facts
        .generics
        .implementations
        .replace(Box::new(waiting))
        .ok_or("missing completed registry")?;
    Ok((facts, owner, predicate, completed))
}
fn goal(facts: &mut Declarations, context: &mut Context, predicate: Predicate) -> Result<Goal> {
    let mut substitution = Instantiation::new(context, &facts.types, &[])?;
    Ok(context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)?)
}
fn caller() -> OwnedCaller {
    OwnedCaller {
        assumptions: Arc::new(Assumptions::default()),
        environment: None,
        first_region: 1,
        retain_sized: false,
    }
}
fn pending(progress: Progress, owner: DefId) -> Result<Box<Pending>> {
    let Progress::Pending(dependency, query) = progress else {
        return Err("actual unfinished Copy query did not retain a dependency".into());
    };
    assert_eq!(dependency, Dependency::ImplementationValidated(owner));
    Ok(query)
}
#[test]
fn counted_copy_query_preserves_pending_then_consumes_genuine_completion() -> Result {
    let (mut facts, owner, predicate, completed) = setup()?;
    let mut context = Context::default();
    let goal = goal(&mut facts, &mut context, predicate)?;
    // The legacy completed-only view must not contaminate the executable query.
    assert!(matches!(
        selection::begin_canonical(
            &mut facts,
            &mut context,
            &goal,
            caller(),
            (Availability::Validated, None),
            None
        )?,
        Progress::Ready(Selection::Absent)
    ));
    let mut observed = 0;
    let mut query = pending(
        selection::begin_executable_counted(
            &mut facts,
            &mut context,
            &goal,
            caller(),
            &mut observed,
        )?,
        owner,
    )?;
    assert!(observed > 0);
    let first = observed;
    query = pending(
        query.resume_counted(&mut facts, &mut context, &goal, 1, &mut observed)?,
        owner,
    )?;
    assert!(observed > first);
    assert!(
        !facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|r| r.is_validated(owner))
    );
    facts.generics.implementations = Some(completed);
    let Progress::Ready(Selection::Proven(evidence)) =
        query.resume_counted(&mut facts, &mut context, &goal, 1, &mut observed)?
    else {
        return Err("actual completed proof was lost".into());
    };
    assert!(
        matches!(evidence.source, selection::Source::Implementation { owner: actual, .. } if actual == owner)
    );
    assert!(context.steps >= observed);
    Ok(())
}
#[test]
fn repeated_pending_lookups_exhaust_sticky_query_work_without_absence() -> Result {
    let (mut facts, owner, predicate, _) = setup()?;
    let mut context = Context::default();
    let goal = goal(&mut facts, &mut context, predicate)?;
    let mut observed = 0;
    let mut query = pending(
        selection::begin_executable_counted(
            &mut facts,
            &mut context,
            &goal,
            caller(),
            &mut observed,
        )?,
        owner,
    )?;
    query.query.steps = 999_999;
    query = pending(
        query.resume_counted(&mut facts, &mut context, &goal, 1, &mut observed)?,
        owner,
    )?;
    assert_eq!(observed, 1_000_000);
    assert!(
        query
            .resume_counted(&mut facts, &mut context, &goal, 1, &mut observed)
            .is_err()
    );
    assert!(observed > 1_000_000);
    assert!(
        !facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|r| r.is_validated(owner))
    );
    Ok(())
}
#[test]
fn counted_ready_failure_and_precreation_error_keep_attempted_prefix() -> Result {
    let (mut facts, owner, predicate, completed) = setup()?;
    let mut context = Context::default();
    let goal = goal(&mut facts, &mut context, predicate)?;
    let mut observed = 0;
    let query = pending(
        selection::begin_executable_counted(
            &mut facts,
            &mut context,
            &goal,
            caller(),
            &mut observed,
        )?,
        owner,
    )?;
    let prefix = observed;
    facts.generics.implementations = Some(completed);
    context.steps = 999_999;
    assert!(
        query
            .resume_counted(&mut facts, &mut context, &goal, 1, &mut observed)
            .is_err()
    );
    assert!(observed >= prefix);
    assert!(context.steps > 1_000_000);
    let mut empty = Context::default();
    let mut previous = 17;
    assert!(
        selection::begin_executable_counted(&mut facts, &mut empty, &goal, caller(), &mut previous)
            .is_err()
    );
    assert!(previous >= 17);
    Ok(())
}
