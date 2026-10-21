#[path = "helpers/qualify.rs"]
mod rustc;
use rustgm::{
    borrow::regions::constraints::Constraints,
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
        aggregate::{self, Definitions, Repr, Variant},
        arguments::{Argument, ConstKind, Constant},
        infer::{self, Context, Domain},
        parameters::{Parameter, ParameterKind},
        variance::{Variance, Variances},
    },
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[derive(Default)]
struct Fixture {
    types: Store,
    definitions: Definitions,
    schemas: Vec<(ItemId, GenericArgs)>,
}
impl Fixture {
    fn parameter(&mut self, item: ItemId, index: u32) -> Result<TypeId, &'static str> {
        let id = self.types.intern_parameter(Parameter {
            owner: item,
            index,
            kind: ParameterKind::Type { sized: true },
        })?;
        self.types.intern(Kind::Parameter(id))
    }
    fn define(&mut self, item: ItemId, arguments: &[Argument], fields: &[TypeId]) -> TestResult {
        let arguments = self.types.intern_arguments(arguments)?;
        self.schemas.push((item, arguments));
        let fields = self.types.intern_list(fields)?;
        self.definitions.insert(
            item,
            aggregate::Kind::Struct,
            Repr::default(),
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )?;
        Ok(())
    }
    fn nominal(&mut self, item: ItemId, arguments: &[Argument]) -> Result<TypeId, &'static str> {
        let args = self.types.intern_arguments(arguments)?;
        self.types.intern(Kind::Adt(item, args))
    }
    fn pointer(&mut self, pointee: TypeId, mutable: bool) -> Result<TypeId, &'static str> {
        self.types.intern(Kind::Pointer { mutable, pointee })
    }
    fn reference(&mut self, region: RegionId, referent: TypeId) -> Result<TypeId, &'static str> {
        self.types.intern(Kind::Reference {
            mutable: false,
            region,
            referent,
        })
    }
    fn function(
        &mut self,
        parameters: &[TypeId],
        result: TypeId,
        bound_regions: u32,
    ) -> Result<TypeId, &'static str> {
        let parameters = self.types.intern_list(parameters)?;
        self.types.intern(Kind::Function {
            parameters,
            result,
            bound_regions,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        })
    }
    fn infer(&self) -> Result<Variances, &'static str> {
        Variances::infer(&self.types, &self.definitions, &self.schemas)
    }
    fn coercion(&self, variances: &Variances, actual: TypeId, expected: TypeId) -> bool {
        Constraints::new(&self.types, variances, 1, 100_000)
            .and_then(|mut constraints| {
                constraints.coerce(actual, expected, false, 7)?;
                constraints.solve()
            })
            .is_ok()
    }
}

