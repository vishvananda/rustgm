//! Actual wide-reference helpers consume runtime counts, including zero stride.
use super::TestResult;
use crate::provider_traits::lower;
use std::path::Path;

const POSITIVE: &[(&str, &str)] = &[
    ("original-zst-slice3", "3\n"),
    ("zst-count-0", "0\n"),
    ("zst-count-1", "1\n"),
    ("zst-count-8", "8\n"),
    ("zst-count-32", "32\n"),
    ("zst-count-128", "128\n"),
    ("dropless-empty-registry", "0\n"),
    ("borrowed-reference-elements", "0\n"),
    ("generic-helper", "3\n"),
    ("reified-helper", "3\n"),
    ("stride-and-order", "3 0\n"),
];
const ARRAYS: &[(&str, &str)] = &[
    ("sized-zst-array1", "1\n"),
    ("sized-zst-array8", "8\n"),
    ("sized-stride-order3", "3 0\n"),
];
fn fixture(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../pa31/tests/spec/drop-glue-slices/{name}.{extension}"
        )),
    )?)
}
#[test]
fn actual_slice_helpers_lower_with_exact_wide_arguments() -> TestResult {
    lower_cases(POSITIVE)
}
#[test]
fn shared_array_loop_preserves_constant_bounds_and_optimizations() -> TestResult {
    lower_cases(ARRAYS)
}
fn lower_cases(cases: &[(&str, &str)]) -> TestResult {
    let mut failures = Vec::new();
    for &(name, _) in cases {
        if let Err(error) = lower(&fixture(name, "rs")?) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn actual_slice_helpers_preserve_unique_borrow_and_lifetime_opposites() -> TestResult {
    for name in [
        "shared-is-not-mutable",
        "raw-is-not-reference",
        "overlapping-mutable-borrows",
        "scoped-escape",
    ] {
        assert!(lower(&fixture(name, "rs")?).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn runtime_slice_extent_does_not_expand_shared_cleanup_emission() -> TestResult {
    use rustgm::lowir::{program::Signature, vocabulary::Type};
    let mut baseline = None;
    for length in [0, 1, 8, 32, 128] {
        let program = lower(&fixture(&format!("zst-count-{length}"), "rs")?)?;
        let mut loops = Vec::new();
        for symbol in &program.symbols {
            let Some(body) = &symbol.body else { continue };
            let Signature::Scalar(signature) = &program.signatures[symbol.signature.0 as usize]
            else {
                continue;
            };
            let parameters = &program.parameters
                [signature.parameters.start as usize..signature.parameters.end as usize];
            if parameters == [Type::Ptr, Type::I64] {
                loops.push((
                    body.instructions.len(),
                    body.blocks.len(),
                    body.slots.len(),
                    body.arguments.len(),
                ));
            }
        }
        assert_eq!(loops.len(), 1, "length{length}");
        println!("length{length}: shared loop {loops:?}");
        if let Some(expected) = &baseline {
            assert_eq!(&loops, expected);
        } else {
            baseline = Some(loops);
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_slice_helpers_native_counts_stride_and_referent_ownership() -> TestResult {
    native_cases(POSITIVE, "slices")
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn shared_array_loop_native_stride_and_zst_regressions() -> TestResult {
    native_cases(ARRAYS, "arrays")
}
fn native_cases(cases: &[(&str, &str)], label: &str) -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/slice-glue-native")
        .join(label)
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, expected) in cases {
        let checked = || -> TestResult {
            let program = lower(&fixture(name, "rs")?).map_err(|e| format!("{name}: {e}"))?;
            crate::ffi_program::check(
                root,
                &folder,
                name,
                &program,
                &fixture(name, "c")?,
                expected,
            )?;
            assert!(std::fs::read(folder.join(format!("{name}-execute.stderr")))?.is_empty());
            Ok(())
        };
        if let Err(error) = checked() {
            failures.push(format!("{name}: {error}"));
        } else {
            println!("{name}: native observed {expected:?}");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
