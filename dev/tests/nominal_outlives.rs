//! Nominal contracts substitute rigid components without expanding recursive fields.
use rustgm::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    aggregate::{self, Definitions, Repr, Variant},
    arguments::Argument,
    callable::Templates,
    outlives::{Bounds, Queries},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        predicates::{Bound, Predicate},
    },
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[derive(Default)]
struct Fixture {
    types: Store,
    definitions: Definitions,
}
impl Fixture {
    fn parameter(&mut self, owner: u32, index: u32) -> Result<(ParameterId, TypeId), &'static str> {
        let id = self.types.intern_parameter(Parameter {
            owner: ItemId(owner),
            index,
            kind: ParameterKind::Type { sized: true },
        })?;
        Ok((id, self.types.intern(Kind::Parameter(id))?))
    }
    fn nominal(&mut self, owner: u32, args: &[Argument]) -> Result<TypeId, &'static str> {
        let arguments = self.types.intern_arguments(args)?;
        self.types.intern(Kind::Adt(ItemId(owner), arguments))
    }
    fn reference(&mut self, region: RegionId, referent: TypeId) -> Result<TypeId, &'static str> {
        self.types.intern(Kind::Reference {
            mutable: false,
            region,
            referent,
        })
    }
    fn define(&mut self, owner: u32, fields: &[TypeId]) -> Result<(), &'static str> {
        let fields = self.types.intern_list(fields)?;
        self.definitions.insert(
            ItemId(owner),
            aggregate::Kind::Struct,
            Repr::default(),
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )
    }
}

#[test]
fn mutually_recursive_templates_remap_both_parameter_and_lifetime_slots() -> TestResult {
    let mut fixture = Fixture::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut ids = Vec::new();
    for owner in 0..2 {
        let (t_id, t) = fixture.parameter(owner, 2)?;
        let (u_id, u) = fixture.parameter(owner, 3)?;
        ids.push((t_id, u_id));
        let next = fixture.nominal(
            1 - owner,
            &[
                Argument::Lifetime(b),
                Argument::Lifetime(a),
                Argument::Type(u),
                Argument::Type(t),
            ],
        )?;
        let next = fixture.types.intern(Kind::Pointer {
            mutable: false,
            pointee: next,
        })?;
        let field = fixture.reference(a, t)?;
        fixture.define(owner, &[next, field])?;
    }
    let before = fixture.types.len();
    let bounds = Bounds::infer(&fixture.types, &fixture.definitions, &[])?;
    for (owner, (t, u)) in (0..).zip(ids) {
        let mut actual = bounds
            .parameters(ItemId(owner))
            .ok_or("missing bound")?
            .to_vec();
        actual.sort_by_key(|pair| pair.0.0);
        assert_eq!(actual, [(t, a), (u, b)]);
        assert_eq!(bounds.get(ItemId(owner)), Some([].as_slice()));
    }
    assert!(bounds.steps < 150, "{}", bounds.steps);
    assert_eq!(fixture.types.len(), before);
    assert_eq!(fixture.definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn concrete_argument_requirements_keep_nested_referents_and_zero_array_types() -> TestResult {
    let mut fixture = Fixture::default();
    let (id, t) = fixture.parameter(0, 1)?;
    let a = RegionId::bound(0, 0)?;
    let field = fixture.reference(a, t)?;
    fixture.define(0, &[field])?;
    let b = RegionId(1);
    let c = RegionId(2);
    let inner = fixture.reference(c, TypeId::UNIT)?;
    let outer = fixture.reference(b, inner)?;
    let empty = fixture.types.intern(Kind::Array {
        element: outer,
        length: 0,
    })?;
    let actual = fixture.nominal(
        0,
        &[Argument::Lifetime(RegionId::STATIC), Argument::Type(empty)],
    )?;
    let bounds = Bounds::infer(&fixture.types, &fixture.definitions, &[])?;
    assert_eq!(bounds.parameters(ItemId(0)), Some([(id, a)].as_slice()));
    let templates = Templates::default();
    let mut queries = Queries::new(&fixture.types, &bounds, &templates);
    let required = queries.symbolic_requirements(actual)?;
    assert!(required.scalars.parameters.is_empty());
    assert_eq!(required.scalars.regions.len(), 3);
    for pair in [(c, b), (b, RegionId::STATIC), (c, RegionId::STATIC)] {
        assert!(required.scalars.regions.contains(&pair), "{pair:?}");
    }
    Ok(())
}

#[test]
fn declared_type_bounds_propagate_even_without_a_reference_field() -> TestResult {
    let mut fixture = Fixture::default();
    let (_, t) = fixture.parameter(0, 0)?;
    let pointer = fixture.types.intern(Kind::Pointer {
        mutable: false,
        pointee: t,
    })?;
    fixture.define(0, &[pointer])?;
    let a = RegionId::bound(0, 0)?;
    let borrowed = fixture.reference(a, TypeId::UNIT)?;
    let actual = fixture.nominal(0, &[Argument::Type(borrowed)])?;
    fixture.define(1, &[actual])?;
    let declared = [(
        ItemId(0),
        Predicate {
            ty: t,
            bound: Bound::Outlives(RegionId::STATIC),
        },
    )];
    let bounds =
        Bounds::infer_with_predicates(&fixture.types, &fixture.definitions, &[], declared)?;
    assert_eq!(
        bounds.get(ItemId(1)),
        Some([(a, RegionId::STATIC)].as_slice())
    );
    let bound = fixture.reference(RegionId::bound(0, 0)?, TypeId::UNIT)?;
    let actual = fixture.nominal(0, &[Argument::Type(bound)])?;
    let parameters = fixture.types.intern_list(&[actual])?;
    let signature = fixture.types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let templates = Templates::default();
    let mut queries = Queries::new(&fixture.types, &bounds, &templates);
    assert!(
        queries
            .symbolic_requirements(signature)?
            .scalars
            .regions
            .is_empty()
    );
    Ok(())
}

#[test]
fn forward_dependencies_propagate_once_without_instantiating_fields() -> TestResult {
    let mut fixture = Fixture::default();
    let a = RegionId::bound(0, 0)?;
    let mut ids = Vec::new();
    for owner in 0..4096 {
        let (id, t) = fixture.parameter(owner, 1)?;
        ids.push(id);
        let field = if owner == 4095 {
            fixture.reference(a, t)?
        } else {
            fixture.nominal(owner + 1, &[Argument::Lifetime(a), Argument::Type(t)])?
        };
        fixture.define(owner, &[field])?;
    }
    let before = fixture.types.len();
    let bounds = Bounds::infer(&fixture.types, &fixture.definitions, &[])?;
    for (item, id) in (0..).zip(ids) {
        assert_eq!(bounds.parameters(ItemId(item)), Some([(id, a)].as_slice()));
    }
    assert!(bounds.steps < 100_000, "{}", bounds.steps);
    assert_eq!(fixture.types.len(), before);
    assert_eq!(fixture.definitions.instantiation_steps(), 0);
    Ok(())
}
