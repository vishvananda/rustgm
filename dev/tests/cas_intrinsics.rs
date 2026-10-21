//! Exact CAS ordering, expected-slot updates and canonical Result identity.
#[path = "harness/native_references.rs"]
mod native;
#[path = "source_intrinsics/preparation.rs"]
mod preparation;
#[path = "support/semantic.rs"]
mod semantic;
use std::{path::PathBuf, process::Command};
const CASES: &[(&str, i32, i32)] = &[
    ("atomic-cas-orders/compare_exchange-relaxed-relaxed", 0, 0),
    ("atomic-cas-orders/compare_exchange-relaxed-release", 1, 1),
    ("atomic-cas-orders/compare_exchange-relaxed-acquire", 0, 0),
    ("atomic-cas-orders/compare_exchange-relaxed-acqrel", 1, 1),
    ("atomic-cas-orders/compare_exchange-relaxed-seqcst", 0, 0),
    ("atomic-cas-orders/compare_exchange-release-relaxed", 0, 0),
    ("atomic-cas-orders/compare_exchange-release-release", 1, 1),
    ("atomic-cas-orders/compare_exchange-release-acquire", 0, 0),
    ("atomic-cas-orders/compare_exchange-release-acqrel", 1, 1),
    ("atomic-cas-orders/compare_exchange-release-seqcst", 0, 0),
    ("atomic-cas-orders/compare_exchange-acquire-relaxed", 0, 0),
    ("atomic-cas-orders/compare_exchange-acquire-release", 1, 1),
    ("atomic-cas-orders/compare_exchange-acquire-acquire", 0, 0),
    ("atomic-cas-orders/compare_exchange-acquire-acqrel", 1, 1),
    ("atomic-cas-orders/compare_exchange-acquire-seqcst", 0, 0),
    ("atomic-cas-orders/compare_exchange-acqrel-relaxed", 0, 0),
    ("atomic-cas-orders/compare_exchange-acqrel-release", 1, 1),
    ("atomic-cas-orders/compare_exchange-acqrel-acquire", 0, 0),
    ("atomic-cas-orders/compare_exchange-acqrel-acqrel", 1, 1),
    ("atomic-cas-orders/compare_exchange-acqrel-seqcst", 0, 0),
    ("atomic-cas-orders/compare_exchange-seqcst-relaxed", 0, 0),
    ("atomic-cas-orders/compare_exchange-seqcst-release", 1, 1),
    ("atomic-cas-orders/compare_exchange-seqcst-acquire", 0, 0),
    ("atomic-cas-orders/compare_exchange-seqcst-acqrel", 1, 1),
    ("atomic-cas-orders/compare_exchange-seqcst-seqcst", 0, 0),
    (
        "atomic-cas-orders/compare_exchange_weak-relaxed-relaxed",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-relaxed-release",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-relaxed-acquire",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-relaxed-acqrel",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-relaxed-seqcst",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-release-relaxed",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-release-release",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-release-acquire",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-release-acqrel",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-release-seqcst",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acquire-relaxed",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acquire-release",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acquire-acquire",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acquire-acqrel",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acquire-seqcst",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acqrel-relaxed",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acqrel-release",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acqrel-acquire",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acqrel-acqrel",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-acqrel-seqcst",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-seqcst-relaxed",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-seqcst-release",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-seqcst-acquire",
        0,
        0,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-seqcst-acqrel",
        1,
        1,
    ),
    (
        "atomic-cas-orders/compare_exchange_weak-seqcst-seqcst",
        0,
        0,
    ),
    ("atomic-cas/ufcs-invalid-failure", 0, 0),
    ("atomic-cas/alias-invalid-failure", 0, 0),
    ("atomic-cas/variable-order", 0, 0),
    ("atomic-cas/reified", 0, 0),
    ("atomic-cas/result-alias", 0, 0),
    ("atomic-cas/local-result", 1, 1),
    ("atomic-cas/const-direct", 1, 1),
    ("atomic-cas/const-skipped", 1, 0),
    ("atomic-cas/const-alias", 1, 1),
    ("atomic-cas/const-reified", 1, 1),
    ("atomic-cas/wrong-current", 1, 1),
    ("atomic-cas/wrong-new", 1, 1),
    ("atomic-cas/wrong-success", 1, 1),
    ("atomic-cas/wrong-failure", 1, 1),
    ("atomic-cas/packed-receiver", 1, 0),
    ("atomic-cas/unsafe-narrowing", 0, 0),
    ("atomic-cas/no-prelude", 0, 0),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn compare_exchange_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let output = root.join(format!(
        "obj/parallel/pa28/cas-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&output)?;
    for &(name, status, types) in CASES {
        let path = root.join(format!("pa28/tests/spec/{name}.rs"));
        let stem = name.replace('/', "-");
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
            .arg(output.join(format!("{stem}.o")))
            .arg(&path)
            .output()?;
        std::fs::write(
            output.join(format!("{stem}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(
            independent.status.code(),
            Some(status),
            "{name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        for mode in ["--emit-types", "--emit-lowir", "--emit-constants"] {
            let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            std::fs::write(output.join(format!("{stem}.{mode}.stdout")), &result.stdout)?;
            std::fs::write(output.join(format!("{stem}.{mode}.stderr")), &result.stderr)?;
            let expected = if mode == "--emit-types" {
                types
            } else {
                status
            };
            assert_eq!(
                result.status.code(),
                Some(expected),
                "{name}/{mode}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(result.stderr.is_empty(), expected == 0, "{name}/{mode}");
            let reason = if name.starts_with("atomic-cas-orders/") && status == 1 {
                Some("invalid atomic ordering for this operation")
            } else {
                match name {
                    "atomic-cas/const-direct"
                    | "atomic-cas/const-skipped"
                    | "atomic-cas/const-alias" => {
                        Some("runtime-only intrinsic is not allowed during constant evaluation")
                    }
                    "atomic-cas/const-reified" => {
                        Some("non-const calls are not allowed in constant initializers")
                    }
                    "atomic-cas/packed-receiver" if mode != "--emit-types" => {
                        Some("reference to an unaligned packed field")
                    }
                    _ => None,
                }
            };
            if let Some(reason) = reason.filter(|_| expected != 0) {
                assert!(
                    String::from_utf8_lossy(&result.stderr).contains(reason),
                    "{name}/{mode}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
    }
    Ok(())
}
fn examine_shim(
    body: &rustgm::lowir::program::Function,
) -> Result<usize, Box<dyn std::error::Error>> {
    use rustgm::lowir::{
        program::{Operand, Operation},
        vocabulary::Type,
    };
    let mut pairs = std::collections::BTreeSet::new();
    let mut tags = 0;
    let mut expected_reads = 0;
    for instruction in &body.instructions {
        match instruction.operation {
            Operation::AtomicCompareExchange {
                value,
                address,
                ordering,
                ..
            } => {
                assert_eq!(value.ty, Type::U32);
                assert!(matches!(value.operand, Operand::Parameter(3)));
                assert!(matches!(address, Operand::Parameter(1)));
                pairs.insert((ordering.success_encoding(), ordering.failure_encoding()));
            }
            Operation::Load {
                ty: Type::U8,
                address: Operand::Parameter(4 | 5),
                ..
            } => tags += 1,
            Operation::Load {
                ty: Type::U32,
                address: Operand::Slot(_),
                ..
            } => expected_reads += 1,
            Operation::Load { .. }
            | Operation::AtomicLoad { .. }
            | Operation::AtomicStore { .. }
            | Operation::AtomicExchange { .. }
            | Operation::AtomicAddFetch { .. }
            | Operation::CopyObject { .. } => {
                return Err("CAS shim performs an extra object memory read/write".into());
            }
            Operation::Store {
                address: Operand::Parameter(1),
                ..
            } => return Err("CAS shim performs a non-atomic object store".into()),
            _ => {}
        }
    }
    assert_eq!((pairs.len(), tags, expected_reads), (15, 2, 15));
    for success in [0, 2, 3, 4, 5] {
        for failure in [0, 2, 5] {
            assert!(pairs.contains(&(success, failure)));
        }
    }
    Ok(pairs.len())
}
#[test]
fn compare_exchange_identity_layout_and_shims_scale() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{lowir::program::Operation, support::intrinsics::Intrinsic};
    use std::fmt::Write;
    for count in [8, 32, 128] {
        let mut text = String::from(
            "use core::sync::atomic::{AtomicU32 as A,Ordering as O};static VALUE:A=A::new(7);",
        );
        for i in 0..count {
            write!(
                text,
                "fn f{i}(){{let _=VALUE.compare_exchange(7,9,O::Relaxed,O::SeqCst);let _=VALUE.compare_exchange_weak(0,13,O::Release,O::Acquire);}}"
            )?;
        }
        text.push_str("fn main(){");
        for i in 0..count {
            write!(text, "f{i}();")?;
        }
        text.push('}');
        let (mut report, mut facts) = semantic::analyze(&text)?;
        assert_eq!(facts.intrinsics.iter().count(), 11);
        let result = facts
            .intrinsics
            .comparison_result()
            .ok_or("Result identity")?;
        assert!(facts.members.constructor(result.ok).is_some());
        assert!(facts.members.constructor(result.err).is_some());
        assert_ne!(result.ok, result.err);
        for kind in [
            Intrinsic::AtomicCompareExchange,
            Intrinsic::AtomicCompareExchangeWeak,
        ] {
            let method = facts.intrinsics.get(kind).ok_or("method")?;
            assert!(method.owner.is_some());
            assert!(method.parameter.is_none());
            assert!(facts.associated.trait_owner(method.definition).is_none());
        }
        assert!(facts.body_work.steps < 2048 * (count + 1));
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let program = rustgm::lower::instances::build(&report, &mut facts, entry)?;
        let mut shims = 0;
        let mut operations = 0;
        for body in program
            .symbols
            .iter()
            .filter_map(|symbol| symbol.body.as_ref())
        {
            if body
                .instructions
                .iter()
                .any(|i| matches!(i.operation, Operation::AtomicCompareExchange { .. }))
            {
                operations += examine_shim(body)?;
                shims += 1;
            }
        }
        assert_eq!((shims, operations), (2, 30));
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn compare_exchange_methods_execute() -> Result<(), Box<dyn std::error::Error>> {
    for (name, panic) in [
        ("strong-matrix", false),
        ("weak-matrix", false),
        ("reified", false),
        ("shared-storage", false),
        ("dynamic-invalid", true),
        ("weak-invalid", true),
        ("reified-invalid", true),
        ("ufcs-invalid", true),
    ] {
        let text = std::fs::read_to_string(
            root()?.join(format!("pa28/tests/spec/atomic-cas-native/{name}.rs")),
        )?;
        native::check_cases("parallel/pa28/cas-native", &[(name, &text, panic)])?;
    }
    Ok(())
}
