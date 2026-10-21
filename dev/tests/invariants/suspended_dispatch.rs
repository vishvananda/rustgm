//! Retained dispatch state measures cached proof children once, then only new payload.
use super::*;
use crate::types::{RegionId, lifetime_proof::ScopeId};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn append(dispatch: &mut Dispatcher) -> TestResult {
    let first = dispatch.obligations.regions.end;
    let mut incoming = Obligations::new(first);
    let scope = incoming.scope(ScopeId::ROOT, 1)?;
    let mut child = Obligations::new(first);
    child.regions.end = first + 1;
    child.outlives.push((RegionId(first), RegionId::STATIC));
    incoming.append_at(child, scope)?;
    dispatch.extend_evidence(incoming)?;
    Ok(())
}
#[test]
fn snapshots_do_not_revisit_old_proof_children() -> TestResult {
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut dispatch = Dispatcher::new(0, 1)?;
        for _ in 0..width {
            append(&mut dispatch)?;
        }
        assert!(dispatch.retained.is_none());
        let (bytes, first) = dispatch.retained_allocation();
        let (again, repeated) = dispatch.retained_allocation();
        assert_eq!(bytes, again);
        assert!(first > repeated);
        let full = dispatch.obligations.retained_allocation();
        let shallow = dispatch.obligations.shallow_allocation();
        assert_eq!(
            dispatch
                .retained
                .as_ref()
                .ok_or("missing retention")?
                .children,
            full.bytes - shallow.bytes
        );
        append(&mut dispatch)?;
        let (_, updated) = dispatch.retained_allocation();
        let full = dispatch.obligations.retained_allocation();
        let shallow = dispatch.obligations.shallow_allocation();
        assert_eq!(
            dispatch
                .retained
                .as_ref()
                .ok_or("missing retention")?
                .children,
            full.bytes - shallow.bytes
        );
        observations.push((width, first, repeated, updated));
    }
    assert_eq!(observations[0].2, observations[1].2);
    assert_eq!(observations[1].2, observations[2].2);
    assert_eq!(observations[0].3, observations[1].3);
    assert_eq!(observations[1].3, observations[2].3);
    println!("dispatch allocation visits: {observations:?}");
    Ok(())
}
#[test]
fn flat_proof_growth_uses_current_vector_capacity() -> TestResult {
    let mut dispatch = Dispatcher::new(0, 1)?;
    dispatch.retained_allocation();
    for _ in 0..128 {
        let mut proof = Obligations::new(dispatch.obligations.regions.end);
        proof.outlives.push((RegionId::STATIC, RegionId::STATIC));
        dispatch.extend_evidence(proof)?;
    }
    dispatch.retained_allocation();
    assert_eq!(
        dispatch
            .retained
            .as_ref()
            .ok_or("missing retention")?
            .children,
        0
    );
    assert_eq!(
        dispatch.obligations.retained_allocation().bytes,
        dispatch.obligations.shallow_allocation().bytes
    );
    Ok(())
}

#[test]
fn reserved_dispatch_storage_is_charged_without_initializing_elements() -> TestResult {
    let mut dispatch = Dispatcher::new(0, 1)?;
    dispatch
        .obligations
        .equalities
        .reserve((64 * 1024 * 1024) / size_of::<(RegionId, RegionId)>() + 1);
    assert!(dispatch.obligations.equalities.is_empty());
    assert!(dispatch.retained_allocation().0 > 64 * 1024 * 1024);
    Ok(())
}

#[test]
fn allocation_inspection_does_not_resolve_or_constrain_inference() -> TestResult {
    use crate::types::{
        Store,
        infer::{Context, Domain, Error},
    };
    let mut context = Context::default();
    assert_eq!(context.retained_allocation().0, 0);
    let value = context.variable(Domain::Any)?;
    let before = context.steps;
    assert!(context.retained_allocation().0 > 0);
    assert_eq!(context.steps, before);
    assert_eq!(
        context.resolved(&mut Store::default(), value),
        Err(Error::Unresolved)
    );
    Ok(())
}

