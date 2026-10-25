//! Actual source owners bind a scalar plan; completion still uses normal conformance.
use super::super::{
    Checker, DefId,
    eligibility_profile::{Eligibility, Plan, Unsupported},
    scalar_readiness::Pending,
};
use super::{Attempt, Step};
use crate::{
    sema::traits::implementations::{Implementation, Implementations},
    types::parameters::environment::Environment,
};
use std::sync::Arc;
#[path = "early_unavailable.rs"]
mod unavailable;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/early-copy-readiness/{name}.rs"
    ))?)
}
fn owners(checker: &Checker<'_>, name: &str) -> Result<Vec<DefId>> {
    Ok(checker
        .facts
        .generics
        .implementations
        .as_ref()
        .ok_or("missing actual registry")?
        .headers()
        .filter(|header| {
            header
                .predicate
                .bound
                .trait_parts()
                .and_then(|(id, _)| checker.facts.types.trait_application(id))
                .is_some_and(|app| {
                    checker
                        .report
                        .package
                        .names
                        .get(checker.report.resolution.definitions[app.item.0 as usize].name)
                        == name
                })
        })
        .map(|header| header.owner)
        .collect())
}
fn plan(checker: &mut Checker<'_>, pending: &mut Pending, owner: DefId) -> Result<Plan> {
    let header = checker
        .facts
        .generics
        .implementations
        .as_ref()
        .and_then(|r| r.header(owner))
        .ok_or("missing actual header")?;
    let (source, predicate) = (
        header.source.ok_or("actual source header")?,
        header.predicate,
    );
    let Eligibility::Ready(plan) = checker.early_impl_plan(owner, source, predicate, pending)?
    else {
        return Err("actual closed source owner was outside scalar profile".into());
    };
    Ok(plan)
}
#[test]
fn actual_plan_rejects_distinct_full_arguments_and_forked_environment() -> Result {
    let report =
        crate::test_source::source_provider(&source("indexed-requirements")?, "source_traits_v1")?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let extra = owners(&checker, "Extra")?;
    assert_eq!(extra.len(), 2);
    let (a, b) = checker
        .facts
        .generics
        .implementations
        .as_ref()
        .ok_or("missing registry")?
        .headers()
        .filter(|h| extra.contains(&h.owner))
        .map(|h| (h.owner, h.source, h.predicate))
        .collect::<Vec<_>>()
        .split_first()
        .and_then(|(a, rest)| rest.first().map(|b| (*a, *b)))
        .ok_or("missing complete application pair")?;
    assert_eq!(a.2.ty, b.2.ty);
    let apps = [a.2, b.2].map(|p| {
        p.bound
            .trait_parts()
            .and_then(|(id, _)| checker.facts.types.trait_application(id))
    });
    let left = apps[0].ok_or("missing first trait application")?;
    let right = apps[1].ok_or("missing second trait application")?;
    assert_eq!(left.item, right.item);
    assert_ne!(left.arguments, right.arguments);
    let item = left.item;
    let mut pending = Pending::default();
    let plan = plan(&mut checker, &mut pending, a.0)?;
    plan.require_current(&checker.facts, a.0)?;
    let complete = checker
        .facts
        .generics
        .implementations
        .take()
        .ok_or("missing registry")?;
    let mut forged = Implementations::default();
    forged.prepare(
        &mut checker.facts.types,
        item,
        Implementation {
            owner: a.0,
            source: a.1,
            predicate: b.2,
            methods: Vec::new(),
            types: Vec::new(),
            constants: Vec::new(),
        },
    )?;
    checker.facts.generics.implementations = Some(Box::new(forged));
    assert!(
        plan.require_current(&checker.facts, a.0)
            .is_err_and(|e| e.contains("complete predicate"))
    );
    checker.facts.generics.implementations = Some(complete);
    let fork = if let Some(environment) = &plan.environment {
        (**environment).clone()
    } else {
        Environment::new(&checker.facts.types, &[])?
    };
    let old = checker
        .facts
        .generics
        .environments
        .insert(a.0, Arc::new(fork));
    assert!(
        plan.require_current(&checker.facts, a.0)
            .is_err_and(|e| e.contains("environment"))
    );
    if let Some(old) = old {
        checker.facts.generics.environments.insert(a.0, old);
    } else {
        checker.facts.generics.environments.remove(&a.0);
    }
    plan.require_current(&checker.facts, a.0)?;
    Ok(())
}
#[test]
fn environment_free_plan_rejects_same_header_in_foreign_actual_arena() -> Result {
    let report = crate::test_source::source_provider(&source("runtime-read")?, "source_traits_v1")?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let owner = *owners(&checker, "Copy")?
        .first()
        .ok_or("missing actual Copy owner")?;
    let mut pending = Pending::default();
    let plan = plan(&mut checker, &mut pending, owner)?;
    assert!(plan.environment.is_none());
    let mut foreign = Checker::new(&report)?;
    foreign.run()?;
    let original = checker
        .facts
        .generics
        .implementations
        .as_ref()
        .and_then(|r| r.header(owner))
        .ok_or("missing original")?;
    let other = foreign
        .facts
        .generics
        .implementations
        .as_ref()
        .and_then(|r| r.header(owner))
        .ok_or("missing foreign")?;
    assert_eq!(
        (original.owner, original.source, original.predicate),
        (other.owner, other.source, other.predicate)
    );
    assert!(
        plan.require_current(&foreign.facts, owner)
            .is_err_and(|e| e.contains("another actual type arena"))
    );
    assert!(pending.arena_for(&mut foreign.facts).is_err());
    plan.require_current(&checker.facts, owner)?;
    Ok(())
}
#[test]
fn each_required_predicate_has_one_context_start_and_unique_work_transfer() -> Result {
    let report =
        crate::test_source::source_provider(&source("indexed-requirements")?, "source_traits_v1")?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let owner = *owners(&checker, "Pair")?
        .first()
        .ok_or("missing actual Pair owner")?;
    let mut pending = Pending::default();
    let plan = plan(&mut checker, &mut pending, owner)?;
    assert!(plan.requirements.len() >= 2);
    let expected = plan.requirements.len();
    let mut attempt = Attempt::new(plan);
    assert!(matches!(
        attempt.advance(&mut checker.facts, owner)?,
        Step::Complete
    ));
    assert_eq!(attempt.cursor, expected);
    assert!(attempt.goal.is_none());
    assert!(attempt.context.steps > 0);
    assert!(attempt.query_context_start > 0);
    assert!(attempt.query_context_start + attempt.query_work <= attempt.context.steps);
    Ok(())
}
#[test]
fn ordinary_fast_path_and_exhausted_source_budget_allocate_no_owner() -> Result {
    let report = crate::test_source::resolve(&source("ordinary-no-source-policy")?, true)?;
    let mut checker = Checker::new(&report)?;
    let before = checker.facts.body_work.canonical_steps;
    checker.prepare_early_implementations(&mut 0, usize::MAX)?;
    assert!(checker.early_implementations.is_none());
    assert_eq!(checker.facts.body_work.canonical_steps, before);
    let mut pending = Pending::default();
    pending.work = 1_000_000;
    assert!(pending.arena_for(&mut checker.facts).is_err());
    assert_eq!(pending.work, 1_000_001);
    assert_eq!(checker.facts.body_work.canonical_steps, before + 1);
    assert!(pending.plans.is_empty());
    assert!(pending.active.is_empty());
    Ok(())
}
#[test]
fn rejected_owner_publishes_neither_validation_nor_early_constant_inventory() -> Result {
    let report =
        crate::test_source::source_provider(&source("wrong-clone-signature")?, "source_traits_v1")?;
    let mut checker = Checker::new(&report)?;
    assert!(checker.run().is_err());
    for name in ["Clone", "Copy"] {
        for owner in owners(&checker, name)? {
            assert!(
                !checker
                    .facts
                    .generics
                    .implementations
                    .as_ref()
                    .is_some_and(|r| r.is_validated(owner))
            );
        }
    }
    assert!(checker.constant_members.members.is_none());
    Ok(())
}

