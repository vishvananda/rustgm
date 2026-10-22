//! Selected partial inputs commit through current caller constraints and explicit lifetime closure.
use rustgm::support::hash::HashMap;
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::Constant,
    infer::{
        Constructor, Context, Domain, Error, TermId, arguments::Argument, refinement::Refinement,
    },
    substitute::Bound,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
fn ty(argument: Argument) -> Result<TermId, &'static str> {
    if let Argument::Type(id) = argument {
        Ok(id)
    } else {
        Err("missing type input")
    }
}
fn closed() -> Bound {
    Bound::restore_regions(HashMap::default())
}

#[test]
fn unchanged_inputs_keep_their_holes_and_allocate_no_replacements() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Integer)?;
    let (query, _) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    let sentinel = source.variable(Domain::Any)?;
    let committed = query.commit(&mut store, &mut source, &mut closed())?;
    assert!(!committed.changed);
    assert!(committed.equalities.is_empty());
    assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
    assert_eq!(source.unresolved_domain(input)?, Some(Domain::Integer));
    Ok(())
}

#[test]
fn narrowed_and_merged_inputs_keep_a_shared_numeric_domain() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let a = source.variable(Domain::Any)?;
    let b = source.variable(Domain::Any)?;
    let (mut query, inputs) =
        Refinement::new(&store, &mut source, &[Argument::Type(a), Argument::Type(b)])?;
    let number = query.context().variable(Domain::Integer)?;
    query.context().equate(&store, ty(inputs[0])?, number)?;
    query.context().equate(&store, ty(inputs[1])?, number)?;
    assert!(
        query
            .commit(&mut store, &mut source, &mut closed())?
            .changed
    );
    assert_eq!(source.root(a)?, source.root(b)?);
    assert_eq!(source.unresolved_domain(a)?, Some(Domain::Integer));
    let byte = source.known(&store, BYTE)?;
    source.equate(&store, a, byte)?;
    assert_eq!(source.resolved(&mut store, b)?, BYTE);
    Ok(())
}

#[test]
fn fresh_partial_holes_remain_shared_across_mixed_type_and_const_inputs() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let a = source.variable(Domain::Any)?;
    let b = source.variable(Domain::Any)?;
    let length = source.const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let (mut query, inputs) = Refinement::new(
        &store,
        &mut source,
        &[
            Argument::Type(a),
            Argument::Type(b),
            Argument::Const(length),
        ],
    )?;
    let element = query.context().variable(Domain::Integer)?;
    let Argument::Const(copied_length) = inputs[2] else {
        return Err("missing const input".into());
    };
    let array = query
        .context()
        .construct(Constructor::ArrayInfer(copied_length), &[element])?;
    let wrapper = query.context().nominal(
        ItemId(7),
        &[Argument::Type(element), Argument::Const(copied_length)],
    )?;
    query.context().equate(&store, ty(inputs[0])?, array)?;
    query.context().equate(&store, ty(inputs[1])?, wrapper)?;
    assert!(
        query
            .commit(&mut store, &mut source, &mut closed())?
            .changed
    );
    let inferred_element = source.child(&store, a, 0)?;
    let byte = source.known(&store, BYTE)?;
    source.equate(&store, inferred_element, byte)?;
    let constant = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    let three = source.known_const(&store, constant)?;
    source.equate_consts(length, three)?;
    let array = source.resolved(&mut store, a)?;
    assert_eq!(
        store.kind(array),
        Some(Kind::Array {
            element: BYTE,
            length: 3
        })
    );
    let arguments = source.item_arguments(&store, b)?;
    assert_eq!(source.resolved(&mut store, ty(arguments[0])?)?, BYTE);
    let Argument::Const(retained) = arguments[1] else {
        return Err("lost const argument".into());
    };
    assert_eq!(source.resolved_const(&mut store, retained)?, constant);
    Ok(())
}

#[test]
fn source_refinement_during_a_wait_is_reconciled_with_the_query() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut query, inputs) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    let later = source.variable(Domain::Any)?;
    let shape = source.construct(Constructor::Tuple, &[later, later])?;
    source.equate(&store, input, shape)?;
    query.refresh(&store, &mut source)?;
    let child = query.context().child(&store, ty(inputs[0])?, 1)?;
    let byte = query.context().known(&store, BYTE)?;
    query.context().equate(&store, child, byte)?;
    assert!(
        query
            .commit(&mut store, &mut source, &mut closed())?
            .changed
    );
    assert_eq!(source.resolved(&mut store, later)?, BYTE);
    let result = source.resolved(&mut store, input)?;
    let Some(Kind::Tuple(fields)) = store.kind(result) else {
        return Err("missing tuple".into());
    };
    assert_eq!(store.list(fields), Some(&[BYTE, BYTE][..]));
    Ok(())
}

