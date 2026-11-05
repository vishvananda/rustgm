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

#[test]
fn authored_async_originals_keep_their_real_output_through_region_and_loan_checks() -> Result {
    use rustgm::{body, borrow};
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/source-shape.rs")?;
    let report = source::source_provider(&text, "deferred_originals")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let mut inputs = body::regions::collect(&facts)?.bodies;
    for template in &templates.bodies {
        let body::ir::Owner::AsyncBody { identity, .. } = template.body.owner else {
            return Err("source template has another actual owner".into());
        };
        let contract = inputs
            .iter_mut()
            .find(|input| input.key == body::ir::BodyKey::AsyncBody(identity))
            .ok_or("deferred original lost its real input contract")?;
        body::prepare_source_copy(&mut facts, &template.body, contract)?;
        contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    let contracts = body::regions::Contracts {
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
        bodies: inputs,
    };
    let mut copies = types::copy::Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut copies,
    };
    let checked = borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
        &[],
        |_, _| Err("unexpected nominal field in source shape"),
    )?;
    assert_eq!(checked.regions().bodies.len(), 5);
    assert_eq!(checked.loans().len(), 5);
    checked.require_future_store(&facts.types)?;
    for template in &templates.bodies {
        let body::ir::Owner::AsyncBody {
            identity,
            future,
            signature,
        } = template.body.owner
        else {
            return Err("deferred original became its ordinary constructor".into());
        };
        assert!(facts.opaques.declarations[&identity].hidden.is_none());
        assert!(
            matches!(facts.types.kind(future), Some(Kind::Future { identity: actual, .. }) if actual == identity)
        );
        assert!(
            matches!(facts.types.kind(signature), Some(Kind::Function { result, .. }) if result == template.body.result)
        );
        assert!(template.body.instance.is_none());
    }
    let other = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let body::ir::Owner::AsyncBody { future, .. } = templates
        .bodies
        .first()
        .ok_or("actual deferred body")?
        .body
        .owner
    else {
        return Err("actual deferred owner".into());
    };
    assert_eq!(facts.types.kind(future), other.types.kind(future));
    assert!(
        checked.require_future_store(&other.types).is_err(),
        "same-shaped foreign store acquired the checked family"
    );
    checked.require_future_store(&facts.types)?;
    let actual = std::mem::replace(&mut facts.types, other.types);
    assert!(
        checked.require_future_store(&facts.types).is_err(),
        "stale checked family survived replacing its store"
    );
    facts.types = actual;
    checked.require_future_store(&facts.types)?;
    Ok(())
}

#[test]
fn generic_future_templates_defer_unknown_capture_layout_until_real_application() -> Result {
    use rustgm::types::arguments::Argument;
    let text = source::fixture("pa43/tests/spec/deferred-compiler-op/unmarked.rs")?;
    let report = source::source_provider(&text, "generic_future_storage")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "cleanup"
        })
        .ok_or("actual async declaration")?;
    let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("actual source signature".into());
    };
    let Some(Kind::Opaque(identity, formal)) = facts.types.kind(result) else {
        return Err("actual opaque capture frame".into());
    };
    let future = facts.types.intern(Kind::Future {
        identity,
        arguments: formal,
    })?;
    let mut templates = rustgm::abi::templates::Templates::default();
    let mut layouts = Layouts::default();
    assert!(!templates.demand(
        &mut facts.types,
        &mut facts.aggregates,
        &mut layouts,
        future
    )?);
    let symbolic_steps = templates.steps;
    assert!(!templates.demand(
        &mut facts.types,
        &mut facts.aggregates,
        &mut layouts,
        future
    )?);
    assert_eq!(
        templates.steps, symbolic_steps,
        "warm generic layout replayed its graph"
    );
    assert!(
        layouts.get(future).is_none(),
        "unknown generic metadata fabricated a frame layout"
    );
    for (ty, expected) in [(types::Primitive::U8, 16), (types::Primitive::Str, 24)] {
        let captures = facts
            .types
            .arguments(formal)
            .ok_or("actual source captures")?
            .map(|argument| match argument {
                Argument::Lifetime(_) => Ok(Argument::Lifetime(types::RegionId::STATIC)),
                Argument::Type(_) => Ok(Argument::Type(TypeId::primitive(ty))),
                Argument::Const(_) => Err("unexpected const input"),
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let arguments = facts.types.intern_arguments(&captures)?;
        let actual = facts.types.intern(Kind::Future {
            identity,
            arguments,
        })?;
        assert!(templates.demand(
            &mut facts.types,
            &mut facts.aggregates,
            &mut layouts,
            actual
        )?);
        let known_steps = templates.steps;
        assert!(templates.demand(
            &mut facts.types,
            &mut facts.aggregates,
            &mut layouts,
            actual
        )?);
        assert_eq!(
            templates.steps, known_steps,
            "warm actual layout replayed its graph"
        );
        assert_eq!(
            layouts
                .get(actual)
                .ok_or("actual closed frame layout")?
                .size,
            expected
        );
    }
    Ok(())
}