#[test]
fn nested_products_are_solved_independently_of_declaration_order() -> TestResult {
    let mut f = Fixture::default();
    let contra = ItemId(0);
    let double = ItemId(1);
    let invariant = ItemId(2);
    let outer = ItemId(3);
    for item in [double, outer, invariant, contra] {
        let t = f.parameter(item, 0)?;
        let field = match item {
            ItemId(0) => f.function(&[t], TypeId::UNIT, 0)?,
            ItemId(1) => {
                let inner = f.nominal(contra, &[Argument::Type(t)])?;
                f.nominal(contra, &[Argument::Type(inner)])?
            }
            ItemId(2) => f.pointer(t, true)?,
            _ => {
                let inner = f.nominal(invariant, &[Argument::Type(t)])?;
                f.nominal(contra, &[Argument::Type(inner)])?
            }
        };
        f.define(item, &[Argument::Type(t)], &[field])?;
    }
    let byte = TypeId::primitive(Primitive::U8);
    let short = f.reference(RegionId::bound(0, 0)?, byte)?;
    let long = f.reference(RegionId::STATIC, byte)?;
    for _ in 0..4 {
        let variances = f.infer()?;
        for (item, expected, shorten, lengthen) in [
            (contra, Variance::Contravariant, false, true),
            (double, Variance::Covariant, true, false),
            (invariant, Variance::Invariant, false, false),
            (outer, Variance::Invariant, false, false),
        ] {
            assert_eq!(variances.get(item), Some([expected].as_slice()));
            let actual = f.nominal(item, &[Argument::Type(long)])?;
            let expected = f.nominal(item, &[Argument::Type(short)])?;
            assert_eq!(f.coercion(&variances, actual, expected), shorten);
            assert_eq!(f.coercion(&variances, expected, actual), lengthen);
        }
        f.schemas.rotate_left(1);
    }
    assert_eq!(f.definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn recursive_equations_preserve_unused_parameters_and_do_not_expand_fields() -> TestResult {
    let mut f = Fixture::default();
    for index in 0..7 {
        let item = ItemId(index);
        let t = f.parameter(item, 0)?;
        let target = ItemId(match index {
            0 => 1,
            1 => 0,
            2 => 3,
            3 => 2,
            _ => index,
        });
        let arg = if index == 4 {
            let list = f.types.intern_list(&[t])?;
            f.types.intern(Kind::Tuple(list))?
        } else {
            t
        };
        let child = f.nominal(target, &[Argument::Type(arg)])?;
        let indirect = if index == 5 {
            f.function(&[child], TypeId::UNIT, 0)?
        } else {
            f.pointer(child, false)?
        };
        let fields = if matches!(index, 1 | 4 | 5) {
            vec![t, indirect]
        } else {
            vec![indirect]
        };
        f.define(item, &[Argument::Type(t)], &fields)?;
    }
    let variances = f.infer()?;
    for (index, expected) in [
        (0, Variance::Covariant),
        (1, Variance::Covariant),
        (2, Variance::Bivariant),
        (3, Variance::Bivariant),
        (4, Variance::Covariant),
        (5, Variance::Invariant),
        (6, Variance::Bivariant),
    ] {
        assert_eq!(variances.get(ItemId(index)), Some([expected].as_slice()));
    }
    assert_eq!(f.definitions.instantiation_steps(), 0);
    assert!(variances.steps < 250);
    Ok(())
}

#[test]
fn mixed_parameter_ordinals_keep_their_declared_variances() -> TestResult {
    let mut f = Fixture::default();
    let item = ItemId(0);
    let region = RegionId::bound(0, 0)?;
    let t = f.parameter(item, 1)?;
    let usize_ = TypeId::primitive(Primitive::Usize);
    // Const and type slots may interleave; lifetime indices are not type indices.
    let n = f.types.intern_parameter(Parameter {
        owner: item,
        index: 2,
        kind: ParameterKind::Const { ty: usize_ },
    })?;
    let n = f.types.intern_const(ConstKind::Parameter(n))?;
    let u = f.parameter(item, 3)?;
    let fields = [
        f.reference(region, t)?,
        f.function(&[u], TypeId::UNIT, 0)?,
        f.types.intern(Kind::ArrayTerm {
            element: TypeId::primitive(Primitive::U8),
            length: n,
        })?,
    ];
    f.define(
        item,
        &[
            Argument::Lifetime(region),
            Argument::Type(t),
            Argument::Const(n),
            Argument::Type(u),
        ],
        &fields,
    )?;
    let variances = f.infer()?;
    assert_eq!(
        variances.get(item),
        Some(
            [
                Variance::Covariant,
                Variance::Covariant,
                Variance::Invariant,
                Variance::Contravariant
            ]
            .as_slice()
        )
    );
    let byte = TypeId::primitive(Primitive::U8);
    let short = f.reference(region, byte)?;
    let long = f.reference(RegionId::STATIC, byte)?;
    let three = f.types.intern_constant(Constant {
        ty: usize_,
        bits: 3,
    })?;
    let four = f.types.intern_constant(Constant {
        ty: usize_,
        bits: 4,
    })?;
    let actual = f.nominal(
        item,
        &[
            Argument::Lifetime(RegionId::STATIC),
            Argument::Type(long),
            Argument::Const(three),
            Argument::Type(short),
        ],
    )?;
    let expected = f.nominal(
        item,
        &[
            Argument::Lifetime(region),
            Argument::Type(short),
            Argument::Const(three),
            Argument::Type(long),
        ],
    )?;
    let wrong_const = f.nominal(
        item,
        &[
            Argument::Lifetime(region),
            Argument::Type(short),
            Argument::Const(four),
            Argument::Type(long),
        ],
    )?;
    assert!(f.coercion(&variances, actual, expected));
    assert!(!f.coercion(&variances, expected, actual));
    assert!(!f.coercion(&variances, actual, wrong_const));
    Ok(())
}

#[test]
fn const_only_parameters_are_invariant_even_when_no_fields_use_them() -> TestResult {
    let mut f = Fixture::default();
    let item = ItemId(0);
    let usize_ = TypeId::primitive(Primitive::Usize);
    let id = f.types.intern_parameter(Parameter {
        owner: item,
        index: 0,
        kind: ParameterKind::Const { ty: usize_ },
    })?;
    let n = f.types.intern_const(ConstKind::Parameter(id))?;
    f.define(item, &[Argument::Const(n)], &[])?;
    let variances = f.infer()?;
    assert_eq!(variances.get(item), Some([Variance::Invariant].as_slice()));
    let three = f.types.intern_constant(Constant {
        ty: usize_,
        bits: 3,
    })?;
    let four = f.types.intern_constant(Constant {
        ty: usize_,
        bits: 4,
    })?;
    let a = f.nominal(item, &[Argument::Const(three)])?;
    let b = f.nominal(item, &[Argument::Const(four)])?;
    assert!(f.coercion(&variances, a, a));
    assert!(!f.coercion(&variances, a, b));
    let b = f.nominal(item, &[Argument::Type(TypeId::UNIT)])?;
    assert!(!f.coercion(&variances, a, b));
    Ok(())
}

#[test]
fn const_argument_relations_charge_work_even_without_lifetime_edges() -> TestResult {
    let mut f = Fixture::default();
    let item = ItemId(0);
    let usize_ = TypeId::primitive(Primitive::Usize);
    let mut parameters = Vec::new();
    for index in 0..100 {
        let id = f.types.intern_parameter(Parameter {
            owner: item,
            index,
            kind: ParameterKind::Const { ty: usize_ },
        })?;
        parameters.push(Argument::Const(
            f.types.intern_const(ConstKind::Parameter(id))?,
        ));
    }
    f.define(item, &parameters, &[])?;
    let variances = f.infer()?;
    let three = f.types.intern_constant(Constant {
        ty: usize_,
        bits: 3,
    })?;
    let four = f.types.intern_constant(Constant {
        ty: usize_,
        bits: 4,
    })?;
    let mut arguments = vec![Argument::Const(three); 100];
    let actual = f.nominal(item, &arguments)?;
    arguments[99] = Argument::Const(four);
    let expected = f.nominal(item, &arguments)?;
    let mut constraints = Constraints::new(&f.types, &variances, 0, 16)?;
    assert!(matches!(
        constraints.coerce(actual, expected, false, 0),
        Err(rustgm::borrow::regions::graph::Error::Budget)
    ));
    assert!(!f.coercion(&variances, actual, expected));
    Ok(())
}

#[test]
fn nominal_inference_retains_directed_lifetime_facts_for_the_region_proof() -> TestResult {
    for (mutable, accepted) in [(false, true), (true, false)] {
        let mut f = Fixture::default();
        let item = ItemId(0);
        let t = f.parameter(item, 0)?;
        let field = f.pointer(t, mutable)?;
        f.define(item, &[Argument::Type(t)], &[field])?;
        let variances = f.infer()?;
        let byte = TypeId::primitive(Primitive::U8);
        let long = f.reference(RegionId::STATIC, byte)?;
        let short = f.reference(RegionId::bound(0, 0)?, byte)?;
        let actual = f.nominal(item, &[Argument::Type(long)])?;
        let expected = f.nominal(item, &[Argument::Type(short)])?;
        let mut inference = Context::default();
        let unknown = inference.variable(Domain::Any)?;
        let constructor = inference.nominal(item, &[infer::arguments::Argument::Type(unknown)])?;
        let a = inference.known(&f.types, actual)?;
        let b = inference.known(&f.types, expected)?;
        inference.equate_shape(&f.types, constructor, a)?;
        inference.equate_shape(&f.types, constructor, b)?;
        let solved = inference.finish(&mut f.types)?;
        assert!(solved.region_equalities.is_empty());
        assert_eq!(solved.types[constructor.0 as usize], actual);
        assert_eq!(solved.types[a.0 as usize], actual);
        assert_eq!(solved.types[b.0 as usize], expected);
        assert_eq!(
            f.coercion(
                &variances,
                solved.types[a.0 as usize],
                solved.types[b.0 as usize]
            ),
            accepted
        );
    }
    Ok(())
}

#[test]
fn long_shared_products_are_iterative_and_bounded() -> TestResult {
    let mut f = Fixture::default();
    let contra = ItemId(0);
    let t = f.parameter(contra, 0)?;
    let field = f.function(&[t], TypeId::UNIT, 0)?;
    f.define(contra, &[Argument::Type(t)], &[field])?;
    let deep = ItemId(1);
    let t = f.parameter(deep, 0)?;
    let mut field = t;
    for _ in 0..10_000 {
        field = f.nominal(contra, &[Argument::Type(field)])?;
        let fields = f.types.intern_list(&[field, field])?;
        field = f.types.intern(Kind::Tuple(fields))?;
    }
    f.define(deep, &[Argument::Type(t)], &[field, field])?;
    let variances = f.infer()?;
    assert_eq!(variances.get(deep), Some([Variance::Covariant].as_slice()));
    assert!(variances.steps < 300_000);
    assert_eq!(f.definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn wide_templates_diagnose_exhausted_work_before_unbounded_propagation() -> TestResult {
    let mut f = Fixture::default();
    let item = ItemId(0);
    let t = f.parameter(item, 0)?;
    let fields = f.types.intern_list(&vec![t; 600_000])?;
    let field = f.types.intern(Kind::Tuple(fields))?;
    f.define(item, &[Argument::Type(t)], &[field])?;
    assert!(matches!(
        f.infer(),
        Err("nominal variance work budget exceeded")
    ));
    Ok(())
}

#[test]
fn malformed_parameter_owners_slots_and_nominal_arguments_are_rejected() -> TestResult {
    let mut f = Fixture::default();
    let item = ItemId(0);
    let t = f.parameter(item, 0)?;
    let foreign = f.parameter(ItemId(1), 0)?;
    f.define(item, &[Argument::Type(t)], &[foreign])?;
    assert!(f.infer().is_err());
    let args = f.types.intern_arguments(&[Argument::Type(foreign)])?;
    assert!(Variances::infer(&f.types, &f.definitions, &[(item, args)]).is_err());
    let slot = f.parameter(item, 1)?;
    let args = f.types.intern_arguments(&[Argument::Type(slot)])?;
    assert!(Variances::infer(&f.types, &f.definitions, &[(item, args)]).is_err());
    let args = f.types.intern_arguments(&[Argument::Type(TypeId::UNIT)])?;
    assert!(Variances::infer(&f.types, &f.definitions, &[(item, args)]).is_err());
    for wrong_kind in [false, true] {
        let mut f = Fixture::default();
        let t = f.parameter(item, 0)?;
        let argument = if wrong_kind {
            Argument::Lifetime(RegionId::bound(0, 0)?)
        } else {
            Argument::Type(t)
        };
        let field = f.nominal(ItemId(1), &[argument])?;
        f.define(item, &[Argument::Type(t)], &[field])?;
        let other = f.parameter(ItemId(1), 0)?;
        if wrong_kind {
            f.define(ItemId(1), &[Argument::Type(other)], &[other])?;
        } else {
            f.define(ItemId(1), &[], &[])?;
        }
        assert!(f.infer().is_err());
    }
    Ok(())
}

#[test]
fn independent_rustc_controls_qualify_the_nominal_variance_rules() -> TestResult {
    for (name, source, accepted) in [
        (
            "direct-covariant",
            "struct C<T>(T);fn f<'a>(x:C<&'static u8>)->C<&'a u8>{x}",
            true,
        ),
        (
            "direct-contravariant",
            "struct C<T>(fn(T));fn f<'a>(x:C<&'a u8>)->C<&'static u8>{x}",
            true,
        ),
        (
            "mutable-pointer-invariant",
            "struct I<T>(*mut T);fn f<'a>(x:I<&'static u8>)->I<&'a u8>{x}",
            false,
        ),
        (
            "double-contravariant",
            "struct C<T>(fn(T));struct D<T>(C<C<T>>);fn f<'a>(x:D<&'static u8>)->D<&'a u8>{x}",
            true,
        ),
        (
            "nested-invariant",
            "struct I<T>(*mut T);struct W<T>(I<T>);fn f<'a>(x:W<&'static u8>)->W<&'a u8>{x}",
            false,
        ),
        ("recursive-only", "struct R<T>(*const R<T>);", false),
        (
            "expanding-covariant",
            "struct R<T>{value:T,next:*const R<(T,)>}fn f<'a>(x:R<&'static u8>)->R<&'a u8>{x}",
            true,
        ),
        (
            "mutual-covariant",
            "struct A<T>(T,*const B<T>);struct B<T>(*const A<T>);fn f<'a>(x:B<&'static u8>)->B<&'a u8>{x}",
            true,
        ),
        (
            "mutual-recursive-only",
            "struct A<T>(*const B<T>);struct B<T>(*const A<T>);",
            false,
        ),
        ("unused-const", "struct M<const N:usize>;", true),
        (
            "const-invariant",
            "struct M<const N:usize>;fn f<const A:usize,const B:usize>(x:M<A>)->M<B>{x}",
            false,
        ),
        (
            "higher-ranked-input",
            "struct F<T>(for<'b>fn(&'b T));fn f<'a>(x:F<&'a u8>)->F<&'static u8>{x}",
            true,
        ),
        (
            "mixed-slots",
            "struct M<'a,T,U,const N:usize>(&'a T,fn(U),[u8;N]);fn f<'a>(x:M<'static,&'static u8,&'a u8,3>)->M<'a,&'a u8,&'static u8,3>{x}",
            true,
        ),
    ] {
        rustc::qualify(name, source, accepted)?;
    }
    Ok(())
}

#[test]
fn higher_ranked_binders_do_not_capture_nominal_slots() -> TestResult {
    let mut f = Fixture::default();
    let region = RegionId::bound(0, 0)?;
    let higher = ItemId(1);
    let t = f.parameter(higher, 1)?;
    let local = f.reference(RegionId::bound(0, 0)?, t)?;
    let captured = f.reference(RegionId::bound(1, 0)?, t)?;
    let function = f.function(&[local, captured], TypeId::UNIT, 1)?;
    f.define(
        higher,
        &[Argument::Lifetime(region), Argument::Type(t)],
        &[function],
    )?;
    let variances = f.infer()?;
    assert_eq!(
        variances.get(higher),
        Some([Variance::Contravariant, Variance::Contravariant].as_slice())
    );
    let byte = TypeId::primitive(Primitive::U8);
    let short = f.reference(region, byte)?;
    let long = f.reference(RegionId::STATIC, byte)?;
    let actual = f.nominal(higher, &[Argument::Lifetime(region), Argument::Type(short)])?;
    let expected = f.nominal(
        higher,
        &[Argument::Lifetime(RegionId::STATIC), Argument::Type(long)],
    )?;
    assert!(f.coercion(&variances, actual, expected));
    assert!(!f.coercion(&variances, expected, actual));
    Ok(())
}

impl Fixture {
    fn projection(
        &mut self,
        subject: TypeId,
        arguments: &[Argument],
        family: &[Argument],
    ) -> TestResult<TypeId> {
        use rustgm::types::{
            copy::Properties,
            projection::{Application, Schema},
            traits,
        };
        self.types.declare_projection(
            ItemId(90_001),
            Schema {
                trait_item: ItemId(90_000),
                sized: true,
                copies: Properties::default(),
            },
        )?;
        let arguments = self.types.intern_arguments(arguments)?;
        let trait_application = self.types.intern_trait_application(traits::Application {
            item: ItemId(90_000),
            arguments,
        })?;
        let arguments = self.types.intern_arguments(family)?;
        Ok(self.types.intern(Kind::Projection(Application {
            item: ItemId(90_001),
            subject,
            trait_application,
            arguments,
        }))?)
    }
}

#[test]
fn retained_projected_fields_preserve_complete_input_variance_after_normalization() -> TestResult {
    let mut fixture = Fixture::default();
    let owner = ItemId(0);
    let a = RegionId::bound(0, 0)?;
    let subject = fixture.parameter(owner, 1)?;
    let other = fixture.parameter(owner, 2)?;
    let length = fixture.types.intern_parameter(Parameter {
        owner,
        index: 3,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let length = fixture.types.intern_const(ConstKind::Parameter(length))?;
    let output = fixture.projection(
        subject,
        &[Argument::Lifetime(a), Argument::Type(other)],
        &[Argument::Const(length)],
    )?;
    let normalized = TypeId::primitive(Primitive::U16);
    fixture.define(
        owner,
        &[
            Argument::Lifetime(a),
            Argument::Type(subject),
            Argument::Type(other),
            Argument::Const(length),
        ],
        &[normalized],
    )?;
    let original = fixture.types.intern_list(&[output])?;
    let before = fixture.types.len();
    let raw = Variances::infer(&fixture.types, &fixture.definitions, &fixture.schemas)?;
    assert_eq!(
        raw.get(owner),
        Some(
            [
                Variance::Bivariant,
                Variance::Bivariant,
                Variance::Bivariant,
                Variance::Invariant
            ]
            .as_slice()
        )
    );
    let inferred = Variances::infer_with_fields(
        &fixture.types,
        &fixture.definitions,
        &fixture.schemas,
        |_, _, _| original,
    )?;
    assert_eq!(
        inferred.get(owner),
        Some([Variance::Invariant; 4].as_slice())
    );
    assert!(inferred.steps < 100);
    assert_eq!(fixture.types.len(), before);
    let definition = fixture.definitions.get(owner).ok_or("missing aggregate")?;
    assert_eq!(
        fixture
            .types
            .list(fixture.definitions.variants(definition)[0].fields),
        Some([normalized].as_slice())
    );
    Ok(())
}

#[test]
fn declared_field_selection_is_per_variant_and_never_by_normalized_type() -> TestResult {
    let mut fixture = Fixture::default();
    let owner = ItemId(0);
    let subject = fixture.parameter(owner, 0)?;
    let ordinary = fixture.parameter(owner, 1)?;
    let output = fixture.projection(subject, &[], &[])?;
    let normalized = TypeId::primitive(Primitive::U16);
    let first = fixture.types.intern_list(&[normalized, ordinary])?;
    let second = fixture.types.intern_list(&[normalized])?;
    let original = fixture.types.intern_list(&[output])?;
    fixture.definitions.insert(
        owner,
        aggregate::Kind::Enum,
        Repr::default(),
        &[
            Variant {
                fields: first,
                discriminant: 0,
            },
            Variant {
                fields: second,
                discriminant: 1,
            },
        ],
    )?;
    let arguments = fixture
        .types
        .intern_arguments(&[Argument::Type(subject), Argument::Type(ordinary)])?;
    fixture.schemas.push((owner, arguments));
    let another = fixture.parameter(ItemId(1), 0)?;
    fixture.define(
        ItemId(1),
        &[Argument::Type(another)],
        &[normalized, another],
    )?;
    let mut visited = Vec::new();
    let inferred = Variances::infer_with_fields(
        &fixture.types,
        &fixture.definitions,
        &fixture.schemas,
        |item, variant, fields| {
            visited.push((item, variant));
            if item == owner && variant == 1 {
                original
            } else {
                fields
            }
        },
    )?;
    assert_eq!(visited, [(owner, 0), (owner, 1), (ItemId(1), 0)]);
    assert_eq!(
        inferred.get(owner),
        Some([Variance::Invariant, Variance::Covariant].as_slice())
    );
    assert_eq!(
        inferred.get(ItemId(1)),
        Some([Variance::Covariant].as_slice())
    );
    Ok(())
}

#[test]
fn retained_field_reader_rejects_missing_or_changed_variant_arity() -> TestResult {
    let mut fixture = Fixture::default();
    let owner = ItemId(0);
    let subject = fixture.parameter(owner, 0)?;
    fixture.define(owner, &[Argument::Type(subject)], &[subject])?;
    let wrong = fixture.types.intern_list(&[subject, subject])?;
    for fields in [wrong, rustgm::types::ListId(u32::MAX)] {
        assert!(
            Variances::infer_with_fields(
                &fixture.types,
                &fixture.definitions,
                &fixture.schemas,
                |_, _, _| fields
            )
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn retained_field_graphs_share_work_without_expanding_storage() -> TestResult {
    let mut fixture = Fixture::default();
    let owner = ItemId(0);
    let subject = fixture.parameter(owner, 0)?;
    let mut declared = fixture.projection(subject, &[], &[])?;
    for _ in 0..4096 {
        let fields = fixture.types.intern_list(&[declared, declared])?;
        declared = fixture.types.intern(Kind::Tuple(fields))?;
    }
    let original = fixture.types.intern_list(&[declared])?;
    fixture.define(owner, &[Argument::Type(subject)], &[TypeId::UNIT])?;
    let inferred = Variances::infer_with_fields(
        &fixture.types,
        &fixture.definitions,
        &fixture.schemas,
        |_, _, _| original,
    )?;
    assert_eq!(inferred.get(owner), Some([Variance::Invariant].as_slice()));
    assert!(inferred.steps < 100_000, "{}", inferred.steps);
    assert_eq!(fixture.definitions.instantiation_steps(), 0);
    Ok(())
}
