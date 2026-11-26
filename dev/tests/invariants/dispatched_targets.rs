//! Publication retains the exact normalized callable of each proved source original.
use super::*;
use crate::{lower::instances::SourcePublication, resolve::unit::FileId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source_originals(
    fixture: &str,
) -> Result<(
    crate::sema::model::Declarations,
    crate::support::crates::Interface,
)> {
    let foundation =
        crate::test_source::fixture("pa31/tests/spec/source-enum-role-transport/provider.rs")?;
    let prefix = foundation
        .split_once("#[rustc_diagnostic_item = \"Option\"]")
        .ok_or("actual foundation boundary")?
        .0;
    let tail = crate::test_source::fixture(&format!("pa31/tests/spec/{fixture}/provider.rs"))?;
    let report = crate::test_source::source_provider(
        &format!("#![feature(auto_traits)]\n{prefix}{tail}"),
        "retained_dispatch",
    )?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let (_, artifact) = crate::lower::instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleBodyOperations,
    )?;
    Ok((facts, artifact))
}

#[test]
fn original_dispatch_results_keep_owner_site_application_and_payment_boundaries() -> Result {
    let (mut facts, artifact) = source_originals("source-selected-projection-calls")?;
    let rows = facts
        .methods
        .operands
        .as_deref()
        .and_then(|rows| rows.dispatched.as_deref())
        .ok_or("actual completed source dispatch table")?
        .iter()
        .map(|(&key, &target)| (key, target))
        .collect::<Vec<_>>();
    assert!(!rows.is_empty());
    let mut projection_difference = false;
    for ((owner, site, original), target) in rows {
        assert_eq!(
            facts.methods.dispatched(owner, site, original),
            Some(target)
        );
        let foreign = NodeKey {
            file: FileId(u32::MAX),
            ..owner
        };
        assert_eq!(facts.methods.dispatched(foreign, site, original), None);
        assert_eq!(facts.methods.dispatched(owner, foreign, original), None);
        assert_eq!(facts.methods.dispatched(owner, site, TypeId::UNIT), None);
        let raw = raw_callable(&mut facts, site, original, target)?;
        if let Some(raw) = raw
            && raw != target
            && facts.types.contains_projections(raw) == Some(true)
        {
            projection_difference = true;
            let mut dispatcher = crate::sema::dispatch::Dispatcher::new(0, 1)?;
            assert_eq!(dispatcher.select(&mut facts, original)?, target);
            let selections = facts.dispatch_dependencies.work.selections;
            assert_eq!(dispatcher.select(&mut facts, raw)?, target);
            assert!(facts.dispatch_dependencies.work.selections <= selections);
            let proof = dispatcher.finish();
            assert!(proof.self_contained_in(&facts.types, 0)?);
            crate::sema::discharge::quantified(&facts, &proof, 0, "invalid projection query")?;
        }
        let denied = |_, _| Err("retention denied");
        facts
            .methods
            .retain_dispatched(owner, site, original, target, &denied)?;
        assert!(
            facts
                .methods
                .retain_dispatched(owner, site, original, TypeId::UNIT, &|_, _| Ok(()))
                .is_err()
        );
        assert_eq!(
            facts.methods.dispatched(owner, site, original),
            Some(target)
        );
        assert_eq!(
            facts
                .methods
                .retain_dispatched(foreign, site, original, target, &denied)
                .err(),
            Some("retention denied")
        );
        assert_eq!(facts.methods.dispatched(foreign, site, original), None);
    }
    assert!(
        projection_difference,
        "the actual source must require its normalized target"
    );
    let fresh = crate::support::crates::decode(&crate::support::crates::encode(&artifact)?)?;
    crate::source_packets::client_analysis(
        fresh,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observe()->u32{base::direct(11)}",
    )?;
    Ok(())
}

fn raw_callable(
    facts: &mut crate::sema::model::Declarations,
    site: NodeKey,
    original: TypeId,
    target: TypeId,
) -> Result<Option<TypeId>> {
    Ok(match facts.types.kind(original) {
        Some(crate::types::Kind::FunctionItem { .. }) => {
            crate::sema::dispatch::selected_callable_with_projections(
                facts,
                site,
                original,
                &|_, _| Ok(()),
            )?
        }
        Some(crate::types::Kind::Function { .. }) => {
            assert!(matches!(
                facts.types.kind(target),
                Some(crate::types::Kind::Function { .. })
            ));
            None
        }
        _ => return Err("dispatch retention has an unrelated type kind".into()),
    })
}

