//! Genuine source storage shape is independent of executable state-machine proof.
use crate::resolve as source;
use rustgm::{
    abi::layout::Layouts,
    sema,
    types::{self, Kind, TypeId, auto_traits},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn marker(
    facts: &mut sema::model::Declarations,
    ty: TypeId,
    marker: auto_traits::AutoTrait,
) -> Result<bool> {
    let mut query = auto_traits::Query::new(auto_traits::Obligation { ty, marker });
    let step = query.advance(&mut facts.types, &mut facts.aggregates, |_, _| {
        Ok(auto_traits::Rule::Structural)
    })?;
    let auto_traits::Step::Ready(value) = step else {
        return Err("source capture query lost a concrete field".into());
    };
    Ok(value)
}

#[test]
fn actual_typed_future_shapes_preserve_capture_identity_and_do_not_capture_output() -> Result {
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/source-shape.rs")?;
    let (_, authored) = text
        .split_once("pub async fn scalar")
        .ok_or("authored declarations")?;
    let companion = source::fixture("pa43/tests/spec/future-runtime-v1/shape-companion.rs")?;
    assert_eq!(format!("pub async fn scalar{authored}"), companion);
    let report = source::source_provider(&text, "authored_shapes")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let mut copies = types::copy::Copies::default();
    let mut layouts = Layouts::default();
    for (name, fields, size, send) in [
        ("scalar", 2, 2, true),
        ("raw", 2, 16, false),
        ("output", 1, 1, true),
    ] {
        let function = facts
            .functions
            .iter()
            .find(|function| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == name
            })
            .ok_or("actual async declaration")?;
        let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
            return Err("actual constructor signature".into());
        };
        let Some(Kind::Opaque(identity, arguments)) = facts.types.kind(result) else {
            return Err("actual opaque identity".into());
        };
        let ty = facts.types.intern(Kind::Future {
            identity,
            arguments,
        })?;
        assert_ne!(ty, result);
        assert!(
            facts.opaques.declarations[&identity].hidden.is_none(),
            "storage data supplied execution authority"
        );
        assert!(!marker(&mut facts, ty, auto_traits::AutoTrait::Unpin)?);
        assert_eq!(
            marker(&mut facts, ty, auto_traits::AutoTrait::Send)?,
            send,
            "{name}"
        );
        assert_eq!(
            marker(&mut facts, ty, auto_traits::AutoTrait::Sync)?,
            send,
            "{name}"
        );
        assert_eq!(
            copies.demand(
                &facts.types,
                &facts.aggregates,
                &std::collections::HashMap::<types::ItemId, types::copy::Declaration>::new(),
                ty
            )?,
            types::copy::Properties::default()
        );
        let layout = layouts.demand(&facts.types, &facts.aggregates, ty)?;
        let layout = layouts.at(layout).ok_or("actual future layout")?;
        assert_eq!(layout.size, size, "{name}");
        let [variant] = layouts.variants(layout) else {
            return Err("actual anonymous future sequence".into());
        };
        assert_eq!(
            facts
                .types
                .list(variant.fields)
                .ok_or("actual future fields")?
                .len(),
            fields
        );
        let forged = facts.types.intern_regions(&[types::RegionId::STATIC])?;
        assert!(
            facts
                .types
                .intern(Kind::Future {
                    identity,
                    arguments: forged
                })
                .is_err(),
            "accepted forged complete capture arity"
        );
    }
    assert_eq!(size_of::<types::Kind>(), 24);
    assert_eq!(size_of::<rustgm::body::ir::Owner>(), 20);
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    assert_eq!(types::Store::opaque_declaration_size(), 16);
    assert_eq!(size_of::<types::opaque::Declaration>(), 4);
    Ok(())
}

#[test]
fn complete_generic_applications_reuse_one_identity_and_reject_argument_kind_forgery() -> Result {
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/source-shape.rs")?;
    let report = source::source_provider(&text, "generic_shapes")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "generic"
        })
        .ok_or("actual generic async declaration")?;
    let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("actual generic signature".into());
    };
    let Some(Kind::Opaque(identity, _)) = facts.types.kind(result) else {
        return Err("actual generic opaque".into());
    };
    let byte = TypeId::primitive(types::Primitive::U8);
    let pointer = facts.types.intern(Kind::Pointer {
        mutable: true,
        pointee: byte,
    })?;
    let mut layouts = Layouts::default();
    let mut previous = None;
    for (input, send) in [(byte, true), (pointer, false)] {
        let arguments = facts
            .types
            .intern_arguments(&[types::arguments::Argument::Type(input)])?;
        let kind = Kind::Future {
            identity,
            arguments,
        };
        let ty = facts.types.intern(kind)?;
        assert_eq!(
            facts.types.intern(kind)?,
            ty,
            "warm application lost identity"
        );
        assert_ne!(previous, Some(ty));
        previous = Some(ty);
        assert_eq!(marker(&mut facts, ty, auto_traits::AutoTrait::Send)?, send);
        let layout = layouts.demand(&facts.types, &facts.aggregates, ty)?;
        let layout = layouts.at(layout).ok_or("actual generic future layout")?;
        let [variant] = layouts.variants(layout) else {
            return Err("actual generic storage sequence".into());
        };
        assert_eq!(
            facts.types.list(variant.fields),
            Some([byte, input].as_slice())
        );
    }
    let arguments = facts.types.intern_regions(&[types::RegionId::STATIC])?;
    assert!(
        facts
            .types
            .intern(Kind::Future {
                identity,
                arguments
            })
            .is_err(),
        "a type formal accepted a lifetime"
    );
    Ok(())
}
