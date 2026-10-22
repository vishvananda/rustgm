//! Checked unsizing already supplies metadata; final DST fields retain it.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
#[path = "dst_tail_metadata/work.rs"]
mod work;
struct Case {
    name: &'static str,
    path: &'static str,
    rust: i32,
    frontend: [i32; 4],
    reason: &'static str,
    cap_lints: bool,
}
const CASES: &[Case] = &[
    Case {
        name: "early-nominal-value",
        path: "pa25/tests/spec/dst-tail-metadata/early-nominal-value.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "late-nominal-value",
        path: "pa25/tests/spec/dst-tail-metadata/late-nominal-value.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "runtime-owner",
        path: "pa25/tests/spec/dst-tail-metadata/runtime-owner.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "direct-slice",
        path: "pa25/tests/spec/dst-tail-metadata/direct-slice.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "explicit-reference",
        path: "pa25/tests/spec/dst-tail-metadata/explicit-reference.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "nested-tail",
        path: "pa25/tests/spec/dst-tail-metadata/nested-tail.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "mutable-tail",
        path: "pa25/tests/spec/dst-tail-metadata/mutable-tail.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "empty-tail",
        path: "pa25/tests/spec/dst-tail-metadata/empty-tail.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "published-tail",
        path: "pa25/tests/spec/dst-tail-metadata/published-tail.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: true,
    },
    Case {
        name: "wrong-element",
        path: "pa25/tests/spec/dst-tail-metadata/wrong-element.rs",
        rust: 1,
        frontend: [1, 1, 1, 1],
        reason: "incompatible types",
        cap_lints: false,
    },
    Case {
        name: "shared-to-mutable",
        path: "pa25/tests/spec/dst-tail-metadata/shared-to-mutable.rs",
        rust: 1,
        frontend: [1, 1, 1, 1],
        reason: "cannot coerce a shared reference to a mutable reference",
        cap_lints: false,
    },
    Case {
        name: "local-escape",
        path: "pa25/tests/spec/dst-tail-metadata/local-escape.rs",
        rust: 1,
        frontend: [0, 1, 1, 0],
        reason: "required lifetime does not outlive its target",
        cap_lints: false,
    },
    Case {
        name: "prefix-then-tail",
        path: "pa25/tests/spec/dst-tail-metadata/prefix-then-tail.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: false,
    },
    Case {
        name: "prefix-only",
        path: "pa25/tests/spec/dst-tail-metadata/prefix-only.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: false,
    },
    Case {
        name: "tuple-holder",
        path: "pa25/tests/spec/dst-tail-metadata/tuple-holder.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: false,
    },
    Case {
        name: "unsized-tuple-coercion",
        path: "pa25/tests/spec/dst-tail-metadata/unsized-tuple-coercion.rs",
        rust: 1,
        frontend: [1, 1, 1, 1],
        reason: "incompatible types",
        cap_lints: false,
    },
    Case {
        name: "tail-bounds",
        path: "pa25/tests/spec/dst-tail-metadata/tail-bounds.rs",
        rust: 1,
        frontend: [1, 1, 1, 1],
        reason: "constant index is out of bounds",
        cap_lints: false,
    },
    Case {
        name: "published-field-view",
        path: "pa25/tests/spec/dst-tail-metadata/published-field-view.rs",
        rust: 0,
        frontend: [0, 0, 0, 0],
        reason: "",
        cap_lints: false,
    },
    Case {
        name: "packed-reference",
        path: "pa27/tests/spec/dst-tail-metadata/packed-reference.rs",
        rust: 1,
        frontend: [0, 1, 1, 0],
        reason: "reference to an unaligned packed field",
        cap_lints: false,
    },
];
#[test]
fn dst_field_metadata_matches_declared_source_and_phase_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let evidence = root.join(format!(
        "obj/parallel/pa25/dst-tail-metadata/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &Case {
        name,
        path,
        rust,
        frontend,
        reason,
        cap_lints,
    } in CASES
    {
        let modes = [
            ("--emit-types", frontend[0]),
            ("--emit-lowir", frontend[1]),
            ("--emit-constants", frontend[2]),
            ("--emit-body-ir", frontend[3]),
        ];
        let args = if cap_lints {
            &["--cap-lints=allow"][..]
        } else {
            &[]
        };
        checked::check_views_with_rustc_args(
            &root.join(path),
            &evidence,
            name,
            rust,
            &modes,
            reason,
            args,
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn dst_metadata_native_values_and_published_relocations_match_rust()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let paths = &[
        (
            "early-nominal-value",
            "pa25/tests/spec/dst-tail-metadata/early-nominal-value.rs",
        ),
        (
            "late-nominal-value",
            "pa25/tests/spec/dst-tail-metadata/late-nominal-value.rs",
        ),
        (
            "runtime-owner",
            "pa25/tests/spec/dst-tail-metadata/runtime-owner.rs",
        ),
        (
            "direct-slice",
            "pa25/tests/spec/dst-tail-metadata/direct-slice.rs",
        ),
        (
            "explicit-reference",
            "pa25/tests/spec/dst-tail-metadata/explicit-reference.rs",
        ),
        (
            "nested-tail",
            "pa25/tests/spec/dst-tail-metadata/nested-tail.rs",
        ),
        (
            "mutable-tail",
            "pa25/tests/spec/dst-tail-metadata/mutable-tail.rs",
        ),
        (
            "empty-tail",
            "pa25/tests/spec/dst-tail-metadata/empty-tail.rs",
        ),
        (
            "published-tail",
            "pa25/tests/spec/dst-tail-metadata/published-tail.rs",
        ),
        (
            "prefix-then-tail",
            "pa25/tests/spec/dst-tail-metadata/prefix-then-tail.rs",
        ),
        (
            "prefix-only",
            "pa25/tests/spec/dst-tail-metadata/prefix-only.rs",
        ),
        (
            "tuple-holder",
            "pa25/tests/spec/dst-tail-metadata/tuple-holder.rs",
        ),
        (
            "published-field-view",
            "pa25/tests/spec/dst-tail-metadata/published-field-view.rs",
        ),
        (
            "published-tail-runtime",
            "pa25/tests/dst-tail-native/published-tail-runtime.rs",
        ),
        (
            "published-field-runtime",
            "pa25/tests/dst-tail-native/published-field-runtime.rs",
        ),
        (
            "mutable-runtime",
            "pa25/tests/dst-tail-native/mutable-runtime.rs",
        ),
        (
            "prefix-runtime",
            "pa25/tests/dst-tail-native/prefix-runtime.rs",
        ),
        (
            "nested-runtime",
            "pa25/tests/dst-tail-native/nested-runtime.rs",
        ),
        (
            "nested-field-runtime",
            "pa25/tests/dst-tail-native/nested-field-runtime.rs",
        ),
    ];
    let mut inputs = Vec::new();
    for &(name, path) in paths {
        inputs.push((name, std::fs::read_to_string(root.join(path))?));
    }
    let cases = inputs
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_literal_cases("parallel/pa25/dst-tail-metadata/native", &cases)
}
