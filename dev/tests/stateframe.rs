//! Genuine source storage shape is independent of executable state-machine proof.
use crate::resolve as source;
use rustgm::{
    abi::layout::Layouts,
    sema,
    types::{self, Kind, TypeId, auto_traits},
};
#[path = "helpers/asynccheck.rs"]
mod checks;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn ordinary_opaque_results_do_not_request_authored_future_preparation() -> Result {
    let mut report = source::resolve("fn value()->impl Copy{3u8}fn main(){let _=value();}", true)?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    assert!(!facts.uses_source_copy());
    let entry = sema::entry::select(&mut report, &facts)?;
    rustgm::lower::instances::build(&report, &mut facts, entry)?;
    assert_eq!(size_of::<types::Store>(), 472);
    Ok(())
}

#[test]
fn mixed_future_and_ordinary_results_publish_only_actual_checked_batch_families() -> Result {
    let mut text = source::fixture("pa43/tests/spec/future-runtime-v1/construction-source.rs")?;
    text.push_str("\nfn ordinary()->impl Copy{3u8}\n");
    let report = source::source_provider(&text, "mixed_future_results")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
    Ok(())
}

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
    use rustgm::body;
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/source-shape.rs")?;
    let report = source::source_provider(&text, "deferred_originals")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
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

#[test]
fn selected_future_applications_use_actual_checked_bodies_and_complete_capture_frames() -> Result {
    use rustgm::{body, types::arguments::Argument};
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/source-shape.rs")?;
    let report = source::source_provider(&text, "selected_async")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    for (name, argument) in [
        (
            "generic",
            Argument::Type(TypeId::primitive(types::Primitive::U8)),
        ),
        ("borrowed", Argument::Lifetime(types::RegionId::STATIC)),
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
            .ok_or("actual async function")?;
        let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
            return Err("actual source signature".into());
        };
        let Some(Kind::Opaque(identity, _)) = facts.types.kind(result) else {
            return Err("actual source identity".into());
        };
        let arguments = facts.types.intern_arguments(&[argument])?;
        let future = facts.types.intern(Kind::Future {
            identity,
            arguments,
        })?;
        let application = rustgm::pipeline::future_application(&mut facts, &checked, future)?;
        assert!(
            matches!(application.body().owner, body::ir::Owner::AsyncApplication { identity: actual, future: ty, signature }
            if actual == identity && ty == future && Some(signature) == application.contract().signature)
        );
        assert!(application.body().instance.is_none());
        assert_eq!(
            application.contract().key,
            body::ir::BodyKey::AsyncApplication(future)
        );
        assert_eq!(
            application.body().result,
            TypeId::primitive(types::Primitive::U8)
        );
        assert!(
            !facts
                .types
                .contains_parameters(application.body().result)
                .ok_or("actual body output")?
        );
        let repeated = rustgm::pipeline::future_application(&mut facts, &checked, future)?;
        assert_eq!(application.body().source, repeated.body().source);
        assert_eq!(
            application.body().blocks.len(),
            repeated.body().blocks.len()
        );
        assert!(facts.opaques.declarations[&identity].hidden.is_some());
    }
    assert!(rustgm::pipeline::future_application(&mut facts, &checked, TypeId::UNIT).is_err());
    Ok(())
}

