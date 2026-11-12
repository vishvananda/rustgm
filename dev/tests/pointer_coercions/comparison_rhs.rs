//! A source comparison selects its `Rhs` before the right operand coerces, so
//! core's `*const T != *mut T` compares two `*const T`, as rustc does.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/comparison-rhs";
const REJECTED: [(&str, &str); 3] = [
    (
        "mut-eq-const.rs",
        "raw pointer coercion cannot strengthen mutability",
    ),
    ("premise-rhs.rs", "incompatible types"),
    ("ambiguous-user.rs", "incompatible types"),
];
const CALL: &str = "pub fn ne<T>(a: *const T, b: *mut T) -> bool { a != b }\n";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "comparison_rhs")?;
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
fn comparison_right_operands_coerce_to_selected_rhs_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("comparison columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid comparison outcome".into()),
        };
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@comparison_rhs",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "comparison_rhs",
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
        // A coerced raw pointer comparison stays a builtin operation.
        if accepted && path.ends_with("/const-ne-mut.rs") {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let erased = stdout.lines().any(|line| {
                line.contains(" binary BangEqual left=")
                    && line.split_once("left=").is_some_and(|(_, types)| {
                        types.split_once(" right=").is_some_and(|(a, b)| a == b)
                    })
            });
            if !erased {
                failures.push(format!("{path}: comparison is not a builtin operation"));
            }
        }
        if accepted && let Err(error) = program(&crate::resolve::fixture(path)?) {
            failures.push(format!("{path}: lowering: {error}"));
        }
        count += 1;
    }
    assert_eq!(count, 12);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Node identities are file-local. `a.rs` coerces `*const u8 != *mut u8` and
/// `b.rs` compares `u8 != u8` at the same node, after it. A coerced operand
/// recorded for one body must not retype another body's builtin comparison.
#[test]
fn coerced_operands_stay_with_their_body_across_files() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-types",
            "--edition=2024",
            "--cap-lints=allow",
            "--source-core",
            "minicore@comparison_rhs",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "comparison_rhs",
        ])
        .arg(root.join(PACKET).join("stale-node/lib.rs"))
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut erased = Vec::new();
    for line in stdout.lines() {
        let Some((site, types)) = line
            .strip_prefix("operation ")
            .and_then(|line| line.split_once(" binary BangEqual left="))
        else {
            continue;
        };
        let (left, right) = types.split_once(" right=").ok_or("comparison types")?;
        let (file, node) = site.split_once(':').ok_or("comparison site")?;
        erased.push((
            file.to_owned(),
            node.to_owned(),
            left == right,
            left.to_owned(),
        ));
    }
    let [(first, a, true, pointer), (second, b, true, byte)] = erased.as_slice() else {
        return Err(format!("expected two erased comparisons: {erased:?}").into());
    };
    assert_eq!((first.as_str(), second.as_str()), ("1", "2"));
    assert_eq!(a, b, "the fixture must share one node identity");
    assert_ne!(pointer, byte);
    Ok(())
}

/// `calls` coerced comparisons, then unrelated nominals whose own
/// `PartialEq` impls no comparison selects.
fn width(prefix: &str, calls: usize, noise: bool) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "pub fn c{index}<T>(a: *const T, b: *mut T) -> bool {{ a != b }}"
        )?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "pub struct U{index}; impl PartialEq for U{index} {{ fn eq(&self, other: &U{index}) -> bool {{ true }} }}"
        )?;
    }
    Ok(text)
}

#[test]
fn coerced_comparisons_track_calls_without_scanning_unrelated_impls() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/const-ne-mut.rs"))?;
    let prefix = source.strip_suffix(CALL).ok_or("coerced comparison")?;
    let mut previous = None;
    let mut unrelated = None;
    for calls in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let report =
                crate::resolve::source_provider(&width(prefix, calls, noise)?, "comparison_rhs")?;
            let facts = rustgm::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            println!("coerced comparisons calls={calls} noise={noise} work={work:?}");
            if noise {
                // The unrelated impls are validated once; no comparison
                // selects them.
                let added: Vec<_> = std::iter::zip(ordinary, work)
                    .map(|(old, actual)| actual.checked_sub(old))
                    .collect();
                assert_eq!(*unrelated.get_or_insert_with(|| added.clone()), added);
            } else {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(
                            actual <= 4 * old + 512,
                            "calls={calls}: {before:?}->{work:?}"
                        );
                    }
                }
                previous = Some(work);
                ordinary = work;
            }
        }
    }
    Ok(())
}
