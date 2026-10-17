//! Opaque result identity is separate from each occurrence's formation obligation.
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    callable::Templates,
    infer::{
        Context, Domain, Error, FreshRegions,
        arguments::Argument,
        deferred::{Outcome, Relation},
    },
    projection::{Application, Schema},
    substitute::Bound,
    traits,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ITEM: ItemId = ItemId(2);
const TRAIT: ItemId = ItemId(1);
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

#[test]
fn member_lookup_skips_deep_or_unresolved_inputs_without_proving_formation() -> TestResult {
    let mut types = store()?;
    let mut ty = BYTE;
    for _ in 0..4096 {
        ty = projected(&mut types, ty)?;
    }
    let mut inference = Context::default();
    let root = inference.known(&types, ty)?;
    let before = inference.steps;
    for _ in 0..128 {
        assert_eq!(inference.projection_item(&types, root)?, Some(ITEM));
    }
    assert_eq!(inference.steps - before, 128);
    let input = inference.variable(Domain::Any)?;
    assert_eq!(input.0, root.0 + 1, "metadata lookup materialized children");
    assert_eq!(inference.projection_item(&types, input)?, None);
    let open = inference.projected(&types, ITEM, input, &[], &[])?;
    assert_eq!(inference.projection_item(&types, open)?, Some(ITEM));
    assert_eq!(inference.unresolved_domain(input)?, Some(Domain::Any));
    assert_eq!(inference.head(&types, open), Err(Error::Normalization));
    let concrete = projected(&mut types, BYTE)?;
    let occurrence = inference.known(&types, concrete)?;
    assert_eq!(inference.projection_item(&types, occurrence)?, Some(ITEM));
    inference.apply_normalization(&mut types, occurrence, BYTE)?;
    assert_eq!(inference.projection_item(&types, occurrence)?, None);
    assert_eq!(
        inference.projection_item(&types, rustgm::types::infer::TermId(u32::MAX)),
        Err(Error::InvalidIdentity)
    );
    Ok(())
}

#[test]
fn resolution_observes_later_projection_publication() -> TestResult {
    let mut types = store()?;
    let original = projected(&mut types, BYTE)?;
    let mut inference = Context::default();
    let projection = inference.known(&types, original)?;
    let tuple = inference.construct(rustgm::types::infer::Constructor::Tuple, &[projection])?;
    let original_fields = types.intern_list(&[original])?;
    let original_tuple = types.intern(Kind::Tuple(original_fields))?;
    assert_eq!(inference.resolved(&mut types, tuple)?, original_tuple);
    inference.apply_normalization(&mut types, projection, BYTE)?;
    let normalized_fields = types.intern_list(&[BYTE])?;
    let normalized_tuple = types.intern(Kind::Tuple(normalized_fields))?;
    assert_eq!(inference.resolved(&mut types, tuple)?, normalized_tuple);
    Ok(())
}