#[test]
fn tracked_override_keeps_existing_evidence_and_nested_lifetime_debts() -> TestResult {
    let (mut facts, signature) = projected_signature()?;
    let mut original = Dispatcher::new(0, 1)?;
    append(&mut original)?;
    let mut tracked = Dispatcher::new(0, 1)?;
    append(&mut tracked)?;
    tracked.retained_allocation();
    original
        .obligations
        .equalities
        .push((RegionId(1), RegionId(1)));
    tracked
        .obligations
        .equalities
        .push((RegionId(1), RegionId(1)));
    let prefix = original.obligations.equalities.clone();
    let scopes = original.obligations.scope_count();
    let ordinary = original.normalize_override(&mut facts, signature)?;
    let retained = tracked.normalize_override(&mut facts, signature)?;
    assert_eq!(ordinary, retained);
    let ordinary = &original.obligations;
    let retained = &tracked.obligations;
    assert!(ordinary.equalities.starts_with(&prefix));
    assert_eq!(ordinary.regions, retained.regions);
    assert_eq!(ordinary.equalities, retained.equalities);
    assert_eq!(ordinary.outlives, retained.outlives);
    assert_eq!(ordinary.types, retained.types);
    assert!(ordinary.scope_count() > scopes);
    assert_eq!(ordinary.scope_count(), retained.scope_count());
    for index in 1..=ordinary.scope_count() {
        let scope = ScopeId(u32::try_from(index)?);
        assert_eq!(
            ordinary.scope_plan().binder(scope)?,
            retained.scope_plan().binder(scope)?
        );
    }
    assert!(require(&facts, ordinary).is_err());
    assert!(require(&facts, retained).is_err());
    let full = retained.retained_allocation();
    let shallow = retained.shallow_allocation();
    assert_eq!(
        tracked.retained.as_ref().ok_or("missing tracker")?.children,
        full.bytes - shallow.bytes
    );
    Ok(())
}

fn require(facts: &Declarations, proof: &Obligations) -> Result<(), String> {
    use crate::types::{
        outlives::{Bounds, Queries},
        relations::Constraints,
        variance::Variances,
    };
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = crate::types::callable::Templates::default();
    let mut queries = Queries::new(&facts.types, &bounds, &callables);
    let mut constraints =
        Constraints::new(&facts.types, &variances, 0, 100_000).map_err(|e| e.to_string())?;
    proof.require_source(&mut constraints, &mut queries, 0)?;
    constraints.prove().map(|_| ()).map_err(|e| e.to_string())
}

fn projected_signature() -> Result<(Declarations, TypeId), Box<dyn std::error::Error>> {
    use crate::types::{
        GenericArgs, ItemId, Kind, arguments::Argument, callable::Convention,
        parameters::predicates::Bound, projection, traits,
    };
    let report = crate::test_source::resolve(
        "trait A<'a>{type Out;}struct S;impl<'a:'static>A<'a>for S{type Out=&'a u8;}",
        true,
    )?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let header = facts
        .generics
        .implementations()
        .next()
        .ok_or("missing impl")?;
    let subject = header.predicate.ty;
    let item = ItemId(header.types[0].0.0);
    let Bound::Trait(application) = header.predicate.bound else {
        return Err("missing trait".into());
    };
    let owner = facts
        .types
        .trait_application(application)
        .ok_or("missing application")?
        .item;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
    let trait_application = facts.types.intern_trait_application(traits::Application {
        item: owner,
        arguments,
    })?;
    let result = facts
        .types
        .intern(Kind::Projection(projection::Application {
            item,
            subject,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
    let parameters = facts.types.intern_list(&[])?;
    let signature = facts.types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    Ok((facts, signature))
}

#[test]
fn copied_caller_payload_and_later_indexes_are_charged_once() -> TestResult {
    use crate::types::parameters::ParameterId;
    let report = crate::test_source::resolve(
        "trait A<'a>{}fn owner<'a,'b:'a,T:A<'a>+Copy>(x:&'b T){}",
        true,
    )?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let owner = facts.functions[0].definition;
    let mut dispatch = Dispatcher::pattern(&mut facts, 2, 1, Some(owner))?;
    let caller = dispatch.caller.as_ref().ok_or("missing caller")?;
    let environment = caller
        .environment
        .as_ref()
        .ok_or("missing mapped environment")?;
    let shared = caller
        .shared_environment
        .as_ref()
        .ok_or("missing declaration environment")?;
    assert!(!std::sync::Arc::ptr_eq(environment, shared));
    let (before, _) = dispatch.retained_allocation();
    assert!(dispatch.retained.as_ref().ok_or("missing tracker")?.owned > 0);
    assert_eq!(dispatch.retained_allocation().0, before);
    let caller = dispatch.caller.as_ref().ok_or("missing caller")?;
    let environment = caller
        .environment
        .as_ref()
        .ok_or("missing mapped environment")?;
    let (index, _) = environment.trait_index(&facts.types)?;
    let index = index.ok_or("missing mapped index")?;
    let (indexed, _) = dispatch.retained_allocation();
    assert!(indexed > before);
    assert_eq!(dispatch.retained_allocation().0, indexed);
    index.parameter_traits(ParameterId(0));
    let caller = dispatch.caller.as_ref().ok_or("missing caller")?;
    caller.assumptions.parameter_traits(ParameterId(0));
    let (cached, _) = dispatch.retained_allocation();
    assert!(cached > indexed);
    let (_, visits) = dispatch.retained_allocation();
    assert_eq!(dispatch.retained_allocation(), (cached, visits));
    println!(
        "mapped caller retained bytes {before}/{indexed}/{cached}, repeat visits {visits}; Caller size {} (prior {}), ordinary Dispatcher caller=None",
        size_of::<super::super::Caller>(),
        size_of::<(
            Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
            crate::sema::traits::assumptions::Assumptions
        )>()
    );
    Ok(())
}
