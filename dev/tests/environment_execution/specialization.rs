//! Generic captured bodies retain complete instance identities and ordinary proofs.
use super::{checked_sources, native, semantic};
use std::{fmt::Write, path::PathBuf};

const CASES: &[(&str, &str)] = &[
    ("copy-shared", ""),
    ("move-owned", ""),
    ("mutable", ""),
    ("const-array", ""),
    ("returned-opaque", ""),
    ("borrowed-result", ""),
    ("shared-reference", ""),
    ("mutable-reference", ""),
    ("nested", ""),
    ("input-binder", ""),
    ("unused-arguments", ""),
    ("reference-type", ""),
    ("projected-capture", ""),
    ("cleanup", ""),
    (
        "move-twice",
        "local may be uninitialized or moved at this read",
    ),
    (
        "borrow-conflict",
        "access conflicts with a live reference loan",
    ),
    (
        "local-output",
        "required lifetime does not outlive its target",
    ),
    (
        "input-output-escape",
        "required lifetime does not outlive its target",
    ),
    ("fn-pointer-capture", "incompatible types"),
    (
        "shared-reference-escape",
        "required lifetime does not outlive its target",
    ),
    (
        "unused-invalid",
        "local may be uninitialized or moved at this read",
    ),
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/generic-captures/{name}.rs"))
}
#[test]
fn complete_generic_capture_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/generic-captures/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, reason) in CASES {
        let status = i32::from(!reason.is_empty());
        let types = i32::from(name == "fn-pointer-capture");
        checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (status, [types, status, status], reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn generic_captures_execute_and_clean_up() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, reason) in CASES {
        if reason.is_empty() {
            let source = std::fs::read_to_string(fixture(name))?;
            let source = source
                .strip_prefix(native::CHECK)
                .ok_or("missing check prefix")?;
            native::check_cases(
                "parallel/pa24/generic-captures/native",
                &[(name, source, false)],
            )?;
        }
    }
    Ok(())
}
fn measure(source: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let program = rustgm::lower::instances::build(&report, &mut facts, entry)?;
    let closures = program
        .symbols
        .iter()
        .filter(|s| s.name.starts_with("rg.closure.") && s.body.is_some())
        .count();
    Ok((facts.body_work.canonical_steps, closures))
}
#[test]
fn unused_type_and_const_arguments_remain_distinct() -> Result<(), Box<dyn std::error::Error>> {
    let source = "fn value<T,const N:usize>(x:u8)->u8{let f=move||x;f()}fn main(){let _=value::<u8,1>(7);let _=value::<u16,1>(8);let _=value::<u8,2>(9);let _=value::<u8,1>(10);}";
    let (_, closures) = measure(source)?;
    assert_eq!(closures, 3);
    Ok(())
}
#[test]
fn generic_capture_work_grows_with_demanded_bodies() -> Result<(), Box<dyn std::error::Error>> {
    for family in ["ordinary", "repeated", "independent"] {
        let mut observations = Vec::new();
        for width in [8, 32, 128] {
            let mut source = "fn take<T:Copy>(x:T)->T{".to_owned();
            match family {
                "ordinary" => source.push_str("x}"),
                "repeated" => source.push_str("let f=move||x;f()}"),
                _ => {
                    for index in 0..width {
                        write!(source, "let f{index}=move||x;let _=f{index}();")?;
                    }
                    source.push_str("x}");
                }
            }
            source.push_str("fn main(){");
            for _ in 0..if family == "independent" { 1 } else { width } {
                source.push_str("let _=take(7u8);");
            }
            source.push('}');
            let (work, closures) = measure(&source)?;
            assert_eq!(
                closures,
                match family {
                    "ordinary" => 0,
                    "repeated" => 1,
                    _ => width,
                }
            );
            assert!(work < 16384 * (width + 1), "{family}/{width}: {work}");
            eprintln!("{family}/{width}: canonical_steps={work}, closures={closures}");
            observations.push(work);
        }
        assert!(
            observations[2] - observations[1] <= 5 * (observations[1] - observations[0]),
            "{family}: {observations:?}"
        );
    }
    Ok(())
}
