use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument as Canonical, ConstKind, Constant},
    callable::Templates,
    infer::{Constructor, Context, Domain, arguments::Argument},
    parameters::{Parameter, ParameterKind},
};

#[test]
fn unresolved_region_substitution_preserves_nested_binders_and_argument_edges()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut context = Context::default();
    let variable = context.variable(Domain::Any)?;
    let local = RegionId::bound(0, 0)?;
    let outer = RegionId::bound(1, 0)?;
    let nominal_args = types.intern_regions(&[outer])?;
    let nominal = context.construct(Constructor::Adt(ItemId(1), nominal_args), &[])?;
    let constant_id = types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    let constant = context.known_const(&types, constant_id)?;
    let mixed = context.nominal(
        ItemId(2),
        &[
            Argument::Lifetime(outer),
            Argument::Type(variable),
            Argument::Const(constant),
        ],
    )?;
    let borrowed = context.construct(
        Constructor::Reference {
            mutable: false,
            region: local,
        },
        &[variable],
    )?;
    let root = context.construct(
        Constructor::Function {
            parameters: 3,
            bound_regions: 1,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        },
        &[nominal, mixed, borrowed, variable],
    )?;
    let replaced = context.substitute_regions(&mut types, root, &[RegionId(9)])?;
    let byte = context.known(&types, TypeId::primitive(Primitive::U8))?;
    context.equate(&types, variable, byte)?;
    let before = context.resolved(&mut types, root)?;
    let after = context.resolved(&mut types, replaced)?;
    // Canonical and unresolved traversal must make the same binder substitution.
    let known = context.known(&types, before)?;
    let again = context.substitute_regions(&mut types, known, &[RegionId(9)])?;
    assert_eq!(context.resolved(&mut types, again)?, after);
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 1,
        ..
    }) = types.kind(after)
    else {
        return Err("missing function".into());
    };
    assert_eq!(result, TypeId::primitive(Primitive::U8));
    let fields = types.list(parameters).ok_or("missing fields")?;
    for (index, expected) in [
        (0, vec![Canonical::Lifetime(RegionId(9))]),
        (
            1,
            vec![
                Canonical::Lifetime(RegionId(9)),
                Canonical::Type(result),
                Canonical::Const(constant_id),
            ],
        ),
    ] {
        let Some(Kind::Adt(_, arguments)) = types.kind(fields[index]) else {
            return Err("missing nominal type".into());
        };
        assert_eq!(
            types
                .arguments(arguments)
                .ok_or("missing arguments")?
                .collect::<Vec<_>>(),
            expected
        );
    }
    assert!(
        matches!(types.kind(fields[2]), Some(Kind::Reference { region, referent, .. })
        if region == local && referent == result)
    );
    Ok(())
}

#[test]
fn callable_catalog_rejects_foreign_duplicate_and_reordered_parameters()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let owner = ItemId(5);
    let mut ids = Vec::new();
    for (owner, index) in [(owner, 1), (owner, 4), (ItemId(6), 1)] {
        ids.push(types.intern_parameter(Parameter {
            owner,
            index,
            kind: ParameterKind::Type { sized: true },
        })?);
    }
    let parameters = types.intern_list(&[])?;
    let signature = types.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 0,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let mut templates = Templates::default();
    for args in [
        vec![ids[2]],
        vec![ids[0], ids[0]],
        vec![ids[1], ids[0]],
        vec![],
    ] {
        assert!(
            templates
                .insert_generic(&types, owner, signature, 0, &[], &args)
                .is_err()
        );
        assert!(templates.get(owner).is_none());
    }
    templates.insert_generic(&types, owner, signature, 0, &[], &ids[..2])?;
    assert_eq!(templates.generic_parameters(owner), Some(&ids[..2]));
    assert!(
        templates
            .insert_generic(&types, owner, signature, 0, &[], &ids[..2])
            .is_err()
    );
    Ok(())
}

