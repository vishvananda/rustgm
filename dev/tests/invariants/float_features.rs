//! Actual configuration, primitive selection and literal/capture provenance.
use crate::test_source;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn checked(source: &str) -> Result<(), String> {
    let report = test_source::resolve(source, true)?;
    crate::pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn active_wide_features_and_inactive_token_opponents_use_real_authority() -> TestResult {
    for name in [
        "active-f128-pi.rs",
        "active-f16.rs",
        "integer-spelling.rs",
        "disabled-owner.rs",
        "unexpanded-tokens.rs",
    ] {
        checked(&test_source::fixture(&format!(
            "pa31/tests/spec/wide-float-source/{name}"
        ))?)?;
    }
    checked("struct f128; fn ordinary(_:f128){}")?;
    for name in ["missing-feature.rs", "expanded-ungated.rs"] {
        let source = test_source::fixture(&format!("pa31/tests/spec/wide-float-source/{name}"))?;
        let error = checked(&source)
            .err()
            .ok_or("ungated wide float accepted")?;
        assert!(error.contains("wide floating type requires"), "{error}");
    }
    for name in ["invalid-suffix.rs", "nondecimal.rs", "mismatched-width.rs"] {
        let source = test_source::fixture(&format!("pa31/tests/spec/wide-float-source/{name}"))?;
        assert!(checked(&source).is_err(), "{name}");
    }
    let report = test_source::resolve("fn ordinary(){}", true)?;
    assert_eq!(
        report
            .resolution
            .definitions
            .iter()
            .filter(|d| d.kind == crate::resolve::model::DefKind::Primitive)
            .count(),
        19
    );
    assert!(!report.package.requested_wide_float(true));
    assert!(!report.package.requested_wide_float(false));
    let prefix = "#![feature(allow_internal_unstable)] #[allow_internal_unstable(f128)] ";
    checked(&format!(
        "{prefix} macro_rules! literal {{()=>{{1.0_f128}};}} fn f(){{let _=literal!();}}"
    ))?;
    let error = checked(&format!(
        "{prefix} macro_rules! captured {{($x:expr)=>{{$x}};}} fn f(){{let _=captured!(1.0_f128);}}"
    ))
    .err()
    .ok_or("captured token borrowed definition allowance")?;
    assert!(error.contains("wide floating type requires"), "{error}");
    Ok(())
}

#[test]
fn wide_values_keep_exact_shapes_and_unsupported_later_operations_closed() -> TestResult {
    checked("#![feature(f16,f128)] const A:f16=-0.0_f16; const B:f128=1.0_f128;")?;
    for (source, message) in [
        (
            "#![feature(f128)] const A:f128=1.0_f128+2.0_f128;",
            "wide floating constant arithmetic",
        ),
        (
            "#![feature(f128)] const A:f32=1.0_f128 as f32;",
            "wide floating constant casts",
        ),
        (
            "#![feature(f128)] const A:f128=1e99999_f128;",
            "floating literal is out of range",
        ),
    ] {
        let error = checked(source)
            .err()
            .ok_or("unsupported wide operation accepted")?;
        assert!(error.contains(message), "{error}");
    }
    assert!(crate::sema::patterns::scalar::Domain::new(crate::types::Primitive::F128).is_none());
    println!(
        "Features={} MacroFeatures={} Package={} Report={} Source={} SourceCore={}",
        size_of::<super::super::bindings::policy::Features>(),
        size_of::<super::super::bindings::policy::MacroFeatures>(),
        size_of::<super::super::package::Package>(),
        size_of::<super::super::expansion::Report>(),
        size_of::<super::super::bindings::Source>(),
        size_of::<super::super::bindings::SourceCore>()
    );
    Ok(())
}
