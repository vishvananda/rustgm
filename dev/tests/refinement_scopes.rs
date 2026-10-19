//! Reconciliation must preserve callable binders and reject jointly cyclic inputs.
use rustgm::types::{
    Primitive, RegionId, Store, TypeId,
    infer::{
        Constructor, Context, Domain, Error, TermId, arguments::Argument, refinement::Refinement,
    },
    substitute::Bound,
};
use std::collections::HashMap;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn ty(argument: Argument) -> Result<TermId, &'static str> {
    if let Argument::Type(id) = argument {
        Ok(id)
    } else {
        Err("missing type input")
    }
}
fn closed() -> Bound {
    Bound::restore_regions(HashMap::new())
}
fn callable(context: &mut Context, store: &Store, parameters: &[TermId]) -> Result<TermId, Error> {
    let mut children = parameters.to_vec();
    children.push(context.known(store, TypeId::UNIT)?);
    context.construct(
        Constructor::Function {
            bound_regions: 2,
            parameters: u32::try_from(parameters.len()).map_err(|_| Error::Overflow)?,
            unsafe_: false,
        },
        &children,
    )
}
fn reference(
    context: &mut Context,
    store: &Store,
    slot: u16,
) -> Result<TermId, Box<dyn std::error::Error>> {
    let byte = context.known(store, TypeId::primitive(Primitive::U8))?;
    Ok(context.construct(
        Constructor::Reference {
            mutable: false,
            region: RegionId::bound(0, slot)?,
        },
        &[byte],
    )?)
}
#[test]
fn caller_partial_callable_cannot_capture_a_local_parameter_lifetime() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut query, copied) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    let parameter = source.variable(Domain::Any)?;
    let function = callable(&mut source, &store, &[parameter])?;
    source.equate(&store, input, function)?;
    let local = reference(query.context(), &store, 0)?;
    let function = callable(query.context(), &store, &[local])?;
    query.context().equate(&store, ty(copied[0])?, function)?;
    let sentinel = source.variable(Domain::Any)?;
    assert!(matches!(
        query.commit(&mut store, &mut source, &mut closed()),
        Err(Error::Mismatch)
    ));
    assert_eq!(source.unresolved_domain(parameter)?, Some(Domain::Any));
    assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
    Ok(())
}
#[test]
fn alpha_equivalent_callable_inputs_add_no_caller_lifetime_equations() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut query, copied) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    for (context, root, slots) in [
        (&mut source, input, [0, 1]),
        (query.context(), ty(copied[0])?, [1, 0]),
    ] {
        let parameters = slots
            .into_iter()
            .map(|slot| reference(context, &store, slot))
            .collect::<Result<Vec<_>, _>>()?;
        let function = callable(context, &store, &parameters)?;
        context.equate(&store, root, function)?;
    }
    let committed = query.commit(&mut store, &mut source, &mut closed())?;
    assert!(!committed.changed);
    assert!(committed.equalities.is_empty());
    assert!(source.region_equalities.is_empty());
    Ok(())
}
#[test]
fn individually_acyclic_parent_and_query_cannot_commit_a_joint_cycle() -> TestResult {
    for merged in [false, true] {
        let mut store = Store::default();
        let mut source = Context::default();
        let first = source.variable(Domain::Any)?;
        let second = source.variable(Domain::Any)?;
        let (mut query, copied) = Refinement::new(
            &store,
            &mut source,
            &[Argument::Type(first), Argument::Type(second)],
        )?;
        let cycle = query
            .context()
            .construct(Constructor::Tuple, &[ty(copied[1])?])?;
        query.context().equate(&store, ty(copied[0])?, cycle)?;
        let other = if merged {
            first
        } else {
            source.construct(Constructor::Tuple, &[first])?
        };
        source.equate(&store, second, other)?;
        let sentinel = source.variable(Domain::Any)?;
        assert!(matches!(
            query.commit(&mut store, &mut source, &mut closed()),
            Err(Error::Occurs)
        ));
        assert_eq!(source.unresolved_domain(first)?, Some(Domain::Any));
        assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
    }
    Ok(())
}