fn store() -> Result<Store, &'static str> {
    let mut types = Store::default();
    types.declare_projection(
        ITEM,
        Schema {
            trait_item: TRAIT,
            sized: true,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    Ok(types)
}
fn projected(types: &mut Store, subject: TypeId) -> Result<TypeId, &'static str> {
    let trait_application = types.intern_trait_application(traits::Application {
        item: TRAIT,
        arguments: GenericArgs::EMPTY,
    })?;
    types.intern(Kind::Projection(Application {
        item: ITEM,
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))
}

#[test]
fn retained_occurrences_require_independent_normalization() -> TestResult {
    let mut types = store()?;
    let ty = projected(&mut types, BYTE)?;
    let mut inference = Context::default();
    let output = inference.known(&types, ty)?;
    let first = inference.retain_projection(&mut types, output)?;
    let second = inference.retain_projection(&mut types, output)?;
    inference.apply_normalization(&mut types, output, ty)?;
    assert!(inference.head(&types, output)?.is_some());
    for source in [first, second] {
        assert_eq!(inference.resolved(&mut types, source)?, ty);
        assert_eq!(inference.head(&types, source), Err(Error::Normalization));
    }
    inference.apply_normalization(&mut types, first, ty)?;
    assert!(inference.head(&types, first)?.is_some());
    assert_eq!(inference.head(&types, second), Err(Error::Normalization));
    Ok(())
}

#[test]
fn unchanged_opaque_roots_wake_their_suspended_equations() -> TestResult {
    let mut types = store()?;
    let ty = projected(&mut types, BYTE)?;
    let mut inference = Context::default();
    let projection = inference.known(&types, ty)?;
    let result = inference.variable(Domain::Any)?;
    let Outcome::Pending(equation) =
        inference.equate_deferred(&types, projection, result, Relation::Equality)?
    else {
        return Err("unproved projection equation completed".into());
    };
    assert!(inference.ready_equation()?.is_none());
    inference.apply_normalization(&mut types, projection, ty)?;
    assert_eq!(inference.ready_equation()?, Some(equation));
    assert_eq!(
        inference.resume_equation(&types, equation)?,
        Outcome::Complete
    );
    assert!(!inference.has_deferred_equations());
    assert_eq!(inference.resolved(&mut types, result)?, ty);
    Ok(())
}

#[test]
fn distinct_results_never_infer_equations_between_their_inputs() -> TestResult {
    let mut types = store()?;
    let left = projected(&mut types, BYTE)?;
    let right = projected(&mut types, TypeId::UNIT)?;
    let mut inference = Context::default();
    let a = inference.known(&types, left)?;
    let b = inference.known(&types, right)?;
    inference.apply_normalization(&mut types, a, left)?;
    inference.apply_normalization(&mut types, b, right)?;
    assert_eq!(inference.equate(&types, a, b), Err(Error::Mismatch));
    assert!(inference.region_equalities.is_empty());
    let input = inference.variable(Domain::Any)?;
    let pending = inference.projected(&types, ITEM, input, &[], &[])?;
    assert_eq!(
        inference.apply_normalization(&mut types, pending, left),
        Err(Error::Unresolved)
    );
    assert_eq!(inference.unresolved_domain(input)?, Some(Domain::Any));
    Ok(())
}

#[test]
fn transformed_lifetime_inputs_receive_no_normalization_authority() -> TestResult {
    let mut types = store()?;
    let mut inference = Context::default();
    let subject = inference.known(&types, BYTE)?;
    let root = inference.projected(
        &types,
        ITEM,
        subject,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
        &[],
    )?;
    let ty = inference.resolved(&mut types, root)?;
    inference.apply_normalization(&mut types, root, ty)?;
    let mapped = inference.map_regions(&mut types, root, &mut Bound::new(vec![RegionId(7)]))?;
    assert_eq!(inference.head(&types, mapped), Err(Error::Normalization));
    let app = inference.projection_application(&types, mapped)?;
    assert_eq!(
        app.predicate.trait_arguments(),
        Some((TRAIT, &[Argument::Lifetime(RegionId(7))][..]))
    );
    Ok(())
}

#[test]
fn inferred_opaque_values_do_not_freshen_operation_inputs() -> TestResult {
    let mut types = store()?;
    let mut inference = Context::default();
    let subject = inference.known(&types, BYTE)?;
    let root = inference.projected(
        &types,
        ITEM,
        subject,
        &[Argument::Lifetime(RegionId(7))],
        &[],
    )?;
    let ty = inference.resolved(&mut types, root)?;
    inference.apply_normalization(&mut types, root, ty)?;
    let FreshRegions {
        term: fresh,
        regions: count,
        needs_normalization,
    } = inference.fresh_regions(&mut types, &Templates::default(), root, 12)?;
    assert_eq!(count, 0);
    assert!(!needs_normalization);
    assert_eq!(inference.resolved(&mut types, fresh)?, ty);
    Ok(())
}

#[test]
fn rebuilt_callable_results_need_fresh_occurrence_proofs_without_new_lifetimes() -> TestResult {
    use rustgm::types::{
        lifetime_proof::{Obligations, ScopeId},
        parameters::{Parameter, ParameterKind},
    };
    let mut types = store()?;
    let owner = ItemId(3);
    let parameter = types.intern_parameter(Parameter {
        owner,
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = types.intern(Kind::Parameter(parameter))?;
    let result = projected(&mut types, subject)?;
    let parameters = types.intern_list(&[subject, result])?;
    let signature = types.intern(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
    })?;
    let mut templates = Templates::default();
    templates.insert_generic(&types, owner, signature, 0, &[], &[parameter])?;
    let mut inference = Context::default();
    let byte = inference.known(&types, BYTE)?;
    let signature =
        inference.instantiate(&mut types, signature, &[(parameter, Argument::Type(byte))])?;
    let mut scopes = Obligations::new(1);
    let roots = inference.normalization_roots(&types, signature, ScopeId::ROOT, |parent, n| {
        scopes.scope(parent, n).map_err(|_| Error::InvalidIdentity)
    })?;
    assert_eq!(roots.len(), 1);
    for (_, root) in roots {
        inference.apply_normalization(&mut types, root, BYTE)?;
    }
    let item = inference.function_item(owner, signature, &[Argument::Type(byte)])?;
    let original = inference.resolved(&mut types, item)?;
    for _ in 0..2 {
        let fresh = inference.fresh_regions(&mut types, &templates, item, 10)?;
        assert_eq!(fresh.regions, 0);
        assert!(fresh.needs_normalization);
        let roots =
            inference.normalization_roots(&types, fresh.term, ScopeId::ROOT, |parent, n| {
                scopes.scope(parent, n).map_err(|_| Error::InvalidIdentity)
            })?;
        assert_eq!(roots.len(), 1);
        for (_, root) in roots {
            assert_eq!(inference.head(&types, root), Err(Error::Normalization));
            inference.apply_normalization(&mut types, root, BYTE)?;
        }
        assert_eq!(inference.resolved(&mut types, fresh.term)?, original);
    }
    Ok(())
}

#[test]
fn projected_arguments_retain_work_even_with_unprojected_callable_templates() -> TestResult {
    use rustgm::types::{
        lifetime_proof::{Obligations, ScopeId},
        parameters::{Parameter, ParameterKind},
    };
    let mut types = store()?;
    let owner = ItemId(3);
    let parameter = types.intern_parameter(Parameter {
        owner,
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = types.intern(Kind::Parameter(parameter))?;
    let parameters = types.intern_list(&[subject])?;
    let signature = types.intern(Kind::Function {
        parameters,
        result: subject,
        bound_regions: 0,
        unsafe_: false,
    })?;
    assert_eq!(types.contains_projections(signature), Some(false));
    let mut templates = Templates::default();
    templates.insert_generic(&types, owner, signature, 0, &[], &[parameter])?;
    let mut inference = Context::default();
    let result = projected(&mut types, BYTE)?;
    let argument = inference.known(&types, result)?;
    inference.apply_normalization(&mut types, argument, result)?;
    let signature = inference.instantiate(
        &mut types,
        signature,
        &[(parameter, Argument::Type(argument))],
    )?;
    let item = inference.function_item(owner, signature, &[Argument::Type(argument)])?;
    let fresh = inference.fresh_regions(&mut types, &templates, item, 10)?;
    assert_eq!(fresh.regions, 0);
    assert!(fresh.needs_normalization);
    let mut scopes = Obligations::new(1);
    let roots = inference.normalization_roots(&types, fresh.term, ScopeId::ROOT, |parent, n| {
        scopes.scope(parent, n).map_err(|_| Error::InvalidIdentity)
    })?;
    assert!(roots.iter().any(|(scope, _)| *scope != ScopeId::ROOT));
    assert!(roots.iter().any(|(scope, _)| *scope == ScopeId::ROOT));
    Ok(())
}
