//! Context-owned application handles remain scoped through their recipe parents.
use super::*;
use crate::types::{Primitive, RegionId};

#[test]
fn named_items_and_reference_parents_keep_context_while_scalars_share() -> Result {
    let (report, mut facts) = prepared("pub fn named()->u8{7}pub fn first(){}pub fn second(){}")?;
    let definition = |spelling: &str| {
        facts
            .functions
            .iter()
            .find(|function| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == spelling
            })
            .map(|function| function.definition)
            .ok_or("actual source function")
    };
    let named = definition("named")?;
    let first = definition("first")?;
    let second = definition("second")?;
    let item = facts.definitions[named.0 as usize].ok_or("actual named function item")?;
    let Kind::FunctionItem {
        item: selected,
        arguments,
        ..
    } = facts.types.kind(item).ok_or("actual item kind")?
    else {
        return Err("source function lost its real item identity".into());
    };
    assert_eq!(selected.0, named.0);
    let fields = facts
        .types
        .intern_list(&[item, TypeId::primitive(Primitive::U8)])?;
    let tuple = facts.types.intern(Kind::Tuple(fields))?;
    let array = facts.types.intern(Kind::Array {
        element: item,
        length: 2,
    })?;
    let borrowed = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: item,
    })?;
    let mut staged = stage(&report, &mut facts)?;
    let contexts = [
        staged.original_context(first)?,
        staged.original_context(second)?,
    ];
    let before_apps = staged.graph.applications.len();
    let before_types = staged.rows.types.len();
    let mut parents = Vec::new();
    for context in contexts {
        for ty in [item, tuple, array, borrowed] {
            assert_eq!(facts.types.contains_callables(ty), Some(true));
            assert_eq!(staged.type_key(&facts, ty, context)?.context, Some(context));
        }
        let ty = staged.ty(&facts, item, context)?;
        let s::Type::FunctionItem(application) = staged.rows.types[ty.0 as usize] else {
            return Err("actual item recipe".into());
        };
        assert_eq!(
            staged.function_application(&facts, selected, arguments, context)?,
            application
        );
        let parent = staged.ty(&facts, borrowed, context)?;
        assert!(
            matches!(staged.rows.types[parent.0 as usize], s::Type::Reference { referent, .. } if referent == ty)
        );
        let retained = staged.work().reserved_bytes;
        for _ in 0..8 {
            assert_eq!(staged.ty(&facts, borrowed, context)?, parent);
        }
        assert_eq!(staged.work().reserved_bytes, retained);
        parents.push(parent);
        assert!(staged.ty(&facts, tuple, context).is_err());
        assert!(staged.ty(&facts, array, context).is_err());
    }
    assert_ne!(parents[0], parents[1]);
    let scalar = TypeId::primitive(Primitive::U8);
    assert!(
        staged
            .type_key(&facts, scalar, contexts[0])?
            .context
            .is_none()
    );
    assert_eq!(
        staged.ty(&facts, scalar, contexts[0])?,
        staged.ty(&facts, scalar, contexts[1])?
    );
    let added_apps = staged.graph.applications.len() - before_apps;
    let added_types = staged.rows.types.len() - before_types;
    assert_eq!(added_apps, 2);
    assert!(added_types <= 2 * added_apps);
    key_layout(item, contexts[0]);
    Ok(())
}

#[test]
fn actual_projection_declarations_keep_their_wider_transport_requirement() -> Result {
    let (report, mut facts) = prepared(
        "pub trait Family{type Output;}pub fn projected<T:Family>(value:T::Output)->T::Output{value}",
    )?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "projected"
        })
        .ok_or("actual projected function")?;
    let Some(Kind::Function {
        parameters: inputs, ..
    }) = facts.types.kind(function.signature)
    else {
        return Err("actual projected signature".into());
    };
    let projection = facts
        .types
        .list(inputs)
        .and_then(|types| types.first())
        .copied()
        .ok_or("actual projection input")?;
    assert!(matches!(
        facts.types.kind(projection),
        Some(Kind::Projection(_))
    ));
    assert!(
        Stage::begin_with_ordinary_traits(
            &report,
            &mut facts,
            &report
                .package
                .source_core()
                .ok_or("actual provider")?
                .provider
        )
        .is_err()
    );
    Ok(())
}

fn restage_items(
    interface: &crates::Interface,
    rows: s::SourceRows,
    work: &mut g::Work,
) -> Result<std::result::Result<g::Payload, &'static str>> {
    let (_, attributes, modules) = crate::module_packets::raw(interface)?;
    Ok(g::Payload::from_source_module_body_operations(
        interface.identity(),
        rows,
        attributes,
        modules,
        interface
            .source_descriptor()
            .ok_or("actual nested item descriptor")?
            .availability()?
            .clone(),
        0,
        work,
    ))
}

#[test]
fn nested_item_ownership_rejects_orphans_and_scales_with_shared_requests() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut extra =
            String::from("fn leaf(x:u8)->u8{x}fn pass<T>(x:T)->T{x}pub fn calls()->u8{");
        for _ in 0..width {
            extra.push_str("let(f,)=pass((leaf,));let _=f(7);");
        }
        extra.push_str("1}");
        let interface = crate::module_packets::exported_profile(
            &source(&extra)?,
            crate::lower::instances::SourcePublication::ModuleBodyOperations,
            "nested_item_ownership",
        )?;
        let (rows, _, _) = crate::module_packets::raw(&interface)?;
        let mut work = g::Work::default();
        restage_items(&interface, rows.clone(), &mut work)??;
        if let Some(before) = previous {
            assert!(work.steps <= 5 * before);
        }
        previous = Some(work.steps);

        let mut missing = rows.clone();
        for input in &mut missing.body_inputs {
            input.obligations.retain(|input| {
                !matches!(input.kind, s::InputKind::WellFormed(ty)
                    if matches!(rows.types[ty.0 as usize], s::Type::FunctionItem(_)))
            });
        }
        assert_eq!(
            restage_items(&interface, missing, &mut g::Work::default())?.err(),
            Some("source operation omitted a required nontrivial formation input")
        );

        // This well-formed pass::<u8> application is not requested by a body
        // or retained formation. An existing nested leaf cannot own a peer.
        let mut changed = rows;
        let declaration = raw_header(&changed, "pass")?.declaration;
        let scalar = changed
            .types
            .iter()
            .position(|ty| *ty == s::Type::Scalar(crates::Scalar::U8))
            .ok_or("actual u8 recipe")?;
        let start = u32::try_from(changed.arguments.len())?;
        changed
            .arguments
            .push(s::Argument::Type(g::TypeRef(u32::try_from(scalar)?)));
        changed.function_applications.push(s::FunctionApplication {
            declaration: s::DeclarationAddress {
                artifact: s::ArtifactRef(0),
                declaration,
            },
            arguments: g::Range {
                start,
                end: start + 1,
            },
        });
        assert_eq!(
            restage_items(&interface, changed, &mut g::Work::default())?.err(),
            Some("source application has no actual owning header/body request")
        );
    }
    Ok(())
}