#[test]
fn query_proofs_reject_foreign_variables_and_unproved_universal_relations() -> Result {
    use crate::types::{RegionId, lifetime_proof::Obligations};
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let crate::pipeline::Analysis { facts, .. } = crate::pipeline::analyze_source(&report)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut proof = Obligations::new(7);
    proof.outlives.push((a, a));
    assert!(!proof.self_contained(&facts.types)?);
    assert!(proof.self_contained_in(&facts.types, 1)?);
    crate::sema::discharge::quantified(&facts, &proof, 1, "invalid universal relation")?;
    proof.outlives.push((a, b));
    assert!(!proof.self_contained_in(&facts.types, 1)?);
    assert!(proof.self_contained_in(&facts.types, 2)?);
    assert!(
        crate::sema::discharge::quantified(&facts, &proof, 2, "invalid universal relation")
            .is_err()
    );
    proof.outlives.clear();
    proof.outlives.push((RegionId(6), a));
    assert!(!proof.self_contained_in(&facts.types, 1)?);
    Ok(())
}

#[test]
fn completed_signatures_keep_exact_dispatch_identity_and_lexical_safety() -> Result {
    use crate::support::crates;
    let (mut facts, artifact) = source_originals("source-dispatched-call-signatures")?;
    let signatures = facts
        .methods
        .operands
        .as_deref()
        .and_then(|rows| rows.dispatched.as_deref())
        .ok_or("actual dispatch rows")?
        .iter()
        .filter(|((_, _, original), _)| {
            matches!(
                facts.types.kind(*original),
                Some(crate::types::Kind::Function { .. })
            )
        })
        .map(|(&key, &target)| (key, target))
        .collect::<Vec<_>>();
    assert!(
        !signatures.is_empty(),
        "the fixture must rewrite actual opened call signatures"
    );
    for ((owner, site, original), target) in signatures {
        assert_ne!(original, target);
        assert_eq!(facts.types.contains_function_items(original), Some(true));
        assert_eq!(facts.types.contains_function_items(target), Some(true));
        assert_eq!(
            facts.methods.dispatched(owner, site, original),
            Some(target)
        );
        let foreign = NodeKey {
            file: FileId(u32::MAX),
            ..owner
        };
        assert_eq!(facts.methods.dispatched(foreign, site, original), None);
        assert_eq!(facts.methods.dispatched(owner, foreign, original), None);
        assert_eq!(facts.methods.dispatched(owner, site, TypeId::UNIT), None);
        assert!(
            facts
                .methods
                .retain_dispatched(owner, site, original, TypeId::UNIT, &|_, _| Ok(()))
                .is_err()
        );
    }
    let fresh = crates::decode(&crates::encode(&artifact)?)?;
    check_signature_rows(&fresh)?;
    crate::source_packets::client_analysis(
        fresh,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observe()->u32{base::mapped(11)}",
    )?;
    Ok(())
}

