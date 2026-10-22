//! Sequence factories retain exact element prerequisites and metadata-only loops.
use super::*;
use crate::support::comparisons::Comparison;

#[test]
fn sequence_factory_rejects_unpaid_elements_shape_changes_and_injected_regions() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(
        "struct Missing;fn main(){let a=[3u8];let mut b=[3u8];let b:&mut[u8]=&mut b;let _=a==b;}",
    )?;
    let selected = selected
        .into_iter()
        .find(|selected| matches!(selected.descriptor.operation, Operation::Sequences(_)))
        .ok_or("sequence selection")?;
    let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
    crate::body::primitive_methods::validate(&mut facts, selected, &body, &input)?;
    assert_eq!(body.blocks.len(), 6);
    assert_eq!(body.locals.len(), 3);
    assert!(body.values.iter().all(|value| !matches!(
        value.kind,
        crate::body::ir::Rvalue::Read(_)
    ) || value.ty == TypeId::primitive(Primitive::Usize)));
    let mut malformed = body.clone();
    malformed.locals[2].mutable = false;
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &malformed, &input).is_err()
    );
    let mut injected = input.clone();
    injected
        .assumptions
        .push((RegionId::STATIC, RegionId::bound(0, 0)?));
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &injected).is_err()
    );
    let forged = Selected {
        descriptor: Descriptor {
            operands: Operands::Sequences {
                left: Sequence::Slice,
                right: Sequence::MutableSlice,
            },
            ..selected.descriptor
        },
        ..selected
    };
    assert!(forged.validate(&mut facts).is_err());
    let missing = facts
        .definitions
        .iter()
        .flatten()
        .copied()
        .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Adt(..))))
        .ok_or("Missing type")?;
    let array = facts.types.intern(Kind::Array {
        element: missing,
        length: 1,
    })?;
    let forged = references::replace_operands(&mut facts, selected, [array, array])?;
    let forged = Selected {
        descriptor: Descriptor {
            operands: Operands::Sequences {
                left: Sequence::Array,
                right: Sequence::Array,
            },
            ..forged.descriptor
        },
        ..forged
    };
    forged.validate(&mut facts)?;
    assert!(
        forged.sequence_inputs(&mut facts).is_err(),
        "shape alone cannot grant an element implementation"
    );
    Ok(())
}

#[test]
fn latent_equality_keeps_source_ids_and_scalar_contracts_unchanged() -> Result {
    let plain = crate::test_source::resolve("fn before(){}fn main(){let _=1u8+2u8;}", true)?;
    let report = crate::test_source::resolve("fn before(){}fn main(){let _=1u8==2u8;}", true)?;
    let source_ids = |report: &crate::resolve::expansion::Report| {
        report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .filter_map(|(index, definition)| definition.node.map(|node| (index, node)))
            .collect::<Vec<_>>()
    };
    assert_eq!(source_ids(&plain), source_ids(&report));
    assert_eq!(
        report.resolution.definitions.len(),
        plain.resolution.definitions.len() + 10
    );
    let declaration = report
        .resolution
        .builtin_traits
        .comparison(Comparison::PartialEq)
        .ok_or("latent identity")?;
    assert!(
        report
            .resolution
            .builtin_traits
            .comparison_latent(Comparison::PartialEq)
    );
    let mut facts = crate::sema::declarations::analyze(&report)?;
    assert!(
        facts
            .associated
            .comparison(declaration.definition)
            .is_none()
    );
    assert_eq!(facts.associated.language_functions().count(), 0);
    assert_eq!(facts.associated.library_defaults().count(), 0);
    assert!(
        crate::sema::declarations::prepare_comparison(
            &report,
            &mut facts,
            declaration.self_parameter
        )
        .is_err()
    );
    assert!(
        facts
            .associated
            .comparison(declaration.definition)
            .is_none()
    );
    let facts = crate::sema::bodies::analyze(&report, facts)?;
    assert!(
        facts
            .associated
            .comparison(declaration.definition)
            .is_none()
    );
    assert_eq!(facts.associated.library_defaults().count(), 0);
    Ok(())
}

