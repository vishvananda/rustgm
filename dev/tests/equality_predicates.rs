//! Equality obligations retain both graphs without equating their rigid inputs.
#[path = "equality_predicates/scoped_views.rs"]
mod scoped_views;
#[path = "equality_predicates/transport.rs"]
mod transport;
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind, Constant},
    bound_uses,
    infer::{self, Constructor, Context, Domain, Instantiation},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        environment::{Abilities, Environment},
        predicates::{Bound, Predicate},
        uses::Uses,
    },
    projection::{Application, Schema},
    substitute, traits,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const WORD: TypeId = TypeId::primitive(Primitive::U16);
const SIZE: TypeId = TypeId::primitive(Primitive::Usize);

fn projection(
    store: &mut Store,
    subject: TypeId,
    arguments: &[Argument],
) -> Result<TypeId, &'static str> {
    store.declare_projection(
        ItemId(2),
        Schema {
            trait_item: ItemId(1),
            sized: false,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    let arguments = store.intern_arguments(arguments)?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments,
    })?;
    store.intern(Kind::Projection(Application {
        item: ItemId(2),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))
}

fn parameter(
    store: &mut Store,
    index: u32,
    kind: ParameterKind,
) -> Result<ParameterId, &'static str> {
    store.intern_parameter(Parameter {
        owner: ItemId(3),
        index,
        kind,
    })
}

fn borrowed(store: &mut Store, region: RegionId, referent: TypeId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent,
    })
}

#[test]
fn equality_result_occurrences_are_not_lost() -> TestResult {
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: false })?;
    let n = parameter(&mut store, 1, ParameterKind::Const { ty: SIZE })?;
    let element = store.intern(Kind::Parameter(t))?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let array = store.intern(Kind::ArrayTerm { element, length })?;
    let result = borrowed(&mut store, RegionId::bound(0, 2)?, array)?;
    let projected = projection(&mut store, BYTE, &[])?;
    let predicate = Predicate {
        ty: projected,
        bound: Bound::ProjectionEqual(result),
    };
    assert!(predicate.contains_projections(&store)?);
    assert!(predicate.contains_parameters(&store)?);
    assert!(predicate.has_free_regions(&store)?);
    assert_eq!(predicate.escaping_binders(&store)?, 1);
    let mut uses = Uses::default();
    uses.predicate(&store, predicate)?;
    assert_eq!(uses.parameters, [t].into());
    assert_eq!(uses.constants, [n].into());
    let steps = uses.steps;
    uses.predicate(&store, predicate)?;
    assert!(uses.steps <= steps + 2);
    let mut regions = bound_uses::Uses::default();
    regions.predicate(&store, predicate)?;
    assert_eq!(regions.indices, [2].into());
    let invalid = Predicate {
        ty: projected,
        bound: Bound::ProjectionEqual(TypeId(u32::MAX)),
    };
    assert!(invalid.contains_projections(&store).is_err());
    assert!(invalid.contains_parameters(&store).is_err());
    assert!(invalid.has_free_regions(&store).is_err());
    assert!(invalid.escaping_binders(&store).is_err());
    Ok(())
}

#[test]
fn substitution_shares_both_roots_and_preserves_local_binders() -> TestResult {
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: true })?;
    let parameter_ty = store.intern(Kind::Parameter(t))?;
    let subject = borrowed(&mut store, RegionId::bound(0, 0)?, parameter_ty)?;
    let ty = projection(&mut store, subject, &[])?;
    let local = borrowed(&mut store, RegionId::bound(0, 0)?, parameter_ty)?;
    let capture = borrowed(&mut store, RegionId::bound(1, 0)?, parameter_ty)?;
    let parameters = store.intern_list(&[local])?;
    let result = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: capture,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let predicate = Predicate {
        ty,
        bound: Bound::ProjectionEqual(result),
    };
    let mut fold = substitute::Bound::parameters_captured(
        &store,
        &[(t, Argument::Type(WORD))],
        vec![RegionId(9)],
    )?;
    let substituted = fold.predicate(&mut store, predicate)?;
    let subject = borrowed(&mut store, RegionId(9), WORD)?;
    let expected_ty = projection(&mut store, subject, &[])?;
    let local = borrowed(&mut store, RegionId::bound(0, 0)?, WORD)?;
    let capture = borrowed(&mut store, RegionId(9), WORD)?;
    let parameters = store.intern_list(&[local])?;
    let result = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: capture,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    assert_eq!(
        substituted,
        Predicate {
            ty: expected_ty,
            bound: Bound::ProjectionEqual(result)
        }
    );
    let steps = fold.steps;
    assert_eq!(fold.predicate(&mut store, predicate)?, substituted);
    assert_eq!(fold.steps, steps);
    assert!(!substituted.contains_parameters(&store)?);
    assert_eq!(substituted.escaping_binders(&store)?, 0);
    assert!(substituted.has_free_regions(&store)?);
    Ok(())
}