fn check_signature_rows(artifact: &crate::support::crates::Interface) -> Result {
    use crate::support::crates::generics::{self as g, source as s};
    let (mut rows, attributes, modules) = crate::module_packets::raw(artifact)?;
    for (name, safety) in [
        ("explicit_unsafe", s::CallSafety::ExplicitUnsafeBlock),
        ("unsafe_scope", s::CallSafety::UnsafeFunction),
    ] {
        let body = rows
            .originals
            .iter()
            .find(|body| {
                rows.declarations[body.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some(name)
            })
            .ok_or("actual unsafe caller")?;
        let call = body
            .calls
            .iter()
            .find(|call| {
                let target = rows.function_applications[call.target.0 as usize]
                    .declaration
                    .declaration;
                rows.declarations[target.0 as usize].name.as_deref() == Some("unsafe_apply")
            })
            .ok_or("actual rewritten unsafe call")?;
        assert_eq!(call.safety, safety, "{name}");
    }
    let body = |name| {
        rows.originals.iter().position(|body| {
            rows.declarations[body.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
    };
    let through = body("through").ok_or("actual first named-item caller")?;
    let distinct = body("distinct").ok_or("actual distinct implementation caller")?;
    let call = rows.originals[through].calls[0];
    let wrong = rows.originals[distinct].calls[0].signature;
    assert_ne!(call.signature, wrong);
    rows.originals[through].calls[0].signature = wrong;
    let s::ValueKind::Call { signature, .. } =
        &mut rows.originals[through].values[call.value as usize].kind
    else {
        return Err("actual Call value".into());
    };
    *signature = wrong;
    assert!(
        g::Payload::from_source_module_body_operations(
            artifact.identity(),
            rows,
            attributes,
            modules,
            artifact
                .source_descriptor()
                .ok_or("actual descriptor")?
                .availability()?
                .clone(),
            0,
            &mut g::Work::default(),
        )
        .is_err(),
        "equal-arity function-item argument identity is mandatory"
    );
    Ok(())
}

#[test]
fn captured_protocol_requests_keep_their_semantic_callable_and_lifetime_frame() -> Result {
    use crate::{
        sema::{
            model::Operation,
            protocol_plans::{self, Kind},
        },
        support::crates,
        types,
    };
    let (facts, artifact) = source_originals("source-captured-protocol-requests")?;
    let retained = facts
        .methods
        .operands
        .as_deref()
        .and_then(|rows| rows.dispatched.as_deref())
        .ok_or("actual source dispatch replacements")?;
    let mut captured_mismatch = false;
    for (&(owner, site, original), &selected) in retained {
        let Some(types::Kind::FunctionItem {
            signature: original_signature,
            ..
        }) = facts.types.kind(original)
        else {
            continue;
        };
        let Some(types::Kind::FunctionItem {
            signature: selected_signature,
            ..
        }) = facts.types.kind(selected)
        else {
            continue;
        };
        let Some(Operation::MethodReceiver { plan: id }) =
            facts.node(site).and_then(|node| node.operation)
        else {
            continue;
        };
        let plan = id.current(&facts, owner, site)?;
        let Kind::Receiver { steps, .. } = plan.kind else {
            return Err("actual receiver plan".into());
        };
        for step in protocol_plans::access::rows(&facts, plan, owner, steps)? {
            if step.call.callable != original {
                continue;
            }
            let opening = |signature| {
                types::callable::late_result::opening_regions(
                    &facts.types,
                    signature,
                    step.call.signature,
                    &|_, _| Ok(()),
                )
            };
            if opening(selected_signature)?.is_none() {
                assert!(opening(original_signature)?.is_some());
                captured_mismatch = true;
            }
        }
    }
    assert!(
        captured_mismatch,
        "the fixture must need the original captured-lifetime frame"
    );
    let fresh = crates::decode(&crates::encode(&artifact)?)?;
    check_protocol_request_parents(&fresh)?;
    crate::source_packets::client_analysis(
        fresh,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observe()->u32{let x=11;base::through(&base::Outer(base::Inner{value:&x}))}",
    )?;
    Ok(())
}

fn check_protocol_request_parents(artifact: &crate::support::crates::Interface) -> Result {
    let (rows, _, _) = crate::module_packets::raw(artifact)?;
    for (name, role) in [
        ("through", "Deref"),
        ("coerced", "Deref"),
        ("field", "Deref"),
        ("indexed", "Index"),
        ("direct_index", "Index"),
        ("replaced", "DerefMut"),
    ] {
        let body = rows
            .originals
            .iter()
            .find(|body| {
                rows.declarations[body.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some(name)
            })
            .ok_or("actual protocol caller")?;
        assert!(
            body.calls.iter().any(|call| {
                let declaration = rows.function_applications[call.target.0 as usize]
                    .declaration
                    .declaration;
                rows.functions
                    .iter()
                    .find(|header| header.declaration == declaration)
                    .and_then(|header| header.parent)
                    .is_some_and(|parent| {
                        rows.declarations[parent.0 as usize].name.as_deref() == Some(role)
                    })
            }),
            "{name} must retain its actual {role} trait request"
        );
    }
    Ok(())
}
