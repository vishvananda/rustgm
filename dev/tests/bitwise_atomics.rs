//! Canonical bitwise RMWs retain typed CAS-loop effects and runtime-only permission.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/bitwise_loops.rs"]
mod loops;
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use std::path::PathBuf;
const CASES: &[(&str, i32, i32, i32, &str)] = &[
    ("and-orders", 0, 0, 0, ""),
    ("variable-order", 0, 0, 0, ""),
    ("aliased-order", 0, 0, 0, ""),
    ("reified", 0, 0, 0, ""),
    ("ufcs", 0, 0, 0, ""),
    ("shared-static", 0, 0, 0, ""),
    ("masks", 0, 0, 0, ""),
    ("unsafe-narrowing", 0, 0, 0, ""),
    (
        "const-and",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-dead-call",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-reified",
        1,
        1,
        1,
        "non-const calls are not allowed in constant initializers",
    ),
    (
        "packed-receiver",
        1,
        0,
        1,
        "reference to an unaligned packed field",
    ),
    ("wrong-value", 1, 1, 1, ""),
    ("wrong-order", 1, 1, 1, ""),
    ("wrong-arity", 1, 1, 1, ""),
    ("generic-argument", 1, 1, 1, ""),
    ("local-shadow", 0, 0, 0, ""),
    (
        "const-method-alias",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-generic-skipped",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-unsafe",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    ("packed-raw", 0, 0, 0, ""),
    ("or-orders", 0, 0, 0, ""),
    (
        "const-or",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    ("xor-orders", 0, 0, 0, ""),
    (
        "const-xor",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    ("nand-orders", 0, 0, 0, ""),
    (
        "const-nand",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn bitwise_source_rules_match_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa28/bitwise-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, types, execution, reason) in CASES {
        let source = root.join(format!("pa28/tests/spec/atomic-bitwise/{name}.rs"));
        if types == execution {
            checked::check(&source, &evidence, name, (rust, execution, reason))?;
        } else {
            checked::check_modes(
                &source,
                &evidence,
                name,
                (rust, [types, execution, execution], reason),
            )?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn bitwise_old_values_and_effects_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "all-orders",
        "reified",
        "effects-storage",
        "zero-max",
        "once-arguments",
        "unchanged-values",
    ] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa28/tests/spec/atomic-bitwise-native/{name}.rs")),
        )?;
        native::check_cases("parallel/pa28/bitwise-native", &[(name, &source, false)])?;
    }
    Ok(())
}
#[test]
fn bitwise_loops_and_registry_have_bounded_growth() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        lowir::program::Operation,
        support::intrinsics::{ConstPermission, Intrinsic, Safety},
    };
    use std::fmt::Write;
    for (intrinsic, index) in [
        (Intrinsic::AtomicFetchAnd, 9),
        (Intrinsic::AtomicFetchOr, 10),
        (Intrinsic::AtomicFetchXor, 11),
        (Intrinsic::AtomicFetchNand, 12),
    ] {
        assert_eq!(intrinsic.index(), index);
        assert_eq!(intrinsic.descriptor().safety, Safety::Safe);
        assert_eq!(
            intrinsic.descriptor().constant,
            ConstPermission::RuntimeOnly
        );
    }
    for count in [8, 32, 128] {
        let mut source = String::from(
            "use core::sync::atomic::{AtomicU32 as A,Ordering as O};static VALUE:A=A::new(0);",
        );
        for index in 0..count {
            write!(
                source,
                "fn f{index}(){{let _=VALUE.fetch_and({index},O::Relaxed);let _=VALUE.fetch_or({index},O::Release);let _=VALUE.fetch_xor({index},O::Acquire);let _=VALUE.fetch_nand({index},O::SeqCst);}}"
            )?;
        }
        source.push_str("fn main(){");
        for index in 0..count {
            write!(source, "f{index}();")?;
        }
        source.push('}');
        rustc::qualify(
            &format!("parallel/pa28/bitwise-scaling-{count}"),
            &source,
            true,
        )?;
        let (mut report, mut facts) = semantic::analyze(&source)?;
        assert_eq!(facts.intrinsics.iter().count(), 11);
        assert!(facts.body_work.steps < 4096 * (count + 1));
        for intrinsic in [
            Intrinsic::AtomicFetchAnd,
            Intrinsic::AtomicFetchOr,
            Intrinsic::AtomicFetchXor,
            Intrinsic::AtomicFetchNand,
        ] {
            let method = facts.intrinsics.get(intrinsic).ok_or("bitwise identity")?;
            assert!(method.owner.is_some());
            assert!(method.parameter.is_none());
            assert!(facts.associated.trait_owner(method.definition).is_none());
        }
        println!(
            "callers={count} body_steps={} canonical_methods={}",
            facts.body_work.steps,
            facts.intrinsics.iter().count()
        );
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let program = rustgm::lower::instances::build(&report, &mut facts, entry)?;
        let shims = program
            .symbols
            .iter()
            .filter_map(|s| s.body.as_ref())
            .filter(|body| {
                body.instructions
                    .iter()
                    .any(|i| matches!(i.operation, Operation::AtomicCompareExchange { .. }))
            })
            .collect::<Vec<_>>();
        assert_eq!(shims.len(), 4);
        for body in shims {
            loops::check(body)?;
        }
    }
    Ok(())
}