#[test]
fn functions_only_witness_authenticates_both_actual_complete_buckets() -> Result {
    let report = crate::test_source::source_provider(&source("runtime-read")?, "source_traits_v1")?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let owner = *owners(&checker, "Clone")?
        .first()
        .ok_or("missing actual Clone owner")?;
    let mut pending = Pending::default();
    let mut plan = plan(&mut checker, &mut pending, owner)?;
    assert!(!plan.members.is_empty());
    plan.require_no_associated(&checker.facts, owner)?;
    let member = plan.members.pop().ok_or("missing actual method")?;
    assert!(plan.require_no_associated(&checker.facts, owner).is_err());
    plan.members.push(member);
    plan.require_no_associated(&checker.facts, owner)?;
    for (fixture, trait_name) in [
        ("associated-type-profile", "AssociatedType"),
        ("associated-constant-profile", "AssociatedConstant"),
    ] {
        let report = crate::test_source::source_provider(&source(fixture)?, "source_traits_v1")?;
        let mut actual = Checker::new(&report)?;
        actual.run()?;
        let owner = *owners(&actual, trait_name)?
            .first()
            .ok_or("missing actual associated owner")?;
        let header = actual
            .facts
            .generics
            .implementations
            .as_ref()
            .and_then(|r| r.header(owner))
            .ok_or("missing associated header")?;
        let (source, predicate) = (
            header.source.ok_or("actual source header")?,
            header.predicate,
        );
        assert!(matches!(
            actual.early_impl_plan(owner, source, predicate, &mut Pending::default())?,
            Eligibility::Unsupported(Unsupported::Members)
        ));
    }
    Ok(())
}
#[test]
fn early_validation_cold_owner_keeps_existing_hot_records_fixed() {
    use std::mem::size_of;
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    assert_eq!(size_of::<crate::body::regions::Contract>(), 184);
    assert_eq!(size_of::<crate::sema::nominals::traits::State>(), 800);
    println!(
        "early_validation_layout checker={} readiness={} pending={} plan={} attempt={}",
        size_of::<Checker<'_>>(),
        size_of::<super::super::readiness::Readiness>(),
        size_of::<Pending>(),
        size_of::<Plan>(),
        size_of::<Attempt>()
    );
}