#[test]
fn incompatible_later_constraints_reject_before_source_allocation() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut query, inputs) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    let byte = query.context().known(&store, BYTE)?;
    query.context().equate(&store, ty(inputs[0])?, byte)?;
    let boolean = source.known(&store, TypeId::primitive(Primitive::Bool))?;
    source.equate(&store, input, boolean)?;
    let sentinel = source.variable(Domain::Any)?;
    assert!(matches!(
        query.commit(&mut store, &mut source, &mut closed()),
        Err(Error::Mismatch)
    ));
    assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
    assert_eq!(
        source.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::Bool)
    );
    Ok(())
}

#[test]
fn lifetime_closure_validates_known_and_partial_graphs_before_committing() -> TestResult {
    for known in [false, true] {
        let mut store = Store::default();
        let mut source = Context::default();
        let input = source.variable(Domain::Any)?;
        let (mut query, inputs) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
        let reference = if known {
            let ty = store.intern(Kind::Reference {
                mutable: false,
                region: RegionId(41),
                referent: BYTE,
            })?;
            query.context().known(&store, ty)?
        } else {
            let child = query.context().variable(Domain::Any)?;
            query.context().construct(
                Constructor::Reference {
                    mutable: false,
                    region: RegionId(41),
                },
                &[child],
            )?
        };
        query.context().equate(&store, ty(inputs[0])?, reference)?;
        let sentinel = source.variable(Domain::Any)?;
        assert!(matches!(
            query.commit(&mut store, &mut source, &mut closed()),
            Err(Error::InvalidIdentity)
        ));
        assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
        assert_eq!(source.unresolved_domain(input)?, Some(Domain::Any));
    }
    Ok(())
}

#[test]
fn closed_captures_preserve_nested_callable_locals_and_query_equations() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut query, inputs) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    let mut parameters = Vec::new();
    let byte = query.context().known(&store, BYTE)?;
    for region in [RegionId::bound(0, 0)?, RegionId(41)] {
        parameters.push(query.context().construct(
            Constructor::Reference {
                mutable: false,
                region,
            },
            &[byte],
        )?);
    }
    parameters.push(query.context().known(&store, TypeId::UNIT)?);
    let function = query.context().construct(
        Constructor::Function {
            bound_regions: 1,
            parameters: 2,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        },
        &parameters,
    )?;
    query.context().equate(&store, ty(inputs[0])?, function)?;
    query
        .context()
        .region_equalities
        .push((RegionId(41), RegionId(42)));
    let mut closing =
        Bound::restore_regions(HashMap::from_iter([(RegionId(41), RegionId::bound(0, 0)?)]));
    let committed = query.commit(&mut store, &mut source, &mut closing)?;
    assert_eq!(committed.equalities, vec![(RegionId(41), RegionId(42))]);
    let function = source.resolved(&mut store, input)?;
    let Some(Kind::Function { parameters, .. }) = store.kind(function) else {
        return Err("missing function".into());
    };
    let parameters = store.list(parameters).ok_or("missing function inputs")?;
    for (index, depth) in [0, 1].into_iter().enumerate() {
        assert!(
            matches!(store.kind(parameters[index]), Some(Kind::Reference { region, .. }) if region == RegionId::bound(depth, 0)?)
        );
    }
    Ok(())
}

#[test]
fn later_caller_lifetime_equations_are_retained_in_the_callers_scope() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut query, inputs) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    for (context, root, region) in [
        (&mut source, input, RegionId(1)),
        (query.context(), ty(inputs[0])?, RegionId(2)),
    ] {
        let reference = store.intern(Kind::Reference {
            mutable: false,
            region,
            referent: BYTE,
        })?;
        let term = context.known(&store, reference)?;
        context.equate(&store, root, term)?;
    }
    let mut closing = Bound::restore_regions(HashMap::from_iter([(RegionId(2), RegionId(2))]));
    let committed = query.commit(&mut store, &mut source, &mut closing)?;
    assert!(!committed.changed);
    assert!(committed.equalities.is_empty());
    assert!(
        source
            .region_equalities
            .contains(&(RegionId(1), RegionId(2)))
            || source
                .region_equalities
                .contains(&(RegionId(2), RegionId(1)))
    );
    Ok(())
}