#[test]
fn selected_future_applications_revalidate_store_and_warm_publication() -> Result {
    use rustgm::{body, types::arguments::Argument};
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/source-shape.rs")?;
    let report = source::source_provider(&text, "selected_authentication")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    let future = checks::future(
        &report,
        &mut facts,
        "generic",
        &[Argument::Type(TypeId::primitive(types::Primitive::U8))],
    )?;
    let first = rustgm::pipeline::future_application(&mut facts, &checked, future)?;
    let warm = rustgm::pipeline::future_application(&mut facts, &checked, future)?;
    assert!(first.capture_mapping_bytes() > 0);
    assert_eq!(first.capture_mapping_bytes(), warm.capture_mapping_bytes());
    let mut foreign = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let foreign_future = checks::future(
        &report,
        &mut foreign,
        "generic",
        &[Argument::Type(TypeId::primitive(types::Primitive::U8))],
    )?;
    let failure = rustgm::pipeline::future_application(&mut foreign, &checked, foreign_future)
        .err()
        .ok_or("accepted checked source from another store")?;
    assert!(failure.contains("another actual type store"), "{failure}");
    let Some(Kind::Future { identity, .. }) = facts.types.kind(future) else {
        return Err("actual selected future identity".into());
    };
    facts
        .opaques
        .declarations
        .get_mut(&identity)
        .ok_or("actual owner")?
        .hidden = Some(TypeId::UNIT);
    let failure = rustgm::pipeline::future_application(&mut facts, &checked, future)
        .err()
        .ok_or("accepted stale warm defining publication")?;
    assert!(
        failure.contains("actual future representation"),
        "{failure}"
    );
    assert_eq!(
        size_of::<body::instantiate::FutureApplication>(),
        size_of::<body::instantiate::Materialized>() + 2 * size_of::<usize>()
    );
    println!(
        "actual cold Materialized={} FutureApplication={}",
        size_of::<body::instantiate::Materialized>(),
        size_of::<body::instantiate::FutureApplication>()
    );
    Ok(())
}

#[test]
fn future_formation_proves_closed_bounds_and_retains_unpaid_lifetime_requirements() -> Result {
    use types::arguments::Argument;
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/formation-source.rs")?;
    let companion = source::fixture("pa43/tests/spec/future-runtime-v1/formation-companion.rs")?;
    assert!(text.ends_with(&companion));
    let report = source::source_provider(&text, "future_formation")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    let valid = checks::future(
        &report,
        &mut facts,
        "copy_bound",
        &[Argument::Type(TypeId::primitive(types::Primitive::U8))],
    )?;
    let application = rustgm::pipeline::future_application(&mut facts, &checked, valid)?;
    assert!(application.formation_requirements().is_none());
    let definition = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "NonCopy")
        .ok_or("actual NonCopy declaration")?;
    let non_copy = facts.definitions[definition].ok_or("actual NonCopy nominal type")?;
    let invalid = checks::future(
        &report,
        &mut facts,
        "copy_bound",
        &[Argument::Type(non_copy)],
    )?;
    let failure = rustgm::pipeline::future_application(&mut facts, &checked, invalid)
        .err()
        .ok_or("accepted an application violating its defining Copy premise")?;
    assert!(failure.contains("defining input clause"), "{failure}");
    for (regions, debt) in [
        ([types::RegionId::STATIC; 2], false),
        (
            [types::RegionId::bound(0, 0)?, types::RegionId::bound(0, 1)?],
            true,
        ),
    ] {
        let future = checks::future(
            &report,
            &mut facts,
            "related",
            &regions.map(Argument::Lifetime),
        )?;
        let application = rustgm::pipeline::future_application(&mut facts, &checked, future)?;
        assert_eq!(application.formation_requirements().is_some(), debt);
        if let Some(proof) = application.formation_requirements() {
            assert!(!proof.outlives.is_empty());
        }
    }
    Ok(())
}

#[test]
fn selected_future_bounds_consume_genuine_future_and_actual_output_equality() -> Result {
    use types::arguments::Argument;
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/selection-source.rs")?;
    let companion = source::fixture("pa43/tests/spec/future-runtime-v1/selection-companion.rs")?;
    assert!(text.ends_with(&companion));
    let report = source::source_provider(&text, "future_output_selection")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    for (name, valid) in [("value", true), ("wide", false)] {
        let inner = checks::future(&report, &mut facts, name, &[])?;
        let _ = rustgm::pipeline::future_application(&mut facts, &checked, inner)?;
        let future = checks::future(
            &report,
            &mut facts,
            "require_future",
            &[Argument::Type(inner)],
        )?;
        let selected = rustgm::pipeline::future_application(&mut facts, &checked, future);
        if valid {
            let application = selected?;
            assert_eq!(
                application.body().result,
                TypeId::primitive(types::Primitive::U8)
            );
            assert!(application.formation_requirements().is_none());
            let _ = rustgm::pipeline::future_application(&mut facts, &checked, future)?;
        } else {
            let error = selected
                .err()
                .ok_or("accepted wrong actual future Output")?;
            assert!(error.contains("defining input clause"), "{error}");
        }
    }
    Ok(())
}

