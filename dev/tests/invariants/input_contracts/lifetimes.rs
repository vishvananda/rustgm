//! Input lifetime preparation preserves whole conditions and complete applications.
use super::{Families, Inputs};
use crate::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    aggregate::{self, Definitions, Repr, Variant},
    arguments::Argument,
    callable::Templates,
    outlives::{Bounds, Queries},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
    projection::{Application, Schema},
    traits,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn parameter(store: &mut Store, owner: u32, index: u32) -> TestResult<(ParameterId, TypeId)> {
    let id = store.intern_parameter(Parameter {
        owner: ItemId(owner),
        index,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, store.intern(Kind::Parameter(id))?))
}

fn contracts(store: &mut Store) -> TestResult<Families> {
    let (subject, self_ty) = parameter(store, 9, 0)?;
    let (_, parent) = parameter(store, 1, 2)?;
    let (_, own) = parameter(store, 2, 1)?;
    store.declare_projection(
        ItemId(2),
        Schema {
            trait_item: ItemId(1),
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut clauses = Vec::new();
    for (ty, shorter) in [(self_ty, a), (parent, b), (own, RegionId::STATIC)] {
        clauses.push(Clause::Predicate(Predicate {
            ty,
            bound: Bound::Outlives(shorter),
        }));
    }
    // This clause remains an obligation of semantic formation, not an implied fact.
    clauses.push(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: Predicate {
            ty: own,
            bound: Bound::Outlives(RegionId::bound(0, 0)?),
        },
    })?));
    let mut result = Families::default();
    result.insert(
        store,
        ItemId(2),
        Inputs {
            owner: ItemId(1),
            subject,
            captures: 1,
            parent: vec![Argument::Type(parent)].into_boxed_slice(),
            own: vec![Argument::Lifetime(b), Argument::Type(own)].into_boxed_slice(),
        },
        &clauses,
        &[(a, b)],
    )?;
    Ok(result)
}

fn application(
    store: &mut Store,
    types: [TypeId; 3],
    regions: [RegionId; 2],
) -> TestResult<TypeId> {
    let parent =
        store.intern_arguments(&[Argument::Lifetime(regions[0]), Argument::Type(types[1])])?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments: parent,
    })?;
    let arguments =
        store.intern_arguments(&[Argument::Lifetime(regions[1]), Argument::Type(types[2])])?;
    Ok(store.intern(Kind::Projection(Application {
        item: ItemId(2),
        subject: types[0],
        trait_application,
        arguments,
    }))?)
}

fn actuals(store: &mut Store, owner: u32) -> TestResult<[(ParameterId, TypeId); 3]> {
    Ok([
        parameter(store, owner, 2)?,
        parameter(store, owner, 3)?,
        parameter(store, owner, 4)?,
    ])
}

#[test]
fn complete_family_inputs_are_prepared_before_queries_and_remain_instance_specific() -> TestResult {
    let mut store = Store::default();
    let families = contracts(&mut store)?;
    let actual = actuals(&mut store, 3)?;
    let a = RegionId::bound(0, 5)?;
    let b = RegionId::bound(0, 6)?;
    let ty = application(&mut store, actual.map(|p| p.1), [a, b])?;
    let mut bounds =
        Bounds::infer_with_inputs(&mut store, &Definitions::default(), &[], [], families)?;
    let callables = Templates::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    assert!(queries.symbolic_requirements(ty).is_err());
    let cache = queries.into_cache();
    bounds.prepare(&mut store, ty)?;
    let mut queries = cache.attach(&store, &bounds, &callables);
    let required = queries.symbolic_requirements(ty)?;
    assert_eq!(required.scalars.regions, [(a, b)]);
    assert_eq!(
        required.scalars.parameters,
        [
            (actual[0].0, a),
            (actual[1].0, b),
            (actual[2].0, RegionId::STATIC)
        ]
    );
    assert!(required.projections.is_empty());
    let work = bounds.preparation_steps();
    bounds.prepare(&mut store, ty)?;
    assert_eq!(bounds.preparation_steps(), work);
    let other = actuals(&mut store, 4)?;
    let c = RegionId::bound(0, 7)?;
    let different = application(&mut store, other.map(|p| p.1), [b, c])?;
    bounds.prepare(&mut store, different)?;
    let mut queries = Queries::new(&store, &bounds, &callables);
    let required = queries.symbolic_requirements(different)?;
    assert_eq!(required.scalars.regions, [(b, c)]);
    assert_eq!(
        required.scalars.parameters,
        [
            (other[0].0, b),
            (other[1].0, c),
            (other[2].0, RegionId::STATIC)
        ]
    );
    Ok(())
}

