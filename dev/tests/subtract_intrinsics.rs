//! Fetch-sub uses captured add-and-fetch results with wrapping u32 arithmetic.
use crate::checked_sources as checked;
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
use std::path::PathBuf;
const CASES: &[(&str, i32, i32, i32, &str)] = &[
    ("sub-orders", 0, 0, 0, ""),
    ("variable-order", 0, 0, 0, ""),
    ("aliased-order", 0, 0, 0, ""),
    ("reified", 0, 0, 0, ""),
    ("ufcs", 0, 0, 0, ""),
    ("shared-static", 0, 0, 0, ""),
    ("wrapping", 0, 0, 0, ""),
    ("unsafe-narrowing", 0, 0, 0, ""),
    (
        "const-sub",
        1,
        1,
        1,
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-dead-sub",
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
        "const-sub-alias",
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
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn subtraction_matches_independent_objects_and_phase_rules()
-> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa28/subtract-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, types, execution, reason) in CASES {
        let source = root.join(format!("pa28/tests/spec/atomic-subtract/{name}.rs"));
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
fn subtraction_values_and_effects_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["orders-wrapping", "reified", "effects-storage", "zero-max"] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa28/tests/spec/atomic-subtract-native/{name}.rs")),
        )?;
        native::check_cases("parallel/pa28/subtract-native", &[(name, &source, false)])?;
    }
    Ok(())
}
#[test]
fn subtraction_shares_one_typed_effectful_shim() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        lowir::program::Operation,
        support::intrinsics::{ConstPermission, Intrinsic, Safety},
    };
    use std::fmt::Write;
    assert_eq!(Intrinsic::AtomicFetchSub.index(), 8);
    assert_eq!(Intrinsic::AtomicFetchSub.descriptor().safety, Safety::Safe);
    assert_eq!(
        Intrinsic::AtomicFetchSub.descriptor().constant,
        ConstPermission::RuntimeOnly
    );
    for count in [8, 32, 128] {
        let mut text = String::from(
            "use core::sync::atomic::{AtomicU32 as A,Ordering as O};static VALUE:A=A::new(0);",
        );
        for index in 0..count {
            write!(
                text,
                "fn f{index}(){{let _=VALUE.fetch_sub({index},O::Release);}}"
            )?;
        }
        text.push_str("fn main(){");
        for index in 0..count {
            write!(text, "f{index}();")?;
        }
        text.push('}');
        rustc::qualify(
            &format!("parallel/pa28/subtract-scaling-{count}"),
            &text,
            true,
        )?;
        let (mut report, mut facts) = semantic::analyze(&text)?;
        assert_eq!(facts.intrinsics.iter().count(), 13);
        let method = facts
            .intrinsics
            .get(Intrinsic::AtomicFetchSub)
            .ok_or("fetch_sub method")?;
        assert!(method.owner.is_some());
        assert!(method.parameter.is_none());
        assert!(facts.associated.trait_owner(method.definition).is_none());
        assert!(facts.body_work.steps < 1024 * (count + 1));
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
            .filter(|b| {
                b.instructions
                    .iter()
                    .any(|i| matches!(i.operation, Operation::AtomicAddFetch { .. }))
            })
            .collect::<Vec<_>>();
        assert_eq!(shims.len(), 1);
        check_shim(shims[0])?;
    }
    Ok(())
}
fn check_shim(body: &rustgm::lowir::program::Function) -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::lowir::{
        program::{Operand, Operation},
        vocabulary::{Binary, Type},
    };
    let (mut atomic, mut negation, mut reconstruction, mut tags) = (0, 0, 0, 0);
    let mut orders = std::collections::HashSet::new();
    for instruction in &body.instructions {
        match instruction.operation {
            Operation::AtomicAddFetch {
                value,
                address,
                ordering,
            } => {
                assert_eq!(value.ty, Type::U32);
                assert!(matches!(address, Operand::Parameter(0)));
                orders.insert(ordering.encoding());
                atomic += 1;
            }
            Operation::Binary {
                op: Binary::Sub,
                left,
                right,
            } => {
                assert_eq!(left.ty, Type::U32);
                assert!(matches!(right, Operand::Parameter(1)));
                negation += 1;
            }
            Operation::Binary {
                op: Binary::Add,
                left,
                right,
            } => {
                assert_eq!(left.ty, Type::U32);
                assert!(matches!(right, Operand::Parameter(1)));
                reconstruction += 1;
            }
            Operation::Load { ty, address, .. } => {
                assert_eq!(ty, Type::U8);
                assert!(matches!(address, Operand::Parameter(2)));
                tags += 1;
            }
            Operation::AtomicLoad { .. }
            | Operation::AtomicStore { .. }
            | Operation::AtomicExchange { .. }
            | Operation::AtomicCompareExchange { .. }
            | Operation::Store { .. }
            | Operation::CopyObject { .. } => {
                return Err("fetch_sub shim performs an additional memory operation".into());
            }
            _ => {}
        }
    }
    assert_eq!((atomic, negation, reconstruction, tags), (5, 5, 5, 1));
    assert_eq!(orders, std::collections::HashSet::from([0, 2, 3, 4, 5]));
    Ok(())
}