#[test]
fn inferred_results_share_inputs_and_imports_stay_isolated() -> TestResult {
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: true })?;
    let n = parameter(&mut store, 1, ParameterKind::Const { ty: SIZE })?;
    let element = store.intern(Kind::Parameter(t))?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let ty = projection(&mut store, element, &[Argument::Const(length)])?;
    let result = store.intern(Kind::ArrayTerm { element, length })?;
    let predicate = Predicate {
        ty,
        bound: Bound::ProjectionEqual(result),
    };
    let mut context = Context::default();
    let input = context.variable(Domain::Any)?;
    let count = context.const_variable(&store, SIZE)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &store,
        &[
            (t, infer::arguments::Argument::Type(input)),
            (n, infer::arguments::Argument::Const(count)),
        ],
    )?;
    let inferred = context.instantiate_predicate(&mut store, predicate, &mut substitution)?;
    assert!(inferred.trait_arguments().is_none());
    assert!(inferred.builtin_bound().is_none());
    let result_term = inferred
        .projection_result()
        .ok_or("missing inferred result")?;
    let mut probe = Context::default();
    let imported = probe.import_predicate(&store, &mut context, &inferred)?;
    let imported_result = imported
        .projection_result()
        .ok_or("missing imported result")?;
    let word = probe.known(&store, WORD)?;
    let array = probe.construct(Constructor::Array(3), &[word])?;
    probe.equate(&store, imported_result, array)?;
    let three = store.intern_constant(Constant { ty: SIZE, bits: 3 })?;
    let expected_ty = projection(&mut store, WORD, &[Argument::Const(three)])?;
    let expected_result = store.intern(Kind::Array {
        element: WORD,
        length: 3,
    })?;
    assert_eq!(
        probe.resolved_predicate(&mut store, &imported)?,
        Predicate {
            ty: expected_ty,
            bound: Bound::ProjectionEqual(expected_result)
        }
    );
    assert_eq!(
        context.resolved_predicate(&mut store, &inferred),
        Err(infer::Error::Unresolved)
    );
    let byte = context.known(&store, BYTE)?;
    let array = context.construct(Constructor::Array(5), &[byte])?;
    context.equate(&store, result_term, array)?;
    assert_eq!(context.resolved(&mut store, input)?, BYTE);
    let five = store.intern_constant(Constant { ty: SIZE, bits: 5 })?;
    assert_eq!(context.resolved_const(&mut store, count)?, five);
    let expected_ty = projection(&mut store, BYTE, &[Argument::Const(five)])?;
    let expected_result = store.intern(Kind::Array {
        element: BYTE,
        length: 5,
    })?;
    let (_, finished) = context.finish_with_predicates(&mut store, [&inferred].into_iter())?;
    assert_eq!(
        finished,
        [Predicate {
            ty: expected_ty,
            bound: Bound::ProjectionEqual(expected_result)
        }]
    );
    Ok(())
}

#[test]
fn assumptions_preserve_competing_results_without_granting_abilities() -> TestResult {
    let mut store = Store::default();
    let ty = projection(&mut store, BYTE, &[])?;
    let a = Predicate {
        ty,
        bound: Bound::ProjectionEqual(BYTE),
    };
    let b = Predicate {
        ty,
        bound: Bound::ProjectionEqual(WORD),
    };
    let original = Environment::new(&store, &[])?;
    let environment = original.clone().with_predicates(&store, &[a, b, a])?;
    assert_eq!(
        environment
            .projection_clauses()
            .map(rustgm::types::parameters::clauses::Clause::ordinary)
            .collect::<Result<Vec<_>, _>>()?,
        [a, b]
    );
    assert_eq!(environment.trait_clauses().count(), 0);
    assert_eq!(original.projection_clauses().count(), 0);
    assert!(!environment.assumes_sized(ty));
    assert!(!environment.has_shape_assumptions());
    assert_eq!(environment.assumed(ty), Abilities::default());
    let invalid = Predicate {
        ty: BYTE,
        bound: Bound::ProjectionEqual(WORD),
    };
    assert!(original.with_predicates(&store, &[invalid]).is_err());
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let invalid = Predicate {
        ty: TypeId(u32::MAX),
        ..invalid
    };
    assert!(
        context
            .instantiate_predicate(&mut store, invalid, &mut substitution)
            .is_err()
    );
    Ok(())
}