#[test]
fn authored_async_by_value_unsized_input_cannot_publish_sized_future_storage() -> Result {
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/unsized-source.rs")?;
    let report = source::source_provider(&text, "unsized_async_input")?;
    let error = sema::declarations::analyze(&report)
        .and_then(|facts| sema::bodies::analyze(&report, facts))
        .err()
        .ok_or("accepted by-value str and fabricated sized future storage")?;
    assert!(
        error.message.to_ascii_lowercase().contains("sized"),
        "{error:?}"
    );
    Ok(())
}

#[test]
fn selected_authored_poll_calls_keep_the_real_member_and_independent_public_signature() -> Result {
    use rustgm::{
        body::ir::Rvalue,
        types::{arguments::Argument, instance::Instance},
    };
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/poll-consumer-source.rs")?;
    let companion =
        source::fixture("pa43/tests/spec/future-runtime-v1/poll-consumer-companion.rs")?;
    let authored = companion.split_once('\n').ok_or("companion foundation")?.1;
    assert!(text.ends_with(authored));
    let report = source::source_provider(&text, "actual_poll_consumer")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    let value = checks::future(&report, &mut facts, "value", &[])?;
    let _ = rustgm::pipeline::future_application(&mut facts, &checked, value)?;
    let future = checks::future(
        &report,
        &mut facts,
        "driven",
        &[
            Argument::Lifetime(types::RegionId::STATIC),
            Argument::Lifetime(types::RegionId::STATIC),
            Argument::Type(TypeId::primitive(types::Primitive::U8)),
        ],
    )?;
    let poll = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "poll"
        })
        .ok_or("actual required poll declaration")?
        .definition;
    let application = rustgm::pipeline::future_application(&mut facts, &checked, future)?;
    let mut calls = 0;
    for expression in &application.body().values {
        let Rvalue::Call { callee, .. } = expression.kind else {
            continue;
        };
        let callee = application
            .body()
            .values
            .get(callee.0 as usize)
            .ok_or("actual call operand")?;
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(callee.ty)
        else {
            continue;
        };
        if item.0 != poll.0 {
            continue;
        }
        let instance = Instance::new(&facts.types, callee.ty)?;
        assert_eq!(application.body().callable_target(instance), instance);
        assert!(matches!(
            facts.types.kind(signature),
            Some(Kind::Function {
                bound_regions: 3,
                ..
            })
        ));
        calls += 1;
    }
    assert_eq!(calls, 1);
    assert_eq!(application.contract().universals, 2);
    assert_eq!(
        application.body().result,
        TypeId::primitive(types::Primitive::U8)
    );
    let repeated = rustgm::pipeline::future_application(&mut facts, &checked, future)?;
    assert_eq!(application.body().source, repeated.body().source);
    Ok(())
}

#[test]
fn unpolled_constructor_and_drop_defer_divergent_body_and_check_unused_output() -> Result {
    let companion = source::fixture("pa43/tests/spec/future-runtime-v1/construction-companion.rs")?;
    for file in ["construction-source.rs", "dropbyte.rs"] {
        let text = source::fixture(&format!("pa43/tests/spec/future-runtime-v1/{file}"))?;
        assert!(text.ends_with(&companion));
        let mut report = source::source_provider(&text, "unpolled_constructor")?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let definition = facts
            .functions
            .iter()
            .find(|f| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[f.definition.0 as usize].name)
                    == "never"
            })
            .ok_or("actual constructor source owner")?
            .definition;
        let entry = sema::entry::select(&mut report, &facts)?;
        let program =
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        let drop = program
            .symbols
            .iter()
            .find(|s| s.name.contains("rg.future.drop.i"))
            .ok_or("actual generated Drop symbol")?;
        assert_eq!(
            drop.body.as_ref().ok_or("actual Drop CFG")?.blocks.len(),
            5,
            "actual state dispatch"
        );
        assert_eq!(
            program.parameters(drop.signature)?,
            [rustgm::lowir::vocabulary::Type::Ptr]
        );
        let prefix = format!("rg.{}.i", definition.0);
        let constructor = program
            .symbols
            .iter()
            .find(|s| s.name.contains(&prefix))
            .ok_or("actual public constructor symbol")?;
        assert_eq!(
            constructor
                .body
                .as_ref()
                .ok_or("actual constructor CFG")?
                .blocks
                .len(),
            1,
            "deferred source loop executed as constructor"
        );
    }
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/construction-wrong-source.rs")?;
    let report = source::source_provider(&text, "wrong_deferred_output")?;
    let error = rustgm::pipeline::analyze_source(&report)
        .err()
        .ok_or("unused wrong async Output accepted")?
        .to_string();
    assert!(
        error.contains("incompatible types") || error.contains("mismatched type"),
        "{error}"
    );
    Ok(())
}

