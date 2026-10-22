//! Reference permits require the exact independently reconstructed blanket condition.
use super::*;

#[test]
fn reference_factory_keeps_two_shared_reborrows_and_no_mutable_value_read() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(
        "fn main(){let mut a=3u8;let mut b=3u8;let a=&mut a;let b=&mut b;let _=<&mut u8 as PartialEq<&mut u8>>::eq(&a,&b);}",
    )?;
    let selected = selected
        .into_iter()
        .find(|s| matches!(s.descriptor.operation, Operation::References(_)))
        .ok_or("reference selection")?;
    let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
    assert_eq!(body.projections.len(), 4);
    assert_eq!(
        body.values
            .iter()
            .filter(|v| matches!(
                v.kind,
                crate::body::ir::Rvalue::Borrow {
                    kind: crate::body::ir::BorrowKind::Immediate,
                    ..
                }
            ))
            .count(),
        2
    );
    assert!(
        !body
            .values
            .iter()
            .any(|v| matches!(v.kind, crate::body::ir::Rvalue::Read(_)))
    );
    crate::body::primitive_methods::validate(&mut facts, selected, &body, &input)?;
    let wrong = Selected {
        descriptor: Descriptor {
            operands: Operands::References {
                left_mutable: false,
                right_mutable: true,
            },
            ..selected.descriptor
        },
        ..selected
    };
    assert!(wrong.validate(&mut facts).is_err());
    let mut injected = input.clone();
    injected
        .assumptions
        .push((RegionId::STATIC, RegionId::bound(0, 0)?));
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &injected).is_err()
    );
    let primitive = Selected {
        descriptor: Descriptor {
            operation: Operation::Comparison(crate::support::comparisons::Comparison::PartialEq),
            operands: Operands::Scalars(Scalars {
                left: Primitive::U8,
                right: Primitive::U8,
                rhs: Rhs::Owned,
            }),
        },
        ..selected
    };
    assert!(primitive.validate(&mut facts).is_err());
    Ok(())
}

pub(super) fn replace_operands(
    facts: &mut Declarations,
    selected: Selected,
    operands: [TypeId; 2],
) -> Result<Selected> {
    let source = selected.validate(facts)?;
    let item = crate::types::ItemId(source.definition.0);
    let generic = facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("generic parameters")?;
    let arguments = operands.map(Argument::Type);
    let bindings = generic.iter().copied().zip(arguments).collect::<Vec<_>>();
    let template = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("callable template")?
        .signature;
    let signature = crate::types::substitute::Bound::parameters(&facts.types, &bindings)?
        .apply(&mut facts.types, template)?;
    let arguments = facts.types.intern_arguments(&arguments)?;
    let ty = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments,
    })?;
    Ok(Selected {
        instance: Instance::new(&facts.types, ty)?,
        ..selected
    })
}
#[test]
fn reference_factory_rejects_shape_correct_but_unpaid_referent_prerequisite() -> Result {
    let Factory {
        mut facts,
        selected,
        ..
    } = fixture_for("struct Missing;fn main(){let a=3u8;let _=<&u8 as PartialEq>::eq(&&a,&&a);}")?;
    let selected = selected
        .into_iter()
        .find(|s| matches!(s.descriptor.operation, Operation::References(_)))
        .ok_or("reference selection")?;
    let missing = facts
        .definitions
        .iter()
        .copied()
        .flatten()
        .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Adt(..))))
        .ok_or("missing nominal")?;
    let reference = facts.types.intern(Kind::Reference {
        referent: missing,
        mutable: false,
        region: RegionId::STATIC,
    })?;
    let forged = replace_operands(&mut facts, selected, [reference, reference])?;
    forged.validate(&mut facts)?;
    assert!(
        forged.reference_inputs(&mut facts).is_err(),
        "shape alone cannot grant a missing pointee condition"
    );
    Ok(())
}
#[test]
fn repeated_reference_calls_keep_private_factories_nonempty_and_source_scan_bounded() -> Result {
    for width in [8, 32, 128] {
        let mut source = String::from("fn less<T:PartialOrd>(a:&T,b:&T)->bool{a<b}");
        for i in 0..width {
            write!(
                source,
                "struct Unrelated{i};fn unused{i}(_:Unrelated{i}){{}}"
            )?;
        }
        source.push_str("fn main(){let a=1u8;let b=2u8;");
        for _ in 0..width {
            source.push_str("let _=less(&a,&b);");
        }
        source.push('}');
        let factory = fixture_for(&source)?;
        let references = factory
            .selected
            .iter()
            .filter(|s| matches!(s.descriptor.operation, Operation::References(_)))
            .count();
        assert_eq!(references, 1, "width {width}");
        assert!(
            factory.definitions < 12,
            "unrelated source bodies are not demanded"
        );
        eprintln!(
            "reference width={width} factories={} definitions={} instructions={} selections={}",
            factory.selected.len(),
            factory.definitions,
            factory.instructions,
            factory.facts.dispatch_dependencies.work.selections
        );
    }
    Ok(())
}

#[test]
fn reference_factory_cannot_erase_or_inject_declared_lifetime_witnesses() -> Result {
    let source = std::fs::read_to_string(format!(
        "{}/../pa26/tests/spec/reference-comparison/pointee-lifetime.rs",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(&source)?;
    let selected = selected
        .into_iter()
        .find(|s| matches!(s.descriptor.operation, Operation::References(_)))
        .ok_or("reference selection")?;
    let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
    assert!(
        !input.assumptions.is_empty(),
        "declared pointee impl has lifetime witnesses"
    );
    let mut erased = input.clone();
    erased.assumptions.clear();
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &erased).is_err()
    );
    let mut injected = input;
    injected
        .assumptions
        .push((RegionId::bound(0, 2)?, RegionId::STATIC));
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &injected).is_err()
    );
    Ok(())
}
