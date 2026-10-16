//! Input extraction and selected equations preserve their original binder scope.
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::Argument as CanonicalArgument,
    infer::{Context, Domain, Error, TermId, arguments::Argument, predicates::Predicate},
    parameters::predicates::{Bound as Requirement, Predicate as Canonical},
    projection::Schema,
    substitute::Bound,
    traits,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const ITEM: ItemId = ItemId(8);
const TRAIT: ItemId = ItemId(7);

fn store() -> Result<Store, &'static str> {
    let mut store = Store::default();
    store.declare_projection(
        ITEM,
        Schema {
            trait_item: TRAIT,
            sized: true,
        },
    )?;
    Ok(store)
}
fn goal(context: &mut Context, store: &Store, subject: TermId) -> Result<Predicate, Error> {
    let projection = context.projected(store, ITEM, subject, &[], &[])?;
    Ok(context.projection_application(store, projection)?.predicate)
}
fn canonical(store: &mut Store, ty: TypeId) -> Result<Canonical, &'static str> {
    let arguments = store.intern_arguments(&[])?;
    let application = store.intern_trait_application(traits::Application {
        item: TRAIT,
        arguments,
    })?;
    Ok(Canonical {
        ty,
        bound: Requirement::Trait(application),
    })
}
fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        region,
        mutable: false,
        referent: BYTE,
    })
}

#[test]
fn scoped_header_equations_merge_numeric_domains_in_either_direction() -> TestResult {
    let store = store()?;
    for numeric in [Domain::Integer, Domain::Float] {
        for reverse in [false, true] {
            let mut context = Context::default();
            let number = context.variable(numeric)?;
            let parameter = context.variable(Domain::Any)?;
            let left = goal(&mut context, &store, number)?;
            let right = goal(&mut context, &store, parameter)?;
            if reverse {
                context.equate_predicates(&store, &right, &left)?;
            } else {
                context.equate_predicates(&store, &left, &right)?;
            }
            assert_eq!(context.unresolved_domain(number)?, Some(numeric));
            assert_eq!(context.root(number)?, context.root(parameter)?);
            let other = if numeric == Domain::Integer {
                Domain::Float
            } else {
                Domain::Integer
            };
            let conflict = context.variable(other)?;
            let conflicting = goal(&mut context, &store, conflict)?;
            assert_eq!(
                context.equate_predicates(&store, &left, &conflicting),
                Err(Error::Mismatch)
            );
        }
    }
    Ok(())
}

#[test]
fn extraction_preserves_shared_holes_and_the_family_boundary() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let hole = context.variable(Domain::Any)?;
    let size = context.const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let region = RegionId::bound(0, 0)?;
    let inputs = [
        Argument::Type(hole),
        Argument::Lifetime(region),
        Argument::Const(size),
    ];
    let projection = context.projected(&store, ITEM, hole, &inputs, &inputs)?;
    let application = context.projection_application(&store, projection)?;
    assert_eq!(application.item, ITEM);
    assert_eq!(application.predicate.subject(), hole);
    assert_eq!(
        application.predicate.trait_arguments(),
        Some((TRAIT, inputs.as_slice()))
    );
    assert_eq!(application.arguments, inputs);
    assert_eq!(
        context.resolved(&mut store, projection),
        Err(Error::Unresolved)
    );
    let mut mapping = Bound::open_regions(1)?;
    let mapped = context.map_regions(&mut store, projection, &mut mapping)?;
    let application = context.projection_application(&store, mapped)?;
    assert_eq!(application.predicate.subject(), hole);
    assert_eq!(
        application.arguments,
        [
            Argument::Type(hole),
            Argument::Lifetime(RegionId(1)),
            Argument::Const(size)
        ]
    );
    let args = store.intern_arguments(&[CanonicalArgument::Lifetime(region)])?;
    let trait_ = store.intern_trait_application(traits::Application {
        item: TRAIT,
        arguments: args,
    })?;
    let mapped = mapping.predicate(
        &mut store,
        Canonical {
            ty: BYTE,
            bound: Requirement::Trait(trait_),
        },
    )?;
    let Requirement::Trait(id) = mapped.bound else {
        return Err("missing trait".into());
    };
    let args = store
        .trait_application(id)
        .ok_or("missing application")?
        .arguments;
    assert_eq!(
        store
            .arguments(args)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>(),
        [CanonicalArgument::Lifetime(RegionId(1))]
    );
    assert_eq!(mapping.next_region(), Some(2));
    let byte = context.known(&store, BYTE)?;
    assert!(matches!(
        context.projection_application(&store, byte),
        Err(Error::InvalidIdentity)
    ));
    Ok(())
}