#[test]
fn ordinary_borrowed_moves_cannot_use_generated_state_copy_permission() -> Result {
    let valid = source::fixture("pa43/tests/spec/future-runtime-v1/dropowned.rs")?;
    ordinary_program(&valid, "ordinary_owned_move")?;
    let invalid = source::fixture("pa43/tests/spec/future-runtime-v1/dropmove.rs")?;
    let error = ordinary_program(&invalid, "ordinary_borrowed_move")
        .err()
        .ok_or("ordinary borrowed move acquired compiler state permission")?
        .to_string();
    assert!(
        error.contains("borrowed") || error.contains("cannot move"),
        "{error}"
    );
    Ok(())
}

fn ordinary_program(text: &str, name: &str) -> Result<rustgm::lowir::program::Program> {
    let mut report = source::source_provider(text, name)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let entry = sema::entry::select(&mut report, &facts)?;
    Ok(rustgm::lower::instances::build_prepared(
        &report, &mut facts, entry, prepared,
    )?)
}

#[test]
fn future_drop_recipes_keep_full_applications_and_own_capture_cleanup() -> Result {
    use rustgm::body::drops::glue::{Catalog, Method, Recipe, Shape};
    let text = source::fixture("pa43/tests/spec/future-runtime-v1/source-shape.rs")?;
    let report = source::source_provider(&text, "future_drop_data")?;
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
        .ok_or("actual generic async function")?;
    let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("actual generic constructor signature".into());
    };
    let Some(Kind::Opaque(identity, _)) = facts.types.kind(result) else {
        return Err("actual primary opaque".into());
    };
    let byte = TypeId::primitive(types::Primitive::U8);
    let pointer = facts.types.intern(Kind::Pointer {
        mutable: true,
        pointee: byte,
    })?;
    let mut futures = Vec::new();
    for input in [byte, pointer] {
        let arguments = facts
            .types
            .intern_arguments(&[types::arguments::Argument::Type(input)])?;
        futures.push(facts.types.intern(Kind::Future {
            identity,
            arguments,
        })?);
    }
    let mut catalog = Catalog::new(&facts.types, &facts.aggregates, &facts.members, &[])?;
    let mut previous = None;
    for future in futures {
        let id = catalog
            .demand(future)?
            .ok_or("actual future cleanup obligation")?;
        assert_ne!(
            previous,
            Some(id),
            "distinct capture applications shared a method"
        );
        previous = Some(id);
        assert_eq!(
            catalog.demand(future)?,
            Some(id),
            "warm recipe changed identity"
        );
        let recipe = catalog
            .recipes()
            .get(id.0 as usize)
            .ok_or("actual recipe")?;
        assert_eq!(recipe.ty, future);
        assert_eq!(recipe.method, Some(Method::Future(future)));
        assert!(
            matches!(recipe.kind, Shape::Fields(range) if range.start == range.end),
            "capture cleanup duplicated outside its state entry"
        );
    }
    assert_eq!(catalog.recipes().len(), 2);
    assert!(
        facts.opaques.declarations[&identity].hidden.is_none(),
        "cleanup data granted publication"
    );
    assert_eq!(size_of::<Method>(), 8);
    assert_eq!(size_of::<Recipe>(), 32);
    Ok(())
}