#[test]
fn source_provider_mismatch_rejects_before_a_plan_or_owner_is_published() -> Result {
    let text = source("runtime-read")?;
    let report = crate::test_source::source_provider(&text, "source_traits_v1")?;
    let foreign = crate::test_source::source_provider(&text, "foreign_source_traits")?;
    let mut checker = Checker::new(&report)?;
    checker.facts.source_bindings = crate::sema::compiler_items::Catalog::reserve(&foreign)?;
    let before = checker.facts.body_work.canonical_steps;
    let error = checker
        .prepare_early_implementations(&mut 0, usize::MAX)
        .err()
        .ok_or("foreign actual provider catalog was accepted")?;
    assert!(error.message.contains("another provider"));
    assert!(checker.early_implementations.is_none());
    assert!(checker.facts.body_work.canonical_steps > before);
    Ok(())
}

#[test]
fn failed_required_query_charges_work_without_masking_its_semantic_error() -> Result {
    let report =
        crate::test_source::source_provider(&source("missing-clone")?, "source_traits_v1")?;
    let mut checker = Checker::new(&report)?;
    assert!(checker.run().is_err());
    let owner = *owners(&checker, "Copy")?
        .first()
        .ok_or("missing actual Copy header")?;
    let mut pending = Pending::default();
    let plan = plan(&mut checker, &mut pending, owner)?;
    assert!(!plan.requirements.is_empty());
    pending.active.insert(owner, Attempt::new(plan));
    // Both real query failure and cumulative local exhaustion must be observed.
    pending.work = 1_000_000 - 5;
    let before = checker.facts.body_work.canonical_steps;
    let error = checker
        .scalar_impl_requirements(owner, &mut pending)
        .err()
        .ok_or("missing actual Clone prerequisite was accepted")?;
    assert!(
        error
            .message
            .contains("does not satisfy the trait's requirements"),
        "{error}"
    );
    assert!(pending.work > 1_000_000);
    assert!(checker.facts.body_work.canonical_steps > before + 5);
    assert!(pending.active.is_empty());
    assert!(
        !checker
            .facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|r| r.is_validated(owner))
    );
    Ok(())
}
