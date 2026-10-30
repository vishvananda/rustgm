//! Independently observed reservation declarations never become trait-use evidence.
use super::*;
use crate::{pipeline, test_source};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-reservation-impl/{name}.rs"
    ))?)
}
fn report(name: &str) -> TestResult<crate::resolve::expansion::Report> {
    let source = input(name)?;
    Ok(
        if matches!(
            name,
            "copy_physical_ok" | "copy_physical_bad" | "copy_use_denied" | "auto_structural_kept"
        ) {
            test_source::source_provider(&source, "reservation_source")?
        } else {
            test_source::resolve(&source, true)?
        },
    )
}
fn checked(name: &str) -> TestResult<(crate::resolve::expansion::Report, pipeline::Analysis)> {
    let report = report(name)?;
    let analysis = pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
    Ok((report, analysis))
}
#[test]
fn reservation_headers_keep_ordinary_assumptions_and_independent_implementations() -> TestResult {
    for name in [
        "unused",
        "empty_message",
        "assumption",
        "real_positive",
        "const_unused",
        "const_real",
        "wf_exemption",
        "unsafe_valid",
        "body_explicit_parameter_bound",
        "body_independent_real_impl",
        "associated_with_real_impl",
        "coherence_clear",
        "coherence_incompatible_subject",
        "coherence_incompatible_arguments",
        "macro_allowance",
        "cfg_removed",
        "constant_unused",
        "copy_physical_ok",
    ] {
        let (report, analysis) = checked(name)?;
        if name == "associated_with_real_impl" {
            let function = analysis
                .facts
                .functions
                .iter()
                .find(|function| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[function.definition.0 as usize].name)
                        == "call"
                })
                .ok_or("actual projected signature")?;
            let Some(Kind::Function { parameters, .. }) =
                analysis.facts.types.kind(function.signature)
            else {
                return Err("actual function signature".into());
            };
            assert_eq!(
                analysis.facts.types.list(parameters),
                Some(
                    [crate::types::TypeId::primitive(
                        crate::types::Primitive::U32
                    )]
                    .as_slice()
                )
            );
        }
    }
    Ok(())
}
#[test]
fn reservation_never_supplies_call_projection_const_or_own_body_evidence() -> TestResult {
    for name in [
        "use_denied",
        "bound_denied",
        "projection_denied",
        "associated_type_probe",
        "associated_type_requires_real_self_bound",
        "body_own_trait_probe",
        "const_denied",
        "const_body_restriction",
        "wf_positive_control",
        "coherence_reserved",
        "constant_use_denied",
        "copy_use_denied",
        "copy_physical_bad",
        "auto_structural_kept",
    ] {
        let outcome: TestResult<pipeline::Analysis> =
            report(name).and_then(|report| pipeline::analyze_source(&report).map_err(Into::into));
        let error = outcome
            .err()
            .ok_or_else(|| format!("{name}: accepted without genuine proof"))?
            .to_string();
        let expected: &[&str] = match name {
            "coherence_reserved" => &["overlap", "coherence"],
            "const_body_restriction" => &["const"],
            "copy_physical_bad" => &["Copy", "field"],
            _ => &["trait", "implementation", "associated"],
        };
        assert!(
            expected.iter().any(|word| error.contains(word)),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn reservation_attribute_keeps_actual_gate_owner_signature_safety_and_duplicate_checks()
-> TestResult {
    for name in [
        "missing_gate",
        "word",
        "number",
        "list",
        "duplicate",
        "inherent",
        "function",
        "negative",
        "negative-trait",
        "missing_method",
        "wrong_signature",
        "unsafe_missing",
        "unsafe_wrapper",
    ] {
        let outcome: TestResult<pipeline::Analysis> =
            report(name).and_then(|report| pipeline::analyze_source(&report).map_err(Into::into));
        let error = outcome
            .err()
            .ok_or_else(|| format!("{name}: accepted malformed declaration"))?
            .to_string();
        let expected: &[&str] = match name {
            "missing_method" | "wrong_signature" => &["method", "signature"],
            "unsafe_missing" => &["safety", "unsafe"],
            "missing_gate" => &["rustc_attrs"],
            "negative" => &["custom source auto traits are not implemented"],
            _ => &["reservation"],
        };
        assert!(
            expected.iter().any(|word| error.contains(word)),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn checked_reservation_has_no_selected_member_but_real_exported_body_can_lower() -> TestResult {
    for name in [
        "real_positive",
        "const_real",
        "reservation_body_direct_export",
    ] {
        let (
            report,
            pipeline::Analysis {
                mut facts,
                prepared,
            },
        ) = checked(name)?;
        let registry = facts.generics.implementations.as_ref().ok_or("registry")?;
        let reserved = registry
            .headers()
            .find(|header| {
                registry.polarity(&facts.types, &facts.associated, header.owner)
                    == Ok(Polarity::Reservation)
            })
            .ok_or("actual reservation")?;
        assert!(registry.is_validated(reserved.owner));
        assert!(
            reserved
                .methods
                .iter()
                .all(|&(member, _)| registry.method(reserved.owner, member).is_none())
        );
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let mut output = Vec::new();
        crate::lowir::writer::write(&program, &mut output)?;
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../obj/parallel/reservation-impl-v1/candidate");
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join(format!("{name}.lowir")), output)?;
    }
    Ok(())
}
#[test]
fn reservation_source_witnesses_are_current_and_failed_queries_retain_work() -> TestResult {
    let (report, mut analysis) = checked("unused")?;
    assert!(report.package.library_policy().is_none());
    let header = analysis
        .facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .headers()
        .next()
        .ok_or("header")?;
    let owner = header.owner;
    let key = header.source.ok_or("source")?;
    let producer = report.lint_producer().ok_or("producer")?;
    let before = producer.work();
    let annotation = producer
        .declaration_annotation(&report.package, key)
        .map_err(crate::resolve::package::Error::from)?
        .ok_or("annotation")?;
    assert_eq!(annotation.reservation.ok_or("marker")?.1, "future impl");
    let other = test_source::resolve(&input("unused")?, true)?;
    assert!(
        producer
            .declaration_annotation(&other.package, key)
            .is_err()
    );
    assert!(producer.work().steps > before.steps);
    let warm = producer.work();
    producer
        .declaration_annotation(&report.package, key)
        .map_err(crate::resolve::package::Error::from)?;
    assert_eq!(producer.work().bytes, warm.bytes);
    let _ = producer.diagnostic_payload(key, 10_000_000, 0);
    let failed = producer.work();
    for _ in 0..2 {
        assert!(
            producer
                .declaration_annotation(&report.package, key)
                .is_err()
        );
    }
    assert!(producer.work().steps >= failed.steps + 2);
    let id = analysis.facts.associated.owner(owner).ok_or("owner")?;
    if let crate::sema::associated::OwnerKind::Impl { polarity, .. } = &mut analysis
        .facts
        .associated
        .owner_for_test(id)
        .ok_or("owner")?
        .kind
    {
        *polarity = Polarity::Positive;
    }
    assert!(
        analysis
            .facts
            .generics
            .implementations
            .as_ref()
            .ok_or("registry")?
            .polarity(&analysis.facts.types, &analysis.facts.associated, owner)
            .is_err()
    );
    Ok(())
}
#[test]
fn reservation_work_is_linear_and_hot_layouts_stay_fixed() -> TestResult {
    let mut previous = (0, 0);
    for width in [8, 32, 128] {
        let mut source = "#![feature(rustc_attrs)]trait A{}\n".to_owned();
        for index in 0..width {
            writeln!(
                source,
                "struct S{index};#[rustc_reservation_impl=\"future impl\"]impl A for S{index}{{}}"
            )?;
        }
        let report = test_source::resolve(&source, true)?;
        let _ = pipeline::analyze_source(&report)?;
        let work = report.lint_producer().ok_or("producer")?.work();
        println!(
            "reservation width={width} steps={} bytes={}",
            work.steps, work.bytes
        );
        if previous.0 != 0 {
            assert!(work.steps < previous.0 * 6);
            assert!(work.bytes < previous.1 * 6);
        }
        previous = (work.steps, work.bytes);
    }
    println!(
        "reservation polarity={} ownerkind={} owner={} status={} entry={} annotations={}",
        size_of::<Polarity>(),
        size_of::<crate::sema::associated::OwnerKind>(),
        size_of::<crate::sema::associated::Owner>(),
        size_of::<Status>(),
        size_of::<Entry>(),
        size_of::<crate::resolve::lint_policy::DeclarationAnnotation<'_>>()
    );
    println!("cold coherence inputs={}", overlap::input_layout());
    assert_eq!(size_of::<Polarity>(), 1);
    assert_eq!(size_of::<crate::sema::associated::OwnerKind>(), 20);
    assert_eq!(size_of::<crate::sema::associated::Owner>(), 32);
    assert_eq!(size_of::<Status>(), 1);
    assert_eq!(size_of::<Entry>(), 136);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn old_metadata_profiles_cannot_drop_actual_reservation_polarity() -> TestResult {
    let source = input("module-provider")?;
    let report = test_source::source_provider(&source, "future_impl")?;
    let producer = report.lint_producer().ok_or("actual provider producer")?;
    for feature_predicates in [false, true] {
        let error = producer
            .require_impl_annotation_transport(&report.package, feature_predicates)
            .err()
            .ok_or("reservation polarity lost at annotation boundary")?;
        assert_eq!(
            error.message,
            "reservation implementation transport requires a wider metadata profile"
        );
    }
    let error = crate::module_packets::exported_profile(
        &source,
        crate::lower::instances::SourcePublication::ModuleCallables,
        "future_impl",
    )
    .err()
    .ok_or("reservation transport was silently erased")?;
    assert!(error.to_string().contains("reservation"), "{error}");
    Ok(())
}

#[test]
fn actual_local_trait_bounds_distinguish_closed_absence_from_real_or_open_overlap() -> TestResult {
    for (name, accepted) in [
        ("closed-local", true),
        ("real-bound-overlap", false),
        ("open-bound-overlap", false),
    ] {
        let source =
            test_source::fixture(&format!("pa20/tests/spec/local-bound-coherence/{name}.rs"))?;
        let report = test_source::resolve(&source, true)?;
        let outcome = pipeline::analyze_source(&report);
        assert_eq!(
            outcome.is_ok(),
            accepted,
            "{name}: {}",
            outcome.err().map_or(String::new(), |e| e.to_string())
        );
    }
    Ok(())
}

#[test]
fn failed_coherence_scratch_payment_cannot_publish_a_pair_result() -> TestResult {
    let (_report, mut analysis) = checked("coherence_incompatible_subject")?;
    let registry = analysis
        .facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?;
    let pair = registry
        .headers()
        .filter(|header| {
            analysis
                .facts
                .types
                .contains_parameters(header.predicate.ty)
                == Some(true)
        })
        .map(|header| (header.predicate, header.owner))
        .next()
        .ok_or("actual blanket")?;
    let concrete = registry
        .headers()
        .find(|header| header.owner != pair.1 && header.predicate.bound == pair.0.bound)
        .map(|header| (header.predicate, header.owner))
        .ok_or("actual competing header")?;
    let _ = crate::sema::header_attributes::reserve_source_contracts(
        &mut analysis.facts,
        0,
        crate::support::crates::MAX_BYTES,
    );
    let before = crate::sema::header_attributes::source_contract_costs(&analysis.facts);
    assert!(overlap::possible(&mut analysis.facts, pair, concrete).is_err());
    let after = crate::sema::header_attributes::source_contract_costs(&analysis.facts);
    assert!(after.0 > before.0 && after.1 >= before.1);
    Ok(())
}