#[test]
fn exact_latent_contract_publication_is_complete_and_idempotent() -> Result {
    let report = crate::test_source::resolve("fn main(){let _=[1u8]==[1u8];}", true)?;
    let declaration = report
        .resolution
        .builtin_traits
        .comparison(Comparison::PartialEq)
        .ok_or("latent identity")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    assert!(
        facts
            .associated
            .comparison(declaration.definition)
            .is_none()
    );
    crate::sema::declarations::prepare_comparison(&report, &mut facts, declaration.definition)?;
    assert_eq!(
        facts.associated.comparison(declaration.definition),
        Some(Comparison::PartialEq)
    );
    for (_, method) in declaration.methods.ok_or("members")?.iter() {
        let Some(Kind::FunctionItem { signature, .. }) =
            facts.definitions[method.0 as usize].and_then(|ty| facts.types.kind(ty))
        else {
            return Err("missing complete member".into());
        };
        let Some(Kind::Function {
            parameters,
            bound_regions: 2,
            ..
        }) = facts.types.kind(signature)
        else {
            return Err("missing independent borrow slots".into());
        };
        for (slot, &parameter) in facts
            .types
            .list(parameters)
            .ok_or("parameters")?
            .iter()
            .enumerate()
        {
            assert!(
                matches!(facts.types.kind(parameter),Some(Kind::Reference { mutable: false, region, .. }) if region == RegionId::bound(0, u16::try_from(slot)?)?)
            );
        }
    }
    let before = (
        facts.types.len(),
        facts.associated.language_functions().count(),
    );
    crate::sema::declarations::prepare_comparison(&report, &mut facts, declaration.definition)?;
    assert_eq!(
        before,
        (
            facts.types.len(),
            facts.associated.language_functions().count()
        )
    );
    Ok(())
}

#[test]
fn repeated_sequence_demand_avoids_unrelated_source_materialization() -> Result {
    for width in [8, 32, 128] {
        let mut source = String::from("fn same<T:PartialEq>(a:&[T],b:&[T])->bool{a==b}");
        for i in 0..width {
            write!(source, "struct Other{i};fn unused{i}(_:Other{i}){{}}")?;
        }
        source.push_str("fn main(){let a=[1u8,2];");
        for _ in 0..width {
            source.push_str("let _=same(&a,&a);");
        }
        source.push('}');
        let factory = fixture_for(&source)?;
        assert_eq!(
            factory
                .selected
                .iter()
                .filter(|selection| matches!(
                    selection.descriptor.operation,
                    Operation::Sequences(_)
                ))
                .count(),
            1
        );
        assert!(factory.definitions < 12);
        eprintln!(
            "sequence width={width} factories={} definitions={} instructions={} selections={}",
            factory.selected.len(),
            factory.definitions,
            factory.instructions,
            factory.facts.dispatch_dependencies.work.selections
        );
    }
    Ok(())
}
#[test]
fn sequence_ordering_reconstructs_only_selected_element_equations() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(
        "fn compare<'a,'b>(a:[&'a u8;1],b:[&'b u8;1])->bool{a<b}fn main(){let a=1;let b=2;let _=compare([&a],[&b]);}",
    )?;
    let selected = selected
        .into_iter()
        .find(|selected| {
            matches!(
                selected.descriptor.operation,
                Operation::Sequences(Comparison::PartialOrd)
            )
        })
        .ok_or("sequence ordering selection")?;
    let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
    let permit = crate::body::primitive_methods::validate(&mut facts, selected, &body, &input)?;
    assert_eq!(body.locals.len(), 4);
    assert!(
        !input.assumptions.is_empty(),
        "distinct selected operand views retain their matching equations"
    );
    for &(a, b) in &input.assumptions {
        assert!(
            input.assumptions.contains(&(b, a)),
            "same-element matching retains both directions"
        );
    }
    let templates = crate::body::templates::Templates { bodies: Vec::new() };
    let checked = originals(&mut facts, &templates)?;
    prove(&mut facts, &checked, body.clone(), &input, Some(permit))?;
    let mut unpaid = input.clone();
    unpaid.assumptions.clear();
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &unpaid).is_err()
    );
    let mut injected = input.clone();
    injected
        .assumptions
        .push((RegionId::STATIC, RegionId::bound(0, 0)?));
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &injected).is_err()
    );
    let mut malformed = body.clone();
    malformed.locals[3].ty = TypeId::UNIT;
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &malformed, &input).is_err()
    );
    let wrong = Selected {
        descriptor: Descriptor {
            operation: Operation::Sequences(Comparison::Ord),
            ..selected.descriptor
        },
        ..selected
    };
    assert!(wrong.validate(&mut facts).is_err());
    Ok(())
}