#[test]
fn family_conditions_with_callable_locals_are_filtered_before_component_decomposition() -> TestResult
{
    let mut store = Store::default();
    let families = contracts(&mut store)?;
    let actual = actuals(&mut store, 3)?;
    let local = store.intern(Kind::Reference {
        referent: TypeId::UNIT,
        mutable: false,
        region: RegionId::bound(0, 0)?,
    })?;
    let list = store.intern_list(&[actual[2].1, local])?;
    let composite = store.intern(Kind::Tuple(list))?;
    let a = RegionId::bound(1, 5)?;
    let b = RegionId::bound(1, 6)?;
    let projected = application(&mut store, [actual[0].1, actual[1].1, composite], [a, b])?;
    let parameters = store.intern_list(&[projected])?;
    let ty = store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let mut bounds =
        Bounds::infer_with_inputs(&mut store, &Definitions::default(), &[], [], families)?;
    bounds.prepare(&mut store, ty)?;
    let callables = Templates::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    let required = queries.symbolic_requirements(ty)?;
    let a = RegionId::bound(0, 5)?;
    let b = RegionId::bound(0, 6)?;
    assert_eq!(required.scalars.regions, [(a, b)]);
    assert_eq!(
        required.scalars.parameters,
        [(actual[0].0, a), (actual[1].0, b)]
    );
    Ok(())
}

#[test]
fn nominal_fields_require_independent_bounds_before_their_family_conditions_hold() -> TestResult {
    let mut store = Store::default();
    let families = contracts(&mut store)?;
    let actual = actuals(&mut store, 3)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let field = application(&mut store, actual.map(|p| p.1), [a, b])?;
    let fields = store.intern_list(&[field])?;
    let mut definitions = Definitions::default();
    definitions.insert(
        ItemId(3),
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let missing = Bounds::infer_with_inputs(&mut store, &definitions, &[], [], families)?;
    assert_eq!(missing.get(ItemId(3)), Some(&[][..]));
    assert_eq!(missing.parameters(ItemId(3)), Some(&[][..]));
    let families = contracts(&mut store)?;
    let clauses = [
        (actual[0].1, a),
        (actual[1].1, b),
        (actual[2].1, RegionId::STATIC),
    ]
    .map(|(ty, shorter)| {
        (
            ItemId(3),
            Clause::Predicate(Predicate {
                ty,
                bound: Bound::Outlives(shorter),
            }),
        )
    });
    let mut bounds = Bounds::infer_with_inputs(
        &mut store,
        &definitions,
        &[(ItemId(3), (a, b))],
        clauses,
        families,
    )?;
    let foreign = actuals(&mut store, 4)?;
    let x = RegionId::bound(0, 7)?;
    let y = RegionId::bound(0, 8)?;
    let args = store.intern_arguments(&[
        Argument::Lifetime(x),
        Argument::Lifetime(y),
        Argument::Type(foreign[0].1),
        Argument::Type(foreign[1].1),
        Argument::Type(foreign[2].1),
    ])?;
    let ty = store.intern(Kind::Adt(ItemId(3), args))?;
    bounds.prepare(&mut store, ty)?;
    let callables = Templates::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    let required = queries.symbolic_requirements(ty)?;
    assert_eq!(required.scalars.regions, [(x, y)]);
    let mut parameters = required.scalars.parameters.to_vec();
    parameters.sort_unstable_by_key(|p| p.0.0);
    assert_eq!(
        parameters,
        [
            (foreign[0].0, x),
            (foreign[1].0, y),
            (foreign[2].0, RegionId::STATIC)
        ]
    );
    Ok(())
}

#[test]
fn repeated_family_inputs_share_preparation_and_distinct_applications_grow_linearly() -> TestResult
{
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let families = contracts(&mut store)?;
        let actual = actuals(&mut store, 3)?;
        let mut roots = Vec::new();
        for slot in 0..width {
            roots.push(application(
                &mut store,
                actual.map(|p| p.1),
                [
                    RegionId::bound(0, 0)?,
                    RegionId::bound(0, u16::try_from(slot)?)?,
                ],
            )?);
        }
        let list = store.intern_list(&roots)?;
        let root = store.intern(Kind::Tuple(list))?;
        let mut bounds =
            Bounds::infer_with_inputs(&mut store, &Definitions::default(), &[], [], families)?;
        bounds.prepare(&mut store, root)?;
        let work = bounds.preparation_steps();
        println!("family-input width={width} steps={work}");
        assert!(work <= 64 * width + 128, "{width}: {work}");
        bounds.prepare(&mut store, root)?;
        assert_eq!(bounds.preparation_steps(), work);
    }
    Ok(())
}
