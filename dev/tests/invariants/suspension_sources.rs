//! A suspension advisory never overrides ownership, drop or object evidence.
use super::*;
const CLUSTER: &str = "source-suspend-advisory";
#[test]
fn nominal_and_trait_words_reasons_and_macro_envelopes_preserve_real_bodies() -> TestResult {
    for name in [
        "structure",
        "enumeration",
        "union",
        "trait",
        "reason",
        "raw-reason",
        "empty-reason",
        "generic",
        "const-allowed",
        "mutation",
        "drop-body",
        "deny-unused",
        "cfg-removed",
        "macro-meta-allowance",
        "owner-replay",
    ] {
        let (report, analysis) = checked(CLUSTER, name)?;
        if name == "const-allowed" {
            assert_v(&report, &analysis.facts)?;
        }
        if name == "macro-meta-allowance" {
            assert!(!report.package.requested_must_not_suspend()?);
        }
        lower((report, analysis)).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn own_feature_word_or_string_and_exact_target_and_duplicate_rules_are_authoritative() -> TestResult
{
    for name in [
        "list",
        "empty-list",
        "non-string",
        "byte-string",
        "duplicate",
        "different-reasons",
        "free",
        "method",
        "type-alias",
        "constant",
        "variant",
        "field",
        "impl-target",
        "unsafe-wrapper",
        "missing-gate",
        "wrong-feature",
        "macro-item-input-missing-gate",
        "macro-without-allowance",
        "trait-method",
        "foreign-type",
    ] {
        let error = checked(CLUSTER, name)
            .err()
            .ok_or_else(|| format!("{name}: invalid suspension advisory accepted"))?;
        let text = error.to_string();
        assert!(
            text.contains("must_not_suspend") || text.contains("advisory"),
            "{name}: {text}"
        );
    }
    Ok(())
}
#[test]
fn marked_types_keep_shared_mutation_move_drop_copy_union_and_send_restrictions() -> TestResult {
    for (name, words) in [
        (
            "mutation-denied",
            &["mutable", "writable", "assign", "mutation"][..],
        ),
        ("drop-move-denied", &["move", "initialized", "moved"][..]),
        ("drop-copy-denied", &["Copy", "copy", "Drop", "drop"][..]),
        ("unsafe-denied", &["unsafe"][..]),
        (
            "auto-marker-denied",
            &["Send", "marker", "coerc", "incompatible", "trait"][..],
        ),
    ] {
        let error = checked(CLUSTER, name)
            .and_then(lower)
            .err()
            .ok_or_else(|| format!("{name}: advisory granted permission"))?;
        let text = error.to_string();
        assert!(
            !text.contains("advisory requires") && !text.contains("requires its must_not_suspend"),
            "{name}: {text}"
        );
        assert!(
            words.iter().any(|word| text.contains(word)),
            "{name}: {text}"
        );
    }
    Ok(())
}
