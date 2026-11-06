//! Actual `ConstParamTy` derivation is a checked trait impl, never const-value permission.
use super::*;
use crate::{support::bindings::LangItem, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const FIXTURES: &str = "pa31/tests/spec/const-parameter-derive/";
fn source(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{FIXTURES}{name}.rs"))?)
}
fn resolved(input: &str) -> TestResult<super::super::Report> {
    Ok(test_source::source_provider(
        input,
        "const_parameter_derive",
    )?)
}
fn expect_rejection(input: &str, message: &str) -> TestResult {
    let error = crate::pipeline::analyze_source(&resolved(input)?)
        .err()
        .ok_or("counterfeit source role accepted")?
        .to_string();
    assert!(error.contains(message), "expected {message}: {error}");
    Ok(())
}

#[test]
fn original_simd_alignment_keeps_four_real_members_and_actual_marker_equality() -> TestResult {
    let input = source("actual-source")?;
    let report = resolved(&input)?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual derive list")?
        .entries;
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.contexts.len(), 4);
    report.expanded_derive(entry.owner, entry.effect)?;
    assert!(entry.contexts.clone().any(|index| matches!(
        report.definitions[report.invocations[index].definition].body,
        Body::Builtin(Builtin::ConstParamTy)
    )));
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("actual source catalog")?
        .validated_language(LangItem::ConstParamTy)?
        .ok_or("actual ConstParamTy role")?;
    assert_eq!(
        report
            .package
            .names
            .get(report.resolution.definitions[declaration.definition.0 as usize].name),
        "ConstParamTy_"
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
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
    );
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/const-parameter-derive-v1/candidate/native");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn parameter_member_incarnation_and_same_spelling_traits_do_not_confer_roles() -> TestResult {
    let input = source("actual-source")?;
    let mut report = resolved(&input)?;
    let entry = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual transaction")?
        .entries[0];
    let (owner, effect) = (entry.owner, entry.effect);
    let member = entry
        .contexts
        .clone()
        .find(|&index| {
            matches!(
                report.definitions[report.invocations[index].definition].body,
                Body::Builtin(Builtin::ConstParamTy)
            )
        })
        .ok_or("actual selected ConstParamTy")?;
    let definition = report.invocations[member].definition;
    let before = report
        .package
        .source_core()
        .ok_or("source payer")?
        .macros
        .usage();
    report.definitions[definition].body = Body::Builtin(Builtin::Copy);
    assert!(report.expanded_derive(owner, effect).is_err());
    let rejected = report
        .package
        .source_core()
        .ok_or("source payer")?
        .macros
        .usage();
    assert!(rejected.0 > before.0);
    report.definitions[definition].body = Body::Builtin(Builtin::ConstParamTy);
    report.expanded_derive(owner, effect)?;
    let warm = report
        .package
        .source_core()
        .ok_or("source payer")?
        .macros
        .usage();
    assert!(warm.0 > rejected.0);
    assert_eq!(warm.1, before.1);
    let missing = input.replace("#[lang = \"const_param_ty\"]", "");
    assert_ne!(missing, input);
    expect_rejection(&missing, "requires its actual language trait")?;
    // Redirect the real path while keeping a distinct genuine language declaration.
    let wrong = input.replace("pub trait ConstParamTy_: StructuralPartialEq + Eq {}",
        "pub trait Genuine: StructuralPartialEq + Eq {} pub trait ConstParamTy_: StructuralPartialEq + Eq {}");
    assert_ne!(wrong, input);
    expect_rejection(&wrong, "different actual trait")?;
    let wrong = input.replace(
        "pub trait ConstParamTy_: StructuralPartialEq + Eq {}",
        "pub trait Wrong {} pub trait ConstParamTy_: StructuralPartialEq + Wrong {}",
    );
    expect_rejection(&wrong, "exact StructuralPartialEq and Eq")?;
    Ok(())
}

