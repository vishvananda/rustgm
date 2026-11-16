//! `#[unstable_feature_bound(feature)]` adds an `UnstableFeature(feature)`
//! clause to its owner whatever the owner's availability, as alloc's
//! `#[unstable_feature_bound(transmutability)]` impl of its private
//! `Recyclable` trait shows; only a `#[stable]` owner is rejected. A staged
//! crate meets the clause only through the same bound in the user's
//! environment, never through its own `#![feature]` request.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --cap-lints=allow`). It accepts `private-impl-bound` and
//! `private-impl-bound-caller`. It rejects `private-impl-unbound-use` and
//! `private-impl-enabled-use` with "unstable feature `fixture_probe` is used
//! without being enabled" and `stable-owner-bound` and `stable-impl-bound`
//! with "item annotated with `#[unstable_feature_bound]` should not be
//! stable". `stable-impl-bound` was added, with its rejection declared, after
//! the first run showed a second owner rule for impls. The declaration for
//! `private-impl-enabled-use` was acceptance; rustc rejects it because a
//! staged crate's own feature request never meets a feature bound, and the
//! case now declares that rejection. `RustGM`'s messages for the two uses
//! and for `stable-impl-bound` were pinned after their first run.
//!
//! A trait with the bound, as core's `TransmuteFrom<Src, const ASSUME>`,
//! demands the clause wherever a where-clause names it; alloc's `Recyclable`
//! impl carries the same bound for its `TransmuteFrom` clauses. The four
//! `gated-*` cases were declared later, before either compiler ran: rustc
//! accepts the plain and quantified clause in an impl with the bound and
//! rejects both without it ("unstable feature `fixture_probe` is used
//! without being enabled").
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const ROOT: &str = "#![feature(staged_api, rustc_attrs)]\n#![stable(feature = \"fixture_root\", since = \"1.0.0\")]\n";
const ENABLED: &str = "#![feature(staged_api, rustc_attrs, fixture_probe)]\n#![stable(feature = \"fixture_root\", since = \"1.0.0\")]\n#[unstable(feature = \"fixture_probe\", issue = \"none\")]\npub fn probe() {}\n";
const TRAIT: &str =
    "trait Recyclable {}\n#[unstable_feature_bound(fixture_probe)]\nimpl<T> Recyclable for T {}\n";
const NEEDS: &str = "fn needs<T: Recyclable>() {}\n";
const CALLER: &str = "fn caller() {\n    needs::<u8>();\n}\n";
const UNMET: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
const GATE: &str = "#[unstable_feature_bound(fixture_probe)]\ntrait Gate<T: ?Sized, const SAFE: bool = false> {}\ntrait Recyclable<F> {}\n";
const BOUND: &str = "#[unstable_feature_bound(fixture_probe)]\n";
const GATED: &str = "impl<F, T> Recyclable<F> for T where T: Gate<F> {}\n";
const QUANTIFIED: &str =
    "impl<F, T> Recyclable<F> for T\nwhere\n    for<'a> &'a T: Gate<&'a F, { true }>,\n{\n}\n";
const UNGATED: &str = "trait argument does not satisfy its declaration requirements";

/// One ordinary crate and its rejection, or `None` when it is accepted.
struct Case {
    name: &'static str,
    items: &'static [&'static str],
    rejected: Option<&'static str>,
}
const CASES: [Case; 10] = [
    Case {
        name: "private-impl-bound",
        items: &[ROOT, TRAIT],
        rejected: None,
    },
    Case {
        name: "private-impl-unbound-use",
        items: &[ROOT, TRAIT, NEEDS, CALLER],
        rejected: Some(UNMET),
    },
    Case {
        name: "private-impl-enabled-use",
        items: &[ENABLED, TRAIT, NEEDS, CALLER],
        rejected: Some(UNMET),
    },
    Case {
        name: "private-impl-bound-caller",
        items: &[
            ROOT,
            TRAIT,
            NEEDS,
            "#[unstable_feature_bound(fixture_probe)]\n",
            CALLER,
        ],
        rejected: None,
    },
    Case {
        name: "stable-owner-bound",
        items: &[
            ROOT,
            "#[stable(feature = \"fixture_root\", since = \"1.0.0\")]\n#[unstable_feature_bound(fixture_probe)]\npub fn bound() {}\n",
        ],
        rejected: Some("unstable_feature_bound requires an actual regular unstable owner"),
    },
    Case {
        name: "stable-impl-bound",
        items: &[
            ROOT,
            "#[stable(feature = \"fixture_root\", since = \"1.0.0\")]\npub trait Shared {}\n#[stable(feature = \"fixture_root\", since = \"1.0.0\")]\npub struct Value;\n#[stable(feature = \"fixture_root\", since = \"1.0.0\")]\n#[unstable_feature_bound(fixture_probe)]\nimpl Shared for Value {}\n",
        ],
        rejected: Some(STABLE_IMPL),
    },
    Case {
        name: "gated-bound-in-bound-impl",
        items: &[ROOT, GATE, BOUND, GATED],
        rejected: None,
    },
    Case {
        name: "gated-quantified-bound-in-bound-impl",
        items: &[ROOT, GATE, BOUND, QUANTIFIED],
        rejected: None,
    },
    Case {
        name: "gated-bound-in-unbound-impl",
        items: &[ROOT, GATE, GATED],
        rejected: Some(UNGATED),
    },
    Case {
        name: "gated-quantified-bound-in-unbound-impl",
        items: &[ROOT, GATE, QUANTIFIED],
        rejected: Some(UNGATED),
    },
];
const STABLE_IMPL: &str = "unstable_feature_bound requires an actual regular unstable owner";

fn rejects(stderr: &str, message: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == message)
    })
}

/// A feature bound binds an owner without availability, and its clause is
/// met only by the same bound where it is used.
#[test]
fn feature_bounds_need_no_owner_availability() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/unannotated-bounds")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for case in CASES {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(&path, case.items.concat())?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
            .arg(&path)
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let matches = match case.rejected {
            None => output.status.success(),
            Some(message) => !output.status.success() && rejects(&stderr, message),
        };
        if !matches {
            failures.push(format!(
                "{}: success={} {stderr}",
                case.name,
                output.status.success()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