#[test]
fn monomorphic_templates_preserve_late_binders_and_reject_abstract_signatures()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let owner = ItemId(10);
    let parameter = types.intern_parameter(Parameter {
        owner,
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let abstract_type = types.intern(Kind::Parameter(parameter))?;
    let bound = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: TypeId::primitive(Primitive::U32),
    })?;
    let inputs = types.intern_list(&[bound])?;
    let escaping = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 0)?,
        referent: TypeId::primitive(Primitive::U32),
    })?;
    let escaping_inputs = types.intern_list(&[escaping])?;
    let mut templates = Templates::default();
    for (parameters, result, bound_regions) in [
        (inputs, abstract_type, 1),
        (inputs, TypeId::UNIT, 0),
        (escaping_inputs, TypeId::UNIT, 1),
    ] {
        let signature = types.intern(Kind::Function {
            parameters,
            result,
            bound_regions,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        })?;
        assert!(
            templates
                .insert_monomorphic(&types, owner, signature)
                .is_err()
        );
        assert!(templates.get(owner).is_none());
    }
    let signature = types.intern(Kind::Function {
        parameters: inputs,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    templates.insert_monomorphic(&types, owner, signature)?;
    assert_eq!(templates.generic_parameters(owner), Some([].as_slice()));
    assert_eq!(
        templates.get(owner).ok_or("missing template")?.parameters,
        0
    );
    assert!(
        templates
            .insert_monomorphic(&types, owner, signature)
            .is_err()
    );
    Ok(())
}

#[test]
fn fresh_function_items_rebuild_signatures_and_share_unresolved_const_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let owner = ItemId(3);
    let type_id = types.intern_parameter(Parameter {
        owner,
        index: 1,
        kind: ParameterKind::Type { sized: true },
    })?;
    let const_id = types.intern_parameter(Parameter {
        owner,
        index: 2,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let element = types.intern(Kind::Parameter(type_id))?;
    let length = types.intern_const(ConstKind::Parameter(const_id))?;
    let array = types.intern(Kind::ArrayTerm { element, length })?;
    let parameters = types.intern_list(&[array])?;
    let signature = types.intern(Kind::Function {
        parameters,
        result: array,
        bound_regions: 0,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let mut templates = Templates::default();
    templates.insert_generic(&types, owner, signature, 0, &[], &[type_id, const_id])?;
    let reference = types.intern(Kind::Reference {
        referent: TypeId::primitive(Primitive::U8),
        mutable: false,
        region: RegionId(7),
    })?;
    let mut context = Context::default();
    let element = context.known(&types, reference)?;
    let length = context.const_variable(&types, TypeId::primitive(Primitive::Usize))?;
    let signature = context.instantiate(
        &mut types,
        signature,
        &[
            (type_id, Argument::Type(element)),
            (const_id, Argument::Const(length)),
        ],
    )?;
    let item = context.function_item(
        owner,
        signature,
        &[Argument::Type(element), Argument::Const(length)],
    )?;
    let mut fresh = Vec::new();
    for first in [10, 20] {
        let rebuilt = context.fresh_regions(&mut types, &templates, item, first)?;
        assert_eq!(rebuilt.regions, 1);
        assert!(!rebuilt.needs_normalization);
        fresh.push(rebuilt.term);
    }
    let value = types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 4,
    })?;
    let known = context.known_const(&types, value)?;
    context.equate_consts(length, known)?;
    for (item, region) in fresh.into_iter().zip([RegionId(10), RegionId(20)]) {
        let resolved = context.resolved(&mut types, item)?;
        let Some(Kind::FunctionItem {
            signature,
            arguments,
            ..
        }) = types.kind(resolved)
        else {
            return Err("missing function item".into());
        };
        let arguments = types
            .arguments(arguments)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>();
        let [Canonical::Type(ty), Canonical::Const(count)] = arguments[..] else {
            return Err("wrong argument kinds".into());
        };
        assert_eq!(count, value);
        assert!(matches!(types.kind(ty), Some(Kind::Reference { region: r, .. }) if r == region));
        let Some(Kind::Function {
            parameters, result, ..
        }) = types.kind(signature)
        else {
            return Err("missing signature".into());
        };
        assert_eq!(types.list(parameters), Some([result].as_slice()));
        assert_eq!(
            types.kind(result),
            Some(Kind::Array {
                element: ty,
                length: 4
            })
        );
    }
    Ok(())
}

#[test]
fn template_instantiation_visits_a_shared_deep_graph_once() -> Result<(), Box<dyn std::error::Error>>
{
    let mut types = Store::default();
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(2),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let mut template = types.intern(Kind::Parameter(parameter))?;
    let mut expected = TypeId::primitive(Primitive::U8);
    for _ in 0..4096 {
        let list = types.intern_list(&[template, template])?;
        template = types.intern(Kind::Tuple(list))?;
        let list = types.intern_list(&[expected, expected])?;
        expected = types.intern(Kind::Tuple(list))?;
    }
    let mut context = Context::default();
    let argument = context.variable(Domain::Any)?;
    let inferred = context.instantiate(
        &mut types,
        template,
        &[(parameter, Argument::Type(argument))],
    )?;
    let known = context.known(&types, TypeId::primitive(Primitive::U8))?;
    context.equate(&types, argument, known)?;
    assert_eq!(context.resolved(&mut types, inferred)?, expected);
    assert!(
        context
            .instantiate(
                &mut types,
                template,
                &[
                    (parameter, Argument::Type(argument)),
                    (parameter, Argument::Type(argument))
                ]
            )
            .is_err()
    );
    Ok(())
}