#[test]
fn a_result_equality_waits_for_normalization_without_determining_self() -> TestResult {
    use infer::deferred::{Outcome, Relation};
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: true })?;
    let u = parameter(&mut store, 1, ParameterKind::Type { sized: true })?;
    let input_ty = store.intern(Kind::Parameter(t))?;
    let result_ty = store.intern(Kind::Parameter(u))?;
    let ty = projection(&mut store, input_ty, &[])?;
    let predicate = Predicate {
        ty,
        bound: Bound::ProjectionEqual(result_ty),
    };
    let mut context = Context::default();
    let input = context.variable(Domain::Any)?;
    let result = context.variable(Domain::Any)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &store,
        &[
            (t, infer::arguments::Argument::Type(input)),
            (u, infer::arguments::Argument::Type(result)),
        ],
    )?;
    let inferred = context.instantiate_predicate(&mut store, predicate, &mut substitution)?;
    let result = inferred.projection_result().ok_or("missing result")?;
    let Outcome::Pending(equation) =
        context.equate_deferred(&store, inferred.subject(), result, Relation::Equality)?
    else {
        return Err("projection equality was discharged without normalization".into());
    };
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, result, byte)?;
    assert_eq!(context.unresolved_domain(input)?, Some(Domain::Any));
    assert!(context.has_deferred_equations());
    let word = context.known(&store, WORD)?;
    context.equate(&store, input, word)?;
    context.apply_normalization(&mut store, inferred.subject(), BYTE)?;
    assert_eq!(
        context.resume_equation(&store, equation)?,
        Outcome::Complete
    );
    let (_, finished) = context.finish_with_predicates(&mut store, [&inferred].into_iter())?;
    assert_eq!(
        finished,
        [Predicate {
            ty: BYTE,
            bound: Bound::ProjectionEqual(BYTE)
        }]
    );
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let normalized = context.instantiate_predicate(&mut store, finished[0], &mut substitution)?;
    assert_eq!(
        context.resolved_predicate(&mut store, &normalized)?,
        finished[0]
    );
    Ok(())
}

#[test]
fn a_constant_occurring_only_in_the_result_cannot_be_discarded() -> TestResult {
    let mut store = Store::default();
    let n = parameter(&mut store, 0, ParameterKind::Const { ty: SIZE })?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let result = store.intern(Kind::ArrayTerm {
        element: BYTE,
        length,
    })?;
    let ty = projection(&mut store, WORD, &[])?;
    let predicate = Predicate {
        ty,
        bound: Bound::ProjectionEqual(result),
    };
    let mut context = Context::default();
    let count = context.const_variable(&store, SIZE)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &store,
        &[(n, infer::arguments::Argument::Const(count))],
    )?;
    let inferred = context.instantiate_predicate(&mut store, predicate, &mut substitution)?;
    assert!(matches!(
        context.finish_with_predicates(&mut store, [&inferred].into_iter()),
        Err(infer::Error::Unresolved)
    ));
    Ok(())
}

#[test]
fn result_only_parameters_and_constants_require_the_actual_environment() -> TestResult {
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: false })?;
    let n = parameter(&mut store, 1, ParameterKind::Const { ty: SIZE })?;
    let element = store.intern(Kind::Parameter(t))?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let result = store.intern(Kind::ArrayTerm { element, length })?;
    let ty = projection(&mut store, BYTE, &[])?;
    let predicate = Predicate {
        ty,
        bound: Bound::ProjectionEqual(result),
    };
    let empty = Environment::new(&store, &[])?;
    assert!(empty.with_predicates(&store, &[predicate]).is_err());
    let type_only = Environment::new(&store, &[(t, Abilities::default())])?;
    assert!(
        type_only
            .clone()
            .with_predicates(&store, &[predicate])
            .is_err()
    );
    let environment = type_only
        .with_const_parameters(&store, &[n])?
        .with_predicates(&store, &[predicate])?;
    assert_eq!(environment.abilities(t), Some(Abilities::default()));
    assert!(!environment.assumes_sized(element));
    assert_eq!(environment.projection_clauses().count(), 1);
    Ok(())
}

#[test]
fn normalized_views_preserve_original_roots_and_shared_holes() -> TestResult {
    let mut store = Store::default();
    let projected = projection(&mut store, BYTE, &[])?;
    let mut context = Context::default();
    let root = context.known(&store, projected)?;
    let hole = context.variable(Domain::Any)?;
    let original = context.construct(Constructor::Tuple, &[root, hole, root])?;
    let view = context.rewritten_projections(&store, original, &[(root, WORD)].into())?;
    let elements = store.intern_list(&[WORD, BYTE, WORD])?;
    let expected = store.intern(Kind::Tuple(elements))?;
    let known = context.known(&store, expected)?;
    context.equate(&store, view, known)?;
    assert_eq!(context.resolved(&mut store, hole)?, BYTE);
    assert_eq!(context.resolved(&mut store, root)?, projected);
    assert_eq!(context.resolved(&mut store, view)?, expected);
    let elements = store.intern_list(&[projected, BYTE, projected])?;
    let expected_original = store.intern(Kind::Tuple(elements))?;
    assert_eq!(context.resolved(&mut store, original)?, expected_original);
    assert!(
        context
            .rewritten_projections(&store, original, &[(original, WORD)].into())
            .is_err()
    );
    Ok(())
}

