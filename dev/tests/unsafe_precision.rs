//! Packed, union and raw-pointer boundaries retain full typed leaf accesses.
use crate::native_references as native;
use crate::semantic;
use std::{path::PathBuf, process::Command};
const CASES: &[(&str, i32)] = &[
    ("packed-aligned-local-borrow", 0),
    ("packed-unaligned-local-borrow", 1),
    ("raw-slot-and-pointee", 0),
    ("raw-slot-requires-mut-call", 1),
    ("packed-copy", 0),
    ("packed-aligned-conflict", 1),
    ("ordinary-disjoint", 0),
    ("packed-nested-disjoint", 0),
    ("packed-nested-conflict", 1),
    ("packed-noncopy-conflict", 1),
    ("ordinary-noncopy-disjoint", 0),
    ("packed-move-conflict", 1),
    ("packed-aligned-reference", 0),
    ("packed-unaligned-reference", 1),
    ("packed-raw-address", 0),
    ("packed-raw-mutable", 0),
    ("raw-read", 0),
    ("raw-write-shared-call", 0),
    ("raw-pointer-slot-conflict", 1),
    ("raw-nested-field", 0),
    ("raw-move-field", 0),
    ("raw-two-dereferences", 0),
    ("raw-cancellation", 0),
    ("raw-unsafe-required", 1),
    ("raw-const-write-denied", 1),
    ("union-copy-read", 0),
    ("union-use-after-capture", 0),
    ("union-write", 0),
    ("union-write-requires-mut-call", 1),
    ("union-nested-disjoint", 0),
    ("union-field-reference", 0),
    ("union-raw-address", 0),
    ("union-raw-unsafe-required", 0),
    ("nested-raw", 0),
    ("nested-packed", 0),
    ("nested-union", 0),
    ("union-consume-before-call", 1),
    ("union-consume-after-call", 0),
    ("union-move-consume", 1),
    ("union-raw-nested-unsafe", 1),
    ("union-raw-nested", 0),
    ("union-raw-mutable", 0),
    ("raw-address-through-pointer-shared-call", 0),
    ("packed-copy-shared-call", 0),
    ("raw-outer-reference", 0),
];

fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn source_precision_and_access_permissions() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let folder = root.join(format!(
        "obj/parallel/pa27/capture-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, expected) in CASES {
        let path = root.join(format!("pa27/tests/spec/capture-precision/{name}.rs"));
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--emit=obj",
                "-C",
                "panic=abort",
                "-o",
            ])
            .arg(folder.join(format!("{name}.o")))
            .arg(&path)
            .output()?;
        std::fs::write(
            folder.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(
            independent.status.code(),
            Some(expected),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        for mode in ["--emit-types", "--emit-lowir", "--emit-constants"] {
            let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            std::fs::write(folder.join(format!("{name}.{mode}.stdout")), &output.stdout)?;
            std::fs::write(folder.join(format!("{name}.{mode}.stderr")), &output.stderr)?;
            std::fs::write(
                folder.join(format!("{name}.{mode}.status")),
                output.status.code().ok_or("signal")?.to_string(),
            )?;
            let actual_expected = if mode == "--emit-types" {
                i32::from(matches!(
                    name,
                    "raw-unsafe-required" | "union-raw-nested-unsafe"
                ))
            } else {
                expected
            };
            if matches!(
                name,
                "packed-unaligned-local-borrow" | "packed-unaligned-reference"
            ) && mode != "--emit-types"
            {
                assert!(
                    String::from_utf8_lossy(&output.stderr)
                        .contains("reference to an unaligned packed field"),
                    "{name}/{mode}"
                );
            }
            if output.status.code() != Some(actual_expected) {
                failures.push(format!(
                    "{name}/{mode}: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires qualified external CPPGM backend"]
fn native_precision_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for name in [
        "packed",
        "packed-move",
        "raw-write",
        "raw-move",
        "raw-cancellation",
        "union",
        "nested",
        "shared-before-boundary",
        "pointer-slot",
    ] {
        let text = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/capture-native/{name}.rs")),
        )?;
        if let Err(e) = native::check_cases("parallel/pa27/capture-native", &[(name, &text, false)])
        {
            failures.push(format!("{name}: {e}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn capture_prefixes_retain_modes_and_full_leaf_capabilities()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        sema::closures::captures::{Mode, Step},
        types::{Kind, closure::CallKind},
    };
    for (name, mode, depth, raw, union) in [
        ("packed-copy", Mode::Shared, 0, false, false),
        ("packed-copy-shared-call", Mode::Value, 0, false, false),
        ("packed-nested-disjoint", Mode::Shared, 1, false, false),
        ("raw-read", Mode::Shared, 0, true, false),
        ("raw-write-shared-call", Mode::Shared, 0, true, false),
        ("raw-nested-field", Mode::Shared, 1, true, false),
        ("raw-move-field", Mode::Value, 1, true, false),
        ("raw-outer-reference", Mode::Shared, 1, true, false),
        ("union-copy-read", Mode::Shared, 0, false, true),
        ("union-nested-disjoint", Mode::Shared, 1, false, true),
    ] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/capture-precision/{name}.rs")),
        )?;
        let (_, facts) = semantic::analyze(&source).map_err(|e| format!("{name}: {e}"))?;
        let (body, env) = facts
            .closures
            .all()
            .iter()
            .find_map(|body| {
                facts
                    .closures
                    .environment(body.identity)
                    .filter(|e| !e.plan.captures().is_empty())
                    .map(|e| (body, e))
            })
            .ok_or("missing environment")?;
        assert_eq!(
            facts.types.closure_kind(body.identity),
            Some(CallKind::Shared),
            "{name}"
        );
        assert_eq!(env.plan.captures().len(), 1, "{name}");
        let capture = env.plan.captures()[0];
        assert_eq!(capture.mode, mode, "{name}");
        let mut cursor = capture.path;
        let mut actual = 0;
        while let Some((parent, step)) = env.plan.parent(cursor) {
            assert!(
                !matches!(step, Step::RawDereference { .. } | Step::UnionField(_)),
                "{name}"
            );
            actual += 1;
            cursor = parent;
        }
        assert_eq!(actual, depth, "{name}");
        let mut raw_steps = 0;
        let mut union_steps = 0;
        for index in env.accesses.values() {
            let mut cursor = *index;
            while let Some((parent, step)) = env.plan.parent(cursor) {
                match step {
                    Step::RawDereference { mutable } => {
                        raw_steps += 1;
                        assert!(
                            matches!(facts.types.kind(env.path_types[parent.index()]),Some(Kind::Pointer{mutable:actual,pointee}) if actual==mutable && pointee==env.path_types[cursor.index()])
                        );
                    }
                    Step::UnionField(_) => union_steps += 1,
                    _ => {}
                }
                cursor = parent;
            }
        }
        assert_eq!(raw_steps != 0, raw, "{name}");
        assert_eq!(union_steps != 0, union, "{name}");
    }
    Ok(())
}
#[test]
fn repeated_precision_boundaries_keep_linear_graph_work() -> Result<(), Box<dyn std::error::Error>>
{
    use std::fmt::Write;
    for nested in [false, true] {
        let mut work = Vec::new();
        for width in [8, 32, 128] {
            let mut source = String::from("#[repr(packed)]struct P{");
            for index in 0..width {
                write!(source, "f{index}:u32,")?;
            }
            source.push_str("}fn main(){let p=P{");
            for index in 0..width {
                write!(source, "f{index}:1,")?;
            }
            source.push_str("};let c=||{");
            if nested {
                source.push_str("let child=||{");
            }
            source.push('0');
            for index in 0..width {
                write!(source, "+p.f{index}")?;
            }
            if nested {
                source.push_str("};child()");
            }
            source.push_str("};let _=c();}");
            let (_, facts) = semantic::analyze(&source)?;
            let mut sum = 0;
            for env in facts
                .closures
                .all()
                .iter()
                .filter_map(|body| facts.closures.environment(body.identity))
            {
                assert_eq!(env.plan.captures().len(), 1);
                assert!(env.plan.parent(env.plan.captures()[0].path).is_none());
                sum += env.plan.work;
            }
            assert!(sum < 16 * (width + 1));
            assert!(facts.body_work.canonical_steps < 2048 * (width + 1));
            work.push(sum);
        }
        assert_eq!(work[2] - work[1], 4 * (work[1] - work[0]));
    }
    Ok(())
}

#[test]
#[ignore = "requires qualified external CPPGM backend"]
fn combined_reference_outputs_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["packed-reference", "union-reference"] {
        let source = std::fs::read_to_string(root()?.join(format!(
            "pa27/tests/spec/capture-integration-native/{name}.rs"
        )))?;
        native::check_cases(
            "parallel/pa27/capture-reference-native",
            &[(name, &source, false)],
        )?;
    }
    Ok(())
}
