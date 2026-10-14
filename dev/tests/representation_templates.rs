use rustgm::{
    abi::{layout::Layouts, templates::Templates},
    types::{
        ItemId, Kind, Primitive, Store, TypeId,
        aggregate::{self, Definitions, Repr, Representation, Variant},
        arguments::Argument,
        parameters::{Parameter, ParameterKind},
    },
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn a_failed_template_query_stays_failed_without_poisoning_unrelated_layouts() -> TestResult {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut layouts = Layouts::default();
    let mut templates = Templates::default();
    let byte = TypeId::primitive(Primitive::U8);
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let parameter = types.intern(Kind::Parameter(parameter))?;
    let fields = types.intern_list(&[parameter, byte])?;
    definitions.insert(
        ItemId(1),
        aggregate::Kind::Struct,
        Repr {
            representation: Representation::Transparent,
            ..Repr::default()
        },
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let arguments = types.intern_arguments(&[Argument::Type(parameter)])?;
    let invalid = types.intern(Kind::Adt(ItemId(1), arguments))?;
    let fields = types.intern_list(&[invalid])?;
    let wrapper = types.intern(Kind::Tuple(fields))?;
    for root in [wrapper, invalid, wrapper] {
        assert_eq!(
            templates
                .demand(&mut types, &mut definitions, &mut layouts, root)
                .err()
                .map(|e| e.message),
            Some("transparent aggregate has more than one nontrivial field")
        );
        assert!(layouts.get(root).is_none());
        let steps = templates.steps;
        assert!(
            templates
                .demand(&mut types, &mut definitions, &mut layouts, root)
                .is_err()
        );
        assert_eq!(templates.steps, steps);
    }
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: invalid,
    })?;
    assert!(templates.demand(&mut types, &mut definitions, &mut layouts, pointer)?);
    assert_eq!(layouts.get(pointer).map(|l| l.size), Some(8));
    Ok(())
}

#[test]
fn symbolic_declarations_validate_their_own_representation() -> TestResult {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let parameter = types.intern(Kind::Parameter(parameter))?;
    let fields = types.intern_list(&[parameter])?;
    definitions.insert(
        ItemId(0),
        aggregate::Kind::Struct,
        Repr {
            tag: Some(Primitive::U8),
            ..Repr::default()
        },
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let arguments = types.intern_arguments(&[Argument::Type(parameter)])?;
    let root = types.intern(Kind::Adt(ItemId(0), arguments))?;
    let mut layouts = Layouts::default();
    let error = Templates::default()
        .declaration(&mut types, &mut definitions, &mut layouts, root)
        .err()
        .map(|e| e.message);
    assert_eq!(error, Some("integer representation requires an enum"));
    assert!(layouts.get(root).is_none());
    Ok(())
}
