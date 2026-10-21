//! Atomic RMW returns the old value and keeps runtime effects through reification.
#[path = "harness/native_references.rs"]
mod native;
#[path = "support/semantic.rs"]
mod semantic;
use std::{path::PathBuf, process::Command};
const CASES: &[(&str, i32, i32)] = &[
    ("swap-orders", 0, 0),
    ("add-orders", 0, 0),
    ("variable-order", 0, 0),
    ("aliased-order", 0, 0),
    ("reified", 0, 0),
    ("ufcs", 0, 0),
    ("shared-static", 0, 0),
    ("wrapping", 0, 0),
    ("unsafe-narrowing", 0, 0),
    ("const-swap", 1, 1),
    ("const-add", 1, 1),
    ("const-dead-add", 1, 1),
    ("const-reified", 1, 1),
    ("packed-receiver", 1, 0),
    ("wrong-value", 1, 1),
    ("wrong-order", 1, 1),
    ("wrong-arity", 1, 1),
    ("generic-argument", 1, 1),
    ("local-shadow", 0, 0),
    ("const-swap-alias", 1, 1),
    ("const-add-alias", 1, 1),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn atomic_rmw_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let output = root.join(format!(
        "obj/parallel/pa28/rmw-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&output)?;
    for &(name, status, types) in CASES {
        let path = root.join(format!("pa28/tests/spec/atomic-rmw/{name}.rs"));
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
            .arg(output.join(format!("{name}.o")))
            .arg(&path)
            .output()?;
        std::fs::write(
            output.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(independent.status.code(), Some(status), "{name}");
        for mode in ["--emit-types", "--emit-lowir", "--emit-constants"] {
            let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            std::fs::write(output.join(format!("{name}.{mode}.stdout")), &result.stdout)?;
            std::fs::write(output.join(format!("{name}.{mode}.stderr")), &result.stderr)?;
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
            let reason = match name {
                "const-swap" | "const-add" | "const-dead-add" | "const-swap-alias"
                | "const-add-alias" => {
                    Some("runtime-only intrinsic is not allowed during constant evaluation")
                }
                "const-reified" => Some("non-const calls are not allowed in constant initializers"),
                "packed-receiver" if mode != "--emit-types" => {
                    Some("reference to an unaligned packed field")
                }
                _ => None,
            };
            if let Some(reason) = reason {
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
#[test]
fn rmw_order_encoding_and_widths_are_typed() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::lowir::{
        atomic::RmwOrder,
        builder::Builder,
        program::{Operand, Value},
        vocabulary::Type,
    };
    let orders = [
        RmwOrder::Relaxed,
        RmwOrder::Release,
        RmwOrder::Acquire,
        RmwOrder::AcquireRelease,
        RmwOrder::Sequential,
    ];
    assert_eq!(orders.map(RmwOrder::encoding), [0, 3, 2, 4, 5]);
    let mut builder = Builder::default();
    let block = builder.block()?;
    builder.enter(block)?;
    for ty in [Type::I8, Type::U16, Type::I64, Type::F32, Type::Ptr] {
        let value = Value {
            ty,
            operand: Operand::Parameter(1),
        };
        assert!(
            builder
                .atomic_exchange(value, Operand::Parameter(0), RmwOrder::Relaxed)
                .is_err()
        );
        assert!(
            builder
                .atomic_add_fetch(value, Operand::Parameter(0), RmwOrder::Relaxed)
                .is_err()
        );
    }
    Ok(())
}
#[test]
fn rmw_registry_and_effectful_shims_grow_by_identity() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{lowir::program::Operation, support::intrinsics::Intrinsic};
    use std::fmt::Write;
    for count in [8, 32, 128] {
        let mut text = String::from(
            "use core::sync::atomic::{AtomicU32 as A,Ordering as O};static VALUE:A=A::new(0);",
        );
        for i in 0..count {
            write!(
                text,
                "fn f{i}(){{let _=VALUE.swap({i},O::Release);let _=VALUE.fetch_add(1,O::Acquire);}}"
            )?;
        }
        text.push_str("fn main(){");
        for i in 0..count {
            write!(text, "f{i}();")?;
        }
        text.push('}');
        let (mut report, mut facts) = semantic::analyze(&text)?;
        assert_eq!(facts.intrinsics.iter().count(), 11);
        for intrinsic in [Intrinsic::AtomicSwap, Intrinsic::AtomicFetchAdd] {
            let method = facts.intrinsics.get(intrinsic).ok_or("method")?;
            assert!(method.owner.is_some());
            assert!(method.parameter.is_none());
            assert!(facts.associated.trait_owner(method.definition).is_none());
        }
        assert!(facts.body_work.steps < 1024 * (count + 1));
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let program = rustgm::lower::instances::build(&report, &mut facts, entry)?;
        let mut exchange = 0;
        let mut add = 0;
        let mut loads = 0;
        let mut subtract = 0;
        for body in program
            .symbols
            .iter()
            .filter_map(|symbol| symbol.body.as_ref())
        {
            let rmw = body.instructions.iter().any(|instruction| {
                matches!(
                    instruction.operation,
                    Operation::AtomicExchange { .. } | Operation::AtomicAddFetch { .. }
                )
            });
            let mut tags = 0;
            for instruction in &body.instructions {
                if rmw {
                    match instruction.operation {
                        Operation::Load { ty, address, .. } => {
                            assert_eq!(ty, rustgm::lowir::vocabulary::Type::U8);
                            assert!(matches!(
                                address,
                                rustgm::lowir::program::Operand::Parameter(2)
                            ));
                            tags += 1;
                        }
                        Operation::Store { .. }
                        | Operation::AtomicLoad { .. }
                        | Operation::AtomicStore { .. }
                        | Operation::CopyObject { .. } => {
                            return Err("RMW shim performs an additional memory access".into());
                        }
                        _ => {}
                    }
                }
                match instruction.operation {
                    Operation::AtomicExchange { .. } => exchange += 1,
                    Operation::AtomicAddFetch { .. } => add += 1,
                    Operation::AtomicLoad { .. } => loads += 1,
                    Operation::Binary {
                        op: rustgm::lowir::vocabulary::Binary::Sub,
                        ..
                    } => subtract += 1,
                    _ => {}
                }
            }
            if rmw {
                assert_eq!(
                    tags, 1,
                    "only the Ordering argument is loaded outside the atomic operation"
                );
            }
        }
        assert_eq!(
            (exchange, add, loads, subtract),
            (5, 5, 0, 5),
            "{count}: exactly two RMW shims and no extra atomic read"
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn rmw_methods_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["orders", "wrapping", "reified", "effects-storage"] {
        let text = std::fs::read_to_string(
            root()?.join(format!("pa28/tests/spec/atomic-rmw-native/{name}.rs")),
        )?;
        native::check_cases("parallel/pa28/rmw-native", &[(name, &text, false)])?;
    }
    Ok(())
}
