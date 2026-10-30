//! Confusable strings do not become successful method names or permissions.
use super::*;
const CLUSTER: &str = "source-confusable-methods";
#[test]
fn repeated_lists_and_real_inherent_methods_preserve_const_mutation_and_macro_authority()
-> TestResult {
    for name in [
        "inherent",
        "multiple-names",
        "repeat-attributes",
        "repeat-names",
        "empty-name",
        "raw-and-escaped",
        "nonidentifier-name",
        "associated-static",
        "const-allowed",
        "mutation",
        "unsafe-allowed",
        "deny-unused",
        "cfg-removed",
        "macro-meta-allowance",
        "owner-replay",
    ] {
        let (report, analysis) = checked(CLUSTER, name)?;
        if name == "const-allowed" {
            assert_v(&report, &analysis.facts)?;
        }
        lower((report, analysis)).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn forms_features_and_actual_noninherent_owners_reject_without_aliasing_names() -> TestResult {
    for name in [
        "word",
        "name-value",
        "empty-list",
        "non-string",
        "identifier",
        "byte-string",
        "unsafe-wrapper",
        "free",
        "trait-required",
        "trait-default",
        "trait-implementation",
        "foreign",
        "structure",
        "constant",
        "missing-gate",
        "wrong-feature",
        "macro-item-input-missing-gate",
        "macro-without-allowance",
        "closure",
        "nested-free",
    ] {
        let error = checked(CLUSTER, name)
            .err()
            .ok_or_else(|| format!("{name}: invalid confusables accepted"))?;
        let text = error.to_string();
        assert!(
            text.contains("confusables")
                || text.contains("advisory")
                || text.contains("rustc_attrs"),
            "{name}: {text}"
        );
    }
    Ok(())
}
#[test]
fn lookup_const_unsafe_and_shared_mutation_fail_at_their_real_consumers() -> TestResult {
    for (name, words) in [
        (
            "typo-is-not-an-alias",
            &["method", "applicable", "unresolved"][..],
        ),
        (
            "qualified-typo-is-not-an-alias",
            &[
                "method",
                "associated",
                "unresolved",
                "unknown",
                "missing value resolution",
            ][..],
        ),
        (
            "mutation-denied",
            &["mutable", "writable", "assign", "mutation"][..],
        ),
        ("unsafe-denied", &["unsafe"][..]),
        ("const-denied", &["const"][..]),
    ] {
        let error = checked(CLUSTER, name)
            .and_then(lower)
            .err()
            .ok_or_else(|| format!("{name}: invented permission"))?;
        let text = error.to_string();
        assert!(
            !text.contains("requires its actual") && !text.contains("requires its rustc_attrs"),
            "{name}: {text}"
        );
        assert!(
            words.iter().any(|word| text.contains(word)),
            "{name}: {text}"
        );
    }
    Ok(())
}
