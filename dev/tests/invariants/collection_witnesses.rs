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
    let plain = crate::test_source::resolve("fn before(){}fn main(){let _=1u8<2u8;}", true)?;
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