#[test]
fn selected_inputs_can_resolve_closed_types_but_cannot_capture_ambient_slots() -> TestResult {
    for counts in [&[1][..], &[1, 0][..]] {
        let mut store = store()?;
        let mut context = Context::default();
        let hole = context.variable(Domain::Any)?;
        let input = goal(&mut context, &store, hole)?;
        let byte = canonical(&mut store, BYTE)?;
        context.equate_trait_predicate_inside(&mut store, &input, byte, counts)?;
        assert_eq!(context.resolved(&mut store, hole)?, BYTE);
        let hole = context.variable(Domain::Any)?;
        let input = goal(&mut context, &store, hole)?;
        let depth = u16::try_from(counts.len() - 1)?;
        let local = reference(&mut store, RegionId::bound(depth, 0)?)?;
        let selected = canonical(&mut store, local)?;
        assert_eq!(
            context.equate_trait_predicate_inside(&mut store, &input, selected, counts),
            Err(Error::Mismatch)
        );
        assert_eq!(context.resolved(&mut store, hole), Err(Error::Unresolved));
    }
    Ok(())
}

#[test]
fn ambient_universals_are_rigid_and_outer_equalities_rebase() -> TestResult {
    let mut store = store()?;
    let a = reference(&mut store, RegionId::bound(0, 0)?)?;
    let b = reference(&mut store, RegionId::bound(0, 1)?)?;
    let mut context = Context::default();
    let a = context.known(&store, a)?;
    let input = goal(&mut context, &store, a)?;
    let b = canonical(&mut store, b)?;
    assert_eq!(
        context.equate_trait_predicate_inside(&mut store, &input, b, &[2]),
        Err(Error::Mismatch)
    );
    let outer = reference(&mut store, RegionId::bound(1, 0)?)?;
    let free = reference(&mut store, RegionId(3))?;
    let outer = context.known(&store, outer)?;
    let input = goal(&mut context, &store, outer)?;
    let selected = canonical(&mut store, free)?;
    context.equate_trait_predicate_inside(&mut store, &input, selected, &[2])?;
    assert_eq!(
        context.region_equalities,
        [(RegionId::bound(0, 0)?, RegionId(3))]
    );
    Ok(())
}

#[test]
fn nested_callable_slots_still_alpha_rename_beneath_rigid_ancestors() -> TestResult {
    let mut store = store()?;
    let mut callables = Vec::new();
    for slot in [0, 1] {
        let local = reference(&mut store, RegionId::bound(0, slot)?)?;
        let outer = reference(&mut store, RegionId::bound(1, 0)?)?;
        let parameters = store.intern_list(&[local, outer])?;
        callables.push(store.intern(Kind::Function {
            bound_regions: 2,
            parameters,
            result: TypeId::UNIT,
            unsafe_: false,
        })?);
    }
    let mut context = Context::default();
    let left = context.known(&store, callables[0])?;
    let input = goal(&mut context, &store, left)?;
    let selected = canonical(&mut store, callables[1])?;
    context.equate_trait_predicate_inside(&mut store, &input, selected, &[1])?;
    assert!(context.region_equalities.is_empty());
    Ok(())
}