#[test]
fn missing_real_supertraits_and_unavailable_macro_members_remain_errors() -> TestResult {
    for name in [
        "missing-eq",
        "missing-structural",
        "unknown-member",
        "not-builtin",
    ] {
        let result = resolved(&source(name)?).and_then(|report| {
            crate::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(Into::into)
        });
        let error = result
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(!error.contains("implied and renamed"), "{name}: {error}");
        eprintln!("ConstParamTy opponent {name}: {error}");
    }
    let report = resolved(&source("same-spelling-macro")?)?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual derive selection")?
        .entries;
    assert_eq!(entries.len(), 1);
    assert!(entries[0].contexts.clone().any(|index| matches!(
        report.definitions[report.invocations[index].definition].body,
        Body::Builtin(Builtin::ConstParamTy)
    )));
    crate::pipeline::analyze_source(&report)?;
    Ok(())
}

#[test]
fn generic_markers_and_nominal_parameters_preserve_actual_proofs_and_cold_roles() -> TestResult {
    for name in ["wider-generic", "nominal-const"] {
        let result = resolved(&source(name)?).and_then(|report| {
            crate::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(Into::into)
        });
        result.map_err(|error| format!("valid Rust {name}: {error}"))?;
    }
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(LangItem::ConstParamTy.index(), 65);
    assert_eq!(LangItem::OffsetOf.index(), 64);
    // Appended wrapper roles extend cold tables; hot records stay fixed.
    assert_eq!(size_of::<crate::resolve::bindings::Reservations>(), 5208);
    assert_eq!(size_of::<crate::sema::compiler_items::Catalog>(), 3760);
    eprintln!(
        "source ConstParamTy cold Reservations={} Catalog={} Transaction={} roles={}",
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<Transaction>(),
        LangItem::ALL.len()
    );
    Ok(())
}

#[test]
fn derived_payloads_require_the_actual_marker_on_each_instantiated_child() -> TestResult {
    let input = source("wider-generic")?
        + "\nfn marker_roundtrip<T:crate::marker::ConstParamTy_>(value:T)->T{value}\n";
    let positive = input.clone()
        + "fn check_payload(value:Payload<SimdAlign>)->Payload<SimdAlign>{marker_roundtrip(value)}\n";
    crate::pipeline::analyze_source(&resolved(&positive)?)?;
    let negative = input
        + "struct Unkeyed;fn check_payload(value:Payload<Unkeyed>)->Payload<Unkeyed>{marker_roundtrip(value)}\n";
    let error = crate::pipeline::analyze_source(&resolved(&negative)?)
        .err()
        .ok_or("derived payload bypassed its actual child marker requirement")?
        .to_string();
    assert!(
        error.contains("can't be used as a const parameter type"),
        "{error}"
    );
    assert!(!error.contains("budget"), "{error}");
    Ok(())
}

#[test]
fn another_builtin_derive_cannot_assert_coerce_pointee_validity() -> TestResult {
    let mut report = resolved(&source("actual-source")?)?;
    let sites = report
        .resolution
        .definitions
        .iter()
        .filter_map(|definition| definition.node)
        .collect::<Vec<_>>();
    let mut selected = None;
    for site in sites {
        if report.derived_const_parameter_impl(site)? {
            selected = Some(site);
            break;
        }
    }
    let site = selected.ok_or("actual generated ConstParamTy implementation")?;
    assert!(!report.derived_coerce_pointee_impl(site)?);
    let span = report
        .package
        .unit(site.file)
        .configured
        .ast
        .node(site.node)
        .span;
    let origin = report
        .package
        .origin(site.file, span.start)
        .ok_or("derived origin")?;
    let index = usize::try_from(
        origin
            .introduction
            .0
            .checked_sub(1)
            .ok_or("actual derive introduction")?,
    )?;
    let definition = report
        .invocations
        .get(index)
        .ok_or("actual derive member")?
        .definition;
    assert!(matches!(
        report.definitions[definition].body,
        Body::Builtin(Builtin::ConstParamTy)
    ));
    report.definitions[definition].body = Body::Builtin(Builtin::CoercePointee);
    assert!(report.derived_coerce_pointee_impl(site).is_err());
    report.definitions[definition].body = Body::Builtin(Builtin::ConstParamTy);
    assert!(report.derived_const_parameter_impl(site)?);
    Ok(())
}
