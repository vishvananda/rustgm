//! Nominal well-formedness retains projected results without deriving input bounds.
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    aggregate::{self, Definitions, Repr, Variant},
    arguments::{Argument, ConstKind},
    callable::Templates,
    copy::Properties,
    outlives::{Bounds, Projection, Queries},
    parameters::{
        Parameter, ParameterKind,
        predicates::{Bound, Predicate},
    },
    projection::{Application, Schema},
    relations::{Constraints, ProofScope},
    traits,
    variance::Variances,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Default)]
struct Fixture {
    types: Store,
    definitions: Definitions,
}
impl Fixture {
    fn parameter(&mut self, owner: u32, index: u32) -> Result<TypeId> {
        let id = self.types.intern_parameter(Parameter {
            owner: ItemId(owner),
            index,
            kind: ParameterKind::Type { sized: true },
        })?;
        Ok(self.types.intern(Kind::Parameter(id))?)
    }
    fn projected(
        &mut self,
        subject: TypeId,
        trait_args: &[Argument],
        family: &[Argument],
    ) -> Result<TypeId> {
        self.types.declare_projection(
            ItemId(50_001),
            Schema {
                trait_item: ItemId(50_000),
                sized: true,
                copies: Properties::default(),
            },
        )?;
        let arguments = self.types.intern_arguments(trait_args)?;
        let trait_application = self.types.intern_trait_application(traits::Application {
            item: ItemId(50_000),
            arguments,
        })?;
        let arguments = self.types.intern_arguments(family)?;
        Ok(self.types.intern(Kind::Projection(Application {
            item: ItemId(50_001),
            subject,
            trait_application,
            arguments,
        }))?)
    }
    fn nominal(&mut self, owner: u32, arguments: &[Argument]) -> Result<TypeId> {
        let arguments = self.types.intern_arguments(arguments)?;
        Ok(self.types.intern(Kind::Adt(ItemId(owner), arguments))?)
    }
    fn borrow(&mut self, region: RegionId, referent: TypeId) -> Result<TypeId> {
        Ok(self.types.intern(Kind::Reference {
            region,
            referent,
            mutable: false,
        })?)
    }
    fn define(&mut self, owner: u32, fields: &[TypeId]) -> Result {
        let fields = self.types.intern_list(fields)?;
        self.definitions.insert(
            ItemId(owner),
            aggregate::Kind::Struct,
            Repr::default(),
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )?;
        Ok(())
    }
    fn callable(&mut self, field: TypeId, count: u32) -> Result<TypeId> {
        let parameters = self.types.intern_list(&[field])?;
        Ok(self.types.intern(Kind::Function {
            bound_regions: count,
            parameters,
            result: TypeId::UNIT,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        })?)
    }
}
#[test]
fn recursive_requirements_remap_complete_projected_inputs() -> Result {
    let mut fixture = Fixture::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut expected = Vec::new();
    for owner in 0..2 {
        let subject = fixture.parameter(owner, 2)?;
        let argument = fixture.parameter(owner, 3)?;
        let length = fixture.types.intern_parameter(Parameter {
            owner: ItemId(owner),
            index: 4,
            kind: ParameterKind::Const {
                ty: TypeId::primitive(Primitive::Usize),
            },
        })?;
        let length = fixture.types.intern_const(ConstKind::Parameter(length))?;
        let own = fixture.projected(
            subject,
            &[Argument::Lifetime(b), Argument::Type(argument)],
            &[Argument::Const(length)],
        )?;
        let other = fixture.projected(
            argument,
            &[Argument::Lifetime(a), Argument::Type(subject)],
            &[Argument::Const(length)],
        )?;
        expected.push([
            (Projection { ty: own, depth: 0 }, a),
            (
                Projection {
                    ty: other,
                    depth: 0,
                },
                b,
            ),
        ]);
        let next = fixture.nominal(
            1 - owner,
            &[
                Argument::Lifetime(b),
                Argument::Lifetime(a),
                Argument::Type(argument),
                Argument::Type(subject),
                Argument::Const(length),
            ],
        )?;
        let next = fixture.types.intern(Kind::Pointer {
            mutable: false,
            pointee: next,
        })?;
        let field = fixture.borrow(a, own)?;
        fixture.define(owner, &[next, field])?;
    }
    let bounds = Bounds::infer(&mut fixture.types, &fixture.definitions, &[])?;
    for (owner, expected) in (0..).zip(expected) {
        let actual = bounds.projections(ItemId(owner));
        assert_eq!(actual.len(), 2);
        assert!(expected.iter().all(|p| actual.contains(p)), "{actual:?}");
        assert_eq!(bounds.parameters(ItemId(owner)), Some([].as_slice()));
    }
    assert!(bounds.steps < 500, "{}", bounds.steps);
    assert_eq!(fixture.definitions.instantiation_steps(), 0);
    Ok(())
}
#[test]
fn preparation_is_requested_cached_and_never_erases_hidden_debts() -> Result {
    let mut fixture = Fixture::default();
    let a = RegionId::bound(0, 0)?;
    let subject = fixture.parameter(0, 1)?;
    let output = fixture.projected(subject, &[], &[])?;
    let field = fixture.borrow(a, output)?;
    fixture.define(0, &[field])?;
    let root = fixture.nominal(
        0,
        &[
            Argument::Lifetime(RegionId::STATIC),
            Argument::Type(TypeId::primitive(Primitive::U8)),
        ],
    )?;
    let expected = fixture.projected(TypeId::primitive(Primitive::U8), &[], &[])?;
    let mut bounds = Bounds::infer(&mut fixture.types, &fixture.definitions, &[])?;
    let templates = Templates::default();
    assert!(
        Queries::new(&fixture.types, &bounds, &templates)
            .symbolic_requirements(root)
            .is_err()
    );
    assert_eq!(bounds.preparation_steps(), 0);
    bounds.prepare(&mut fixture.types, root)?;
    let steps = bounds.preparation_steps();
    bounds.prepare(&mut fixture.types, root)?;
    assert_eq!(bounds.preparation_steps(), steps);
    let mut queries = Queries::new(&fixture.types, &bounds, &templates);
    let requirements = queries.symbolic_requirements(root)?;
    assert_eq!(
        requirements.projections,
        &[(
            Projection {
                ty: expected,
                depth: 0
            },
            RegionId::STATIC
        )]
    );
    assert!(requirements.scalars.parameters.is_empty());
    assert!(requirements.scalars.regions.is_empty());
    assert!(queries.requirements(root).is_err());
    assert_eq!(fixture.definitions.instantiation_steps(), 0);
    Ok(())
}
#[test]
fn implied_output_bounds_do_not_imply_static_inputs() -> Result {
    let mut fixture = Fixture::default();
    let a = RegionId::bound(0, 0)?;
    let subject = fixture.parameter(0, 1)?;
    let output = fixture.projected(subject, &[], &[])?;
    fixture.define(0, &[])?;
    let mut bounds = Bounds::infer_with_predicates(
        &mut fixture.types,
        &fixture.definitions,
        &[],
        [(
            ItemId(0),
            Predicate {
                ty: output,
                bound: Bound::Outlives(a),
            },
        )],
    )?;
    let root = fixture.nominal(
        0,
        &[
            Argument::Lifetime(RegionId::STATIC),
            Argument::Type(subject),
        ],
    )?;
    bounds.prepare(&mut fixture.types, root)?;
    let Kind::Parameter(id) = fixture.types.kind(subject).ok_or("missing input")? else {
        return Err("expected parameter".into());
    };
    let variances = Variances::default();
    let templates = Templates::default();
    for require_input in [false, true] {
        let mut queries = Queries::new(&fixture.types, &bounds, &templates);
        let mut proof = Constraints::new(&fixture.types, &variances, 0, 10_000)?;
        proof.parameter(id)?;
        proof.assume_well_formed(&mut queries, root)?;
        proof.require_type(&mut queries, output, RegionId::STATIC, ProofScope::ROOT, 7)?;
        if require_input {
            proof.parameter_outlives(id, RegionId::STATIC, 9)?;
        }
        assert_eq!(proof.solve().is_ok(), !require_input);
    }
    Ok(())
}
#[test]
fn empty_callable_captures_lift_but_callable_local_inputs_do_not_escape() -> Result {
    let mut fixture = Fixture::default();
    let outer = RegionId::bound(0, 0)?;
    let captured = RegionId::bound(1, 0)?;
    let local = RegionId::bound(0, 0)?;
    let subject = fixture.parameter(0, 1)?;
    let expected = fixture.projected(subject, &[Argument::Lifetime(outer)], &[])?;
    for (owner, region, count) in [(0, captured, 0), (1, local, 1)] {
        let output = fixture.projected(subject, &[Argument::Lifetime(region)], &[])?;
        let field = fixture.borrow(captured, output)?;
        let field = fixture.callable(field, count)?;
        fixture.define(owner, &[field])?;
    }
    let bounds = Bounds::infer(&mut fixture.types, &fixture.definitions, &[])?;
    assert_eq!(
        bounds.projections(ItemId(0)),
        &[(
            Projection {
                ty: expected,
                depth: 0
            },
            outer
        )]
    );
    assert!(bounds.projections(ItemId(1)).is_empty());
    assert_eq!(bounds.parameters(ItemId(0)), Some([].as_slice()));
    assert_eq!(bounds.parameters(ItemId(1)), Some([].as_slice()));
    Ok(())
}
#[test]
fn preparation_preserves_the_occurrence_binder_and_shares_large_graphs() -> Result {
    let mut fixture = Fixture::default();
    let a = RegionId::bound(0, 0)?;
    let subject = fixture.parameter(0, 1)?;
    let output = fixture.projected(subject, &[], &[])?;
    let field = fixture.borrow(a, output)?;
    fixture.define(0, &[field])?;
    let captured = RegionId::bound(1, 0)?;
    let root = fixture.nominal(0, &[Argument::Lifetime(captured), Argument::Type(subject)])?;
    let mut shared = root;
    for _ in 0..2048 {
        let children = fixture.types.intern_list(&[shared, shared])?;
        shared = fixture.types.intern(Kind::Tuple(children))?;
    }
    let signature = fixture.callable(shared, 0)?;
    let mut bounds = Bounds::infer(&mut fixture.types, &fixture.definitions, &[])?;
    bounds.prepare(&mut fixture.types, signature)?;
    assert!(
        bounds.preparation_steps() < 10_000,
        "{}",
        bounds.preparation_steps()
    );
    let steps = bounds.preparation_steps();
    bounds.prepare(&mut fixture.types, shared)?;
    assert_eq!(bounds.preparation_steps(), steps);
    let templates = Templates::default();
    let mut queries = Queries::new(&fixture.types, &bounds, &templates);
    assert_eq!(
        queries.symbolic_requirements(signature)?.projections,
        &[(
            Projection {
                ty: output,
                depth: 1
            },
            a
        )]
    );
    Ok(())
}
#[test]
fn long_projected_dependency_chain_is_bounded_without_field_expansion() -> Result {
    let mut fixture = Fixture::default();
    let a = RegionId::bound(0, 0)?;
    let mut expected = Vec::new();
    for owner in 0..2048 {
        let subject = fixture.parameter(owner, 1)?;
        let output = fixture.projected(subject, &[], &[])?;
        expected.push(output);
        let field = if owner == 2047 {
            fixture.borrow(a, output)?
        } else {
            fixture.nominal(owner + 1, &[Argument::Lifetime(a), Argument::Type(subject)])?
        };
        fixture.define(owner, &[field])?;
    }
    let bounds = Bounds::infer(&mut fixture.types, &fixture.definitions, &[])?;
    for (owner, ty) in (0..).zip(expected) {
        assert_eq!(
            bounds.projections(ItemId(owner)),
            &[(Projection { ty, depth: 0 }, a)]
        );
    }
    assert!(bounds.steps < 100_000, "{}", bounds.steps);
    assert_eq!(fixture.definitions.instantiation_steps(), 0);
    Ok(())
}