#[test]
fn proved_inputs_refine_independent_holes_without_inverting_projections() -> TestResult {
    let mut store = Store::default();
    let projected = projection(&mut store, BYTE, &[])?;
    let id = parameter(&mut store, 0, ParameterKind::Type { sized: true })?;
    let ty = store.intern(Kind::Parameter(id))?;
    let elements = store.intern_list(&[projected, ty])?;
    let ty = store.intern(Kind::Tuple(elements))?;
    let mut context = Context::default();
    let hole = context.variable(Domain::Any)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &store,
        &[(id, infer::arguments::Argument::Type(hole))],
    )?;
    let goal = context.instantiate_predicate(
        &mut store,
        Predicate {
            ty,
            bound: Bound::Sized,
        },
        &mut substitution,
    )?;
    let elements = store.intern_list(&[projected, WORD])?;
    let proof = Predicate {
        ty: store.intern(Kind::Tuple(elements))?,
        bound: Bound::Sized,
    };
    context.equate_proved_predicate(&mut store, &goal, proof)?;
    assert_eq!(context.resolved(&mut store, hole)?, WORD);
    let unrelated = projection(&mut store, WORD, &[])?;
    let elements = store.intern_list(&[unrelated, WORD])?;
    let wrong = Predicate {
        ty: store.intern(Kind::Tuple(elements))?,
        bound: Bound::Sized,
    };
    assert_eq!(
        context.equate_proved_predicate(&mut store, &goal, wrong),
        Err(infer::Error::Normalization)
    );
    Ok(())
}

#[test]
fn repeated_projected_predicate_inputs_share_one_transformed_graph() -> TestResult {
    let mut store = Store::default();
    let projected = projection(&mut store, BYTE, &[])?;
    let width = 256;
    let elements = store.intern_list(&vec![projected; width])?;
    let original = store.intern(Kind::Tuple(elements))?;
    let arguments = store.intern_arguments(&vec![Argument::Type(original); width])?;
    let application = store.intern_trait_application(traits::Application {
        item: ItemId(9),
        arguments,
    })?;
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let goal = context.instantiate_predicate(
        &mut store,
        Predicate {
            ty: BYTE,
            bound: Bound::Trait(application),
        },
        &mut substitution,
    )?;
    let before = context.steps;
    let roots = context.predicate_normalization_roots(
        &store,
        &goal,
        rustgm::types::lifetime_proof::ScopeId::ROOT,
        |_, _| Err(infer::Error::InvalidIdentity),
    )?;
    assert_eq!(roots.len(), 1);
    let view = context.rewritten_projection_inputs(&store, &goal, &[(roots[0].1, WORD)].into())?;
    let (_, arguments) = view
        .trait_arguments()
        .ok_or("missing normalized trait arguments")?;
    let first = arguments[0];
    assert!(arguments.iter().all(|argument| *argument == first));
    assert!(
        context.steps - before < width * 20,
        "{}",
        context.steps - before
    );
    let infer::arguments::Argument::Type(transformed) = first else {
        return Err("expected type input".into());
    };
    let elements = store.intern_list(&vec![WORD; width])?;
    let expected = store.intern(Kind::Tuple(elements))?;
    assert_eq!(context.resolved(&mut store, transformed)?, expected);
    assert_eq!(context.resolved(&mut store, roots[0].1)?, projected);
    let arguments = store.intern_arguments(&vec![Argument::Type(expected); width])?;
    let application = store.intern_trait_application(traits::Application {
        item: ItemId(9),
        arguments,
    })?;
    let expected = Predicate {
        ty: BYTE,
        bound: Bound::Trait(application),
    };
    let before = context.steps;
    assert_eq!(context.resolved_predicate(&mut store, &view)?, expected);
    assert!(context.steps - before < width * 20);
    let header = context.instantiate_predicate(&mut store, expected, &mut substitution)?;
    let before = context.steps;
    context.equate_predicates(&store, &view, &header)?;
    assert!(context.steps - before < width * 20);
    let before = context.steps;
    context.equate_trait_predicate(&mut store, &view, expected)?;
    assert!(context.steps - before < width * 20);
    Ok(())
}