#[test]
fn sequence_ordering_demand_scales_without_unrelated_materialization() -> Result {
    for (kind, function) in [
        (
            Comparison::PartialOrd,
            "fn order<T:PartialOrd>(a:&[T],b:&[T])->bool{a<b}",
        ),
        (
            Comparison::Ord,
            "fn order<T:Ord>(a:&[T],b:&[T])->core::cmp::Ordering{a.cmp(b)}",
        ),
    ] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let mut source = function.to_string();
            for i in 0..width {
                write!(source, "struct Other{i};fn unused{i}(_:Other{i}){{}}")?;
            }
            source.push_str("fn main(){let a=[1u8,2];");
            for _ in 0..width {
                source.push_str("let _=order(&a,&a);");
            }
            source.push('}');
            let factory = fixture_for(&source)?;
            assert_eq!(factory.selected.iter().filter(|selection| selection.descriptor.operation == Operation::Sequences(kind)).count(), 1);
            if let Some(previous) = previous {
                assert_eq!(factory.definitions, previous);
            }
            previous = Some(factory.definitions);
            assert!(factory.definitions < 20);
            eprintln!(
                "ordering kind={kind:?} width={width} factories={} definitions={} instructions={} selections={}",
                factory.selected.len(),
                factory.definitions,
                factory.instructions,
                factory.facts.dispatch_dependencies.work.selections
            );
        }
    }
    Ok(())
}

#[test]
fn sequence_ordering_rejects_forged_binders_and_signature_referents() -> Result {
    let Factory {
        mut facts,
        selected,
        ..
    } = fixture_for("fn main(){let _=[1u8]<[2u8];}")?;
    let selected = selected
        .into_iter()
        .find(|selected| {
            matches!(
                selected.descriptor.operation,
                Operation::Sequences(Comparison::PartialOrd)
            )
        })
        .ok_or("sequence ordering")?;
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts.types.kind(selected.instance.ty())
    else {
        return Err("selected item".into());
    };
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_,
        convention,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("selected signature".into());
    };
    let parameters = facts.types.list(parameters).ok_or("parameters")?.to_vec();
    let array = facts.types.intern(Kind::Array {
        element: TypeId::primitive(Primitive::U16),
        length: 1,
    })?;
    let wrong_referent = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: array,
    })?;
    let Some(Kind::Reference { referent, .. }) = facts.types.kind(parameters[0]) else {
        return Err("borrow".into());
    };
    let wrong_slot = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 1)?,
        referent,
    })?;
    for (first, count) in [(wrong_referent, 2), (wrong_slot, 2), (parameters[0], 3)] {
        let list = facts.types.intern_list(&[first, parameters[1]])?;
        let signature = facts.types.intern(Kind::Function {
            parameters: list,
            result,
            unsafe_,
            convention,
            bound_regions: count,
        })?;
        let ty = facts.types.intern(Kind::FunctionItem {
            item,
            signature,
            arguments,
        })?;
        let wrong = Selected {
            instance: Instance::new(&facts.types, ty)?,
            ..selected
        };
        assert!(wrong.validate(&mut facts).is_err());
    }
    Ok(())
}
