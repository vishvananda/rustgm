//! An always-const premise proves a `[const]` requirement, as rustc's host
//! effect `satisfies` does: stdarch's `Simd` compares `[T; N]` under
//! `T: SimdElement`, whose supertrait is `const PartialEq`.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/always-const-premises";
const UNSTABLE: &str = "#[rustc_const_unstable(feature = \"simd_eq_fixture\", issue = \"none\")]\n";
const SCALAR: &str = "pub const fn f<T: SimdElement>(a: &T, b: &T) -> bool {\n    *a == *b\n}\n";
const UNMATCHED: &str = "trait obligation requires an unambiguous matching assumption";
/// rustc reports E0277 for the first four and E0284 for the ambiguous pair.
const REJECTED: [(&str, &str); 5] = [
    ("plain-supertrait.rs", UNMATCHED),
    ("maybe-plain-premise.rs", UNMATCHED),
    ("const-goal-maybe-premise.rs", "can't compare `T` with `T`"),
    (
        "const-impl-maybe-premise.rs",
        "can't compare `W<T>` with `W<T>`",
    ),
    (
        "both-premises.rs",
        "ambiguous trait assumptions in declaration",
    ),
];
const MEMBER: &str = "const function member lacks its current conditional trait premise";
const RECEIVER: &str = "method borrow differs from selected receiver place";
/// Recorded prepared lowering gaps of accepted cases, which the base shares.
/// A const body's trait member call whose Self an impl proves, such as
/// `[T; N]: [const] PartialEq`, needs a direct environment premise. A
/// comparison of two `&T` operands disagrees on its receiver's region.
const LOWERING: [(&str, &str); 8] = [
    ("core-form.rs", MEMBER),
    ("direct-call.rs", MEMBER),
    ("element-fn.rs", MEMBER),
    ("maybe-supertrait.rs", MEMBER),
    ("const-impl-supertrait.rs", MEMBER),
    ("concrete-use.rs", MEMBER),
    ("scalar-refs-op.rs", RECEIVER),
    ("ordinary-fn-supertrait.rs", RECEIVER),
];

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "const_premises")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}

#[test]
fn always_const_premises_prove_conditional_requirements_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("const premise columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid const premise outcome".into()),
        };
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@const_premises",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "const_premises",
            ])
            .arg(root.join(path))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = REJECTED
            .iter()
            .find(|(name, _)| path.ends_with(&format!("/{name}")))
            .map(|(_, reason)| *reason);
        if output.status.code() != Some(i32::from(!accepted))
            || accepted != reason.is_none()
            || reason.is_some_and(|reason| !stderr.contains(reason))
        {
            failures.push(format!("{path}: {:?} {stderr}", output.status.code()));
        }
        if accepted {
            let gap = LOWERING
                .iter()
                .find(|(name, _)| path.ends_with(&format!("/{name}")))
                .map(|(_, reason)| *reason);
            let lowered = program(&crate::resolve::fixture(path)?)
                .err()
                .map(|error| error.to_string());
            let expected = match (&lowered, gap) {
                (None, None) => true,
                (Some(error), Some(reason)) => error.contains(reason),
                _ => false,
            };
            if !expected {
                failures.push(format!("{path}: lowering: {lowered:?}"));
            }
        }
        count += 1;
    }
    assert_eq!(count, 18);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` const functions, each comparing through the supertrait.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "{UNSTABLE}pub const fn f{index}<T: SimdElement>(a: &T, b: &T) -> bool {{ *a == *b }}"
        )?;
    }
    Ok(text)
}

/// Typing work stays linear, and every width completes prepared lowering.
#[test]
fn const_premise_work_tracks_functions() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/scalar-op.rs"))?;
    let prefix = source
        .strip_suffix(&format!("{UNSTABLE}{SCALAR}"))
        .ok_or("const premise suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "const_premises")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("const premises functions={functions} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "functions={functions}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        program(&text)?;
    }
    Ok(())
}
