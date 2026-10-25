//! Must-use domain and ancestry controls retain every observed reason/count disagreement.
use std::path::Path;
type TestResult = Result<(), Box<dyn std::error::Error>>;
struct Case {
    path: &'static str,
    warnings: usize,
    messages: &'static [&'static str],
}
const CASES: &[Case] = &[
    Case {
        path: "pa20/tests/spec/discard-domains/projection-published-bound.rs",
        warnings: 1,
        messages: &["unused must_use result: BOUND"],
    },
    Case {
        path: "pa20/tests/spec/discard-domains/projection-explicit-let.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa20/tests/spec/discard-domains/projection-normalized-concrete.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa20/tests/spec/discard-domains/projection-unmarked-bound.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa18/tests/spec/discard-domains/array-unknown-length.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa18/tests/spec/discard-domains/array-template-is-not-relinted.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa10/tests/spec/discard-domains/unit-dereference.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/unit-tuple-dereference.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/unit-array-dereference.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa10/tests/spec/discard-domains/unit-borrow-still-must-use.rs",
        warnings: 1,
        messages: &["unused borrow that must be used"],
    },
    Case {
        path: "pa10/tests/spec/discard-domains/unit-marked-function-still-must-use.rs",
        warnings: 1,
        messages: &["unused must_use result"],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/marked-empty-enum.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/marked-public-empty-field.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/marked-private-empty-field-caller.rs",
        warnings: 1,
        messages: &["unused must_use result"],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/marked-private-empty-field-owner.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa18/tests/spec/discard-domains/marked-generic-empty-field.rs",
        warnings: 0,
        messages: &[],
    },
    Case {
        path: "pa21/tests/spec/discard-domains/opaque-left-then-right.rs",
        warnings: 1,
        messages: &["unused must_use result: RIGHT"],
    },
    Case {
        path: "pa21/tests/spec/discard-domains/opaque-right-then-left.rs",
        warnings: 1,
        messages: &["unused must_use result: LEFT"],
    },
    Case {
        path: "pa21/tests/spec/discard-domains/supertrait-left-then-right.rs",
        warnings: 1,
        messages: &["unused must_use result: RIGHT"],
    },
    Case {
        path: "pa21/tests/spec/discard-domains/supertrait-right-then-left.rs",
        warnings: 1,
        messages: &["unused must_use result: LEFT"],
    },
    Case {
        path: "pa21/tests/spec/discard-domains/direct-trait-before-ancestors.rs",
        warnings: 1,
        messages: &["unused must_use result: DIRECT"],
    },
    Case {
        path: "pa21/tests/spec/discard-domains/deep-left-shallow-right.rs",
        warnings: 1,
        messages: &["unused must_use result: SHALLOW"],
    },
    Case {
        path: "pa21/tests/spec/discard-domains/diamond-one-obligation.rs",
        warnings: 1,
        messages: &["unused must_use result: SHARED"],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/tuple-repeated-marked-type.rs",
        warnings: 2,
        messages: &[
            "unused must_use result: ELEMENT",
            "unused must_use result: ELEMENT",
        ],
    },
    Case {
        path: "pa24/tests/spec/discard-domains/tuple-two-closures.rs",
        warnings: 2,
        messages: &[
            "unused closure that must be used",
            "unused closure that must be used",
        ],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/function-and-result-both-marked.rs",
        warnings: 2,
        messages: &[
            "unused must_use result: TYPE",
            "unused must_use result: FUNCTION",
        ],
    },
    Case {
        path: "pa12/tests/spec/discard-domains/marked-array-one-warning.rs",
        warnings: 1,
        messages: &["unused must_use result: ELEMENT"],
    },
];
#[test]
fn discarded_domains_and_reverse_declared_trait_ancestry_match_independent_observations()
-> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repository root")?;
    for case in CASES {
        let source = std::fs::read_to_string(root.join(case.path))?;
        let (_, facts) = crate::attribute_contracts::analyze(&source)
            .map_err(|error| format!("{}: {error}", case.path))?;
        let warnings: Vec<_> = facts.warnings().collect();
        assert_eq!(warnings.len(), case.warnings, "{}", case.path);
        assert!(
            warnings
                .iter()
                .all(|warning| warning.code.name() == "unused_must_use"),
            "{}",
            case.path
        );
        assert_eq!(case.messages.len(), case.warnings, "{}", case.path);
        for (warning, message) in warnings.iter().zip(case.messages) {
            assert_eq!(warning.diagnostic.message, *message, "{}", case.path);
        }
    }
    Ok(())
}
