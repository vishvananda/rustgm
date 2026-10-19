//! Complete predicate transport keeps mixed roots correlated while publishing only inputs.
use super::{BYTE, SIZE, TestResult, WORD, parameter, projection};
use rustgm::types::{
    RegionId, Store,
    arguments::{ConstKind, Constant},
    infer::{self, Constructor, Context, Domain, Instantiation, refinement::Refinement},
    parameters::{
        ParameterKind,
        predicates::{Bound, Predicate},
    },
    substitute,
};
#[test]
fn result_only_type_and_const_holes_participate_in_refinement() -> TestResult {
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: true })?;
    let n = parameter(&mut store, 1, ParameterKind::Const { ty: SIZE })?;
    let element = store.intern(rustgm::types::Kind::Parameter(t))?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let array = store.intern(rustgm::types::Kind::ArrayTerm { element, length })?;
    let projected = projection(&mut store, BYTE, &[])?;
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
    let original = context.instantiate_predicate(
        &mut store,
        Predicate {
            ty: projected,
            bound: Bound::ProjectionEqual(array),
        },
        &mut substitution,
    )?;
    let watch = context.watch_predicate(&original)?;
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    assert!(!context.watch_resolved(watch)?);
    let (mut query, copied) = Refinement::predicate(&store, &mut context, &original)?;
    let result = copied
        .projection_result()
        .ok_or("missing equality result")?;
    let word = query.context.known(&store, WORD)?;
    let fixed = query.context.construct(Constructor::Array(3), &[word])?;
    query.context.equate(&store, result, fixed)?;
    let mut closing = substitute::Bound::parameters(&store, &[])?;
    assert!(query.publish(&mut store, &mut context, &mut closing)?);
    assert_eq!(context.resolved(&mut store, input)?, WORD);
    let three = store.intern_constant(Constant { ty: SIZE, bits: 3 })?;
    assert_eq!(context.resolved_const(&mut store, count)?, three);
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    assert!(context.watch_resolved(watch)?);
    let completed = context.resolved_predicate(&mut store, &original)?;
    assert_eq!(completed.ty, projected);
    Ok(())
}
#[test]
fn outlives_lifetime_is_a_mixed_input_and_cannot_change_argument_kind() -> TestResult {
    use infer::arguments::Argument as Input;
    let mut store = Store::default();
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let original = context.instantiate_predicate(
        &mut store,
        Predicate {
            ty: BYTE,
            bound: Bound::Outlives(RegionId(19)),
        },
        &mut substitution,
    )?;
    let roots: Vec<_> = original.arguments().collect();
    assert_eq!(
        roots,
        [
            Input::Type(original.subject()),
            Input::Lifetime(RegionId(19))
        ]
    );
    let (mut query, copied) = Refinement::predicate(&store, &mut context, &original)?;
    let replaced = copied.map_arguments(|input| {
        Ok(match input {
            Input::Lifetime(_) => Input::Lifetime(RegionId(23)),
            other => other,
        })
    })?;
    assert_eq!(
        query.context.resolved_predicate(&mut store, &replaced)?,
        Predicate {
            ty: BYTE,
            bound: Bound::Outlives(RegionId(23))
        }
    );
    assert!(matches!(
        copied.map_arguments(|_| Ok(Input::Type(copied.subject()))),
        Err(infer::Error::InvalidIdentity)
    ));
    Ok(())
}
