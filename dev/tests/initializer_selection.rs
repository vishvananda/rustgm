//! Eager named values select completed associated contracts before lowering.
use crate::native_references as native;
use crate::resolve as source;
use rustgm::sema::{model::Scalar, values::Reader};
use std::{fmt::Write, path::Path, process::Command};

#[test]
fn historical_associated_values_restore_lowir_and_completed_dispatch()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for (folder, name) in [
        ("constants/value", "global"),
        ("integration/inspection", "members"),
        ("integration/inspection", "mixed"),
    ] {
        for (lane, mode) in [
            ("semantics", "--emit-trait-semantics"),
            ("lowir", "--emit-lowir"),
        ] {
            let path = root.join(format!("pa20/tests/spec/{folder}/{lane}/{name}.rs"));
            let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            assert!(
                output.status.success(),
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty());
            let mut reference = path.as_os_str().to_os_string();
            reference.push(".ref");
            let actual = String::from_utf8(output.stdout)?;
            if lane == "lowir" {
                assert_eq!(
                    actual,
                    std::fs::read_to_string(reference)?,
                    "{folder}/{name}"
                );
            } else {
                // Required evaluation now completes a real dispatch before inspection.
                // Its selected target must remain visible, rather than being hidden to
                // reproduce the historical table's incomplete selection inventory.
                assert!(
                    actual
                        .lines()
                        .any(|line| line.starts_with("constant-dispatch "))
                );
            }
        }
    }
    Ok(())
}
#[test]
fn repeated_unused_associated_roots_share_completed_dependency_selection()
-> Result<(), Box<dyn std::error::Error>> {
    for width in [8usize, 32, 128] {
        let mut text = String::from("trait A{const C:u8=7;}impl A for u8{}");
        for index in 0..width {
            write!(text, "const C{index}:u8=<u8 as A>::C;")?;
        }
        text.push_str("fn main(){}");
        let report = source::resolve(&text, true)?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        assert_eq!(analysis.facts.constant_work.initializer_validations, width);
        for initializer in &analysis.facts.initializers {
            if analysis
                .facts
                .associated
                .owner(initializer.definition)
                .is_none()
            {
                assert_eq!(
                    Reader::new(&analysis.facts)
                        .definition(initializer.definition)
                        .ok_or("missing eager value")?
                        .as_scalar()?,
                    Scalar::Integer(7)
                );
            }
        }
        let work = analysis.facts.dispatch_work();
        assert!(work.selections <= 2 * width + 4);
        assert!(work.dependency_expansions <= width + 4);
    }
    Ok(())
}
#[test]
fn raw_temporary_preserves_the_existing_place_diagnostic() -> Result<(), Box<dyn std::error::Error>>
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let path =
        root.join("pa22/tests/spec/constants/pointer-constants/cases/constant-raw-temporary.rs");
    let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .arg(&path)
        .output()?;
    assert!(!output.status.success());
    let mut reference = path.as_os_str().to_os_string();
    reference.push(".ref.stderr");
    assert_eq!(output.stderr, std::fs::read(reference)?);
    Ok(())
}
#[test]
#[ignore = "requires configured CPPGM native backend and pinned Rust toolchain"]
fn associated_default_override_static_and_lifetime_arguments_execute()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "parallel/pa25-initializer-selection",
        &[
            (
                "default-override",
                "trait A{const C:u8=7;}struct S;struct U;impl A for S{}impl A for U{const C:u8=9;}const X:u8=<S as A>::C;static Y:u8=<U as A>::C;fn main(){check(X==7&&Y==9);}",
                false,
            ),
            (
                "lifetime-type-const",
                "trait A<'a,T:'a,const N:usize>{type Out;const C:usize=N;}struct S;impl<'a,T:'a,const N:usize>A<'a,T,N>for S{type Out=&'a T;}const C:usize=<S as A<'static,u8,3>>::C;fn main(){check(C==3);}",
                false,
            ),
            (
                "nested-unused",
                "trait A{const C:u8=7;}impl A for u8{}const X:u8=<u8 as A>::C;const Y:u8=X;fn main(){check(Y==7);}",
                false,
            ),
        ],
    )
}
