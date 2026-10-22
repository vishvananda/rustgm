//! U32 extrema retain safe runtime-only calls and unsigned CAS-loop semantics.
use crate::checked_sources as checked;
#[path = "extremum_atomics/interference.rs"]
mod interference;
use crate::bitwise_loops as loops;
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
use std::path::Path;
const CASES: &[(&str, i32, [i32; 3], &str)] = &[
    ("fetch_min-relaxed", 0, [0, 0, 0], ""),
    ("fetch_min-release", 0, [0, 0, 0], ""),
    ("fetch_min-acquire", 0, [0, 0, 0], ""),
    ("fetch_min-acqrel", 0, [0, 0, 0], ""),
    ("fetch_min-seqcst", 0, [0, 0, 0], ""),
    ("fetch_max-relaxed", 0, [0, 0, 0], ""),
    ("fetch_max-release", 0, [0, 0, 0], ""),
    ("fetch_max-acquire", 0, [0, 0, 0], ""),
    ("fetch_max-acqrel", 0, [0, 0, 0], ""),
    ("fetch_max-seqcst", 0, [0, 0, 0], ""),
    ("variable-order", 0, [0, 0, 0], ""),
    ("reified", 0, [0, 0, 0], ""),
    ("ufcs", 0, [0, 0, 0], ""),
    ("std-import", 0, [0, 0, 0], ""),
    ("no-prelude", 0, [0, 0, 0], ""),
    ("shared-static", 0, [0, 0, 0], ""),
    ("local-shadow", 0, [0, 0, 0], ""),
    (
        "const-min",
        1,
        [1, 1, 1],
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-max",
        1,
        [1, 1, 1],
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-skipped",
        1,
        [0, 1, 1],
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-alias",
        1,
        [1, 1, 1],
        "runtime-only intrinsic is not allowed during constant evaluation",
    ),
    (
        "const-reified",
        1,
        [1, 1, 1],
        "non-const calls are not allowed in constant initializers",
    ),
    (
        "packed-receiver",
        1,
        [0, 1, 1],
        "reference to an unaligned packed field",
    ),
    ("packed-address", 0, [0, 0, 0], ""),
    ("wrong-value", 1, [1, 1, 1], "incompatible types"),
    ("wrong-order", 1, [1, 1, 1], "incompatible types"),
    ("wrong-arity", 1, [1, 1, 1], "call argument count mismatch"),
    (
        "generic-argument",
        1,
        [1, 1, 1],
        "incorrect number of type and const arguments",
    ),
];
#[test]
fn extrema_source_matches_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa28/extrema-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, rust, front, reason) in CASES {
        let path = root.join(format!("pa28/tests/spec/atomic-extrema/{name}.rs"));
        if front == [front[0]; 3] {
            checked::check(&path, &out, name, (rust, front[0], reason))?;
        } else {
            checked::check_modes(&path, &out, name, (rust, front, reason))?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn extrema_old_values_and_unsigned_edges_execute() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let mut cases = Vec::new();
    for name in [
        "all-orders-unsigned",
        "equal-and-unused",
        "operands-once",
        "reified",
        "static-nested",
        "aliased-order",
    ] {
        cases.push((
            name,
            std::fs::read_to_string(
                root.join(format!("pa28/tests/atomic-extrema-native/{name}.rs")),
            )?,
        ));
    }
    let borrowed = cases
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa28/extrema-native", &borrowed)
}

#[test]
fn extrema_loops_and_canonical_registry_stay_bounded() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::support::intrinsics::{ConstPermission, Intrinsic, Safety};
    use std::fmt::Write;
    for (kind, index) in [
        (Intrinsic::AtomicFetchMin, 15),
        (Intrinsic::AtomicFetchMax, 16),
    ] {
        assert_eq!(kind.index(), index);
        assert_eq!(kind.descriptor().safety, Safety::Safe);
        assert_eq!(kind.descriptor().constant, ConstPermission::RuntimeOnly);
    }
    for count in [8, 32, 128] {
        let mut text = String::from(
            "use core::sync::atomic::{AtomicU32,Ordering};static A:AtomicU32=AtomicU32::new(7);",
        );
        for n in 0..count {
            write!(
                text,
                "fn f{n}(){{let _=A.fetch_min({n},Ordering::Release);let _=A.fetch_max({n},Ordering::Acquire);}}"
            )?;
        }
        text.push_str("fn main(){");
        for n in 0..count {
            write!(text, "f{n}();")?;
        }
        text.push('}');
        rustc::qualify(
            &format!("parallel/pa28/extrema-scaling-{count}"),
            &text,
            true,
        )?;
        let (mut report, mut facts) = semantic::analyze(&text)?;
        assert_eq!(facts.intrinsics.iter().count(), 13);
        assert!(facts.body_work.steps < 4096 * (count + 1));
        for kind in [Intrinsic::AtomicFetchMin, Intrinsic::AtomicFetchMax] {
            let method = facts
                .intrinsics
                .get(kind)
                .ok_or("canonical extremum identity")?;
            assert!(method.owner.is_some());
            assert!(method.parameter.is_none());
            assert!(facts.associated.trait_owner(method.definition).is_none());
        }
        println!(
            "callers={count} body_steps={} methods={}",
            facts.body_work.steps,
            facts.intrinsics.iter().count()
        );
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let program = rustgm::lower::instances::build(&report, &mut facts, entry)?;
        let mut shims = 0;
        for symbol in &program.symbols {
            if let Some(maximum) = interference::family(&symbol.name) {
                let body = symbol.body.as_ref().ok_or("missing extremum shim")?;
                loops::check(body, Some(maximum))?;
                shims += 1;
                println!(
                    "maximum={maximum} instructions={} blocks={}",
                    body.instructions.len(),
                    body.blocks.len()
                );
            }
        }
        assert_eq!(shims, 2);
    }
    Ok(())
}
