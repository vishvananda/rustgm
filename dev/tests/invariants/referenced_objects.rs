//! Reference Self and leaf identity remain distinct across sealed vtable entries.
use super::{build, *};
use crate::types::{Kind, RegionId, closure, substitute::Bound};
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa24/tests/spec/reference-objects/{name}.rs")),
    )?)
}

#[test]
fn reference_tables_retain_complete_self_and_checked_adapters() -> Result {
    for name in ["shared", "nested-shared", "nested-mutable", "outer-result"] {
        let (mut facts, _) = build(&fixture(name)?)?;
        let storage = facts.objects.storage.as_ref().ok_or("missing plans")?;
        assert_eq!(storage.tables.len(), 1);
        assert!(storage.callable_targets.is_empty());
        let table = &storage.tables[0];
        assert!(matches!(
            facts.types.kind(table.concrete),
            Some(Kind::Reference { .. })
        ));
        let concrete = table.concrete;
        let targets = table.methods.iter().flatten().copied().collect::<Vec<_>>();
        assert!(!targets.is_empty());
        for target in targets {
            let Target::Captured(entry) = target else {
                return Err("uncaptured target".into());
            };
            let adapter = entry.adapter.ok_or("missing reference adapter")?;
            adapter.validate(&facts.types)?;
            assert_eq!(adapter.instance(), entry.instance);
            assert_eq!(adapter.requested(), entry.requested);
            assert_ne!(adapter.receiver(), entry.instance.ty());
            let mut erase = Bound::shape();
            assert_eq!(erase.apply(&mut facts.types, adapter.receiver())?, concrete);
            assert_eq!(
                closure::reference_leaf(&facts.types, adapter.receiver())?.0,
                Some(entry.instance.ty())
            );
            let Some(Kind::Function {
                parameters,
                bound_regions: 0,
                unsafe_: false,
                convention,
                ..
            }) = facts.types.kind(entry.execution)
            else {
                return Err("invalid execution ABI".into());
            };
            assert_eq!(convention, crate::types::callable::Convention::RUST);
            let receiver = facts
                .types
                .list(parameters)
                .and_then(|p| p.first())
                .copied()
                .ok_or("no receiver")?;
            assert!(
                matches!(facts.types.kind(receiver), Some(Kind::Reference { referent, mutable, .. }) if referent == adapter.receiver() && mutable == (entry.requested == CallTrait::FnMut))
            );
            let count = facts
                .objects
                .storage
                .as_ref()
                .ok_or("plans")?
                .captured_targets
                .len();
            assert!(
                prepare_captured(&mut facts, entry.instance.ty(), [Some(entry), None]).is_err()
            );
            let mut changed = entry;
            changed.adapter = None;
            assert!(
                prepare_captured(&mut facts, adapter.receiver(), [Some(changed), None]).is_err()
            );
            changed = entry;
            changed.requested = if entry.requested == CallTrait::Fn {
                CallTrait::FnMut
            } else {
                CallTrait::Fn
            };
            assert!(
                prepare_captured(&mut facts, adapter.receiver(), [Some(changed), None]).is_err()
            );
            assert_eq!(
                facts
                    .objects
                    .storage
                    .as_ref()
                    .ok_or("plans")?
                    .captured_targets
                    .len(),
                count
            );
        }
    }
    Ok(())
}

#[test]
fn once_reference_view_has_no_executable_slot() -> Result {
    let (facts, _) = build(&fixture("once-view")?)?;
    let storage = facts.objects.storage.as_ref().ok_or("plans")?;
    assert_eq!(storage.tables.len(), 1);
    assert!(storage.tables[0].methods.iter().all(Option::is_none));
    assert!(storage.captured_targets.is_empty());
    assert!(storage.callable_targets.is_empty());
    Ok(())
}

#[test]
fn neutral_reference_walk_counts_only_the_touched_chain() -> Result {
    let (mut facts, _) = build(&fixture("shared")?)?;
    let leaf = facts
        .objects
        .storage
        .as_ref()
        .ok_or("plans")?
        .captured_targets
        .values()
        .next()
        .ok_or("target")?
        .instance
        .ty();
    let mut ty = leaf;
    for depth in 0..=128 {
        assert_eq!(
            closure::reference_leaf(&facts.types, ty)?,
            (Some(leaf), depth + 1)
        );
        ty = facts.types.intern(Kind::Reference {
            referent: ty,
            mutable: false,
            region: RegionId::STATIC,
        })?;
    }
    let pointer = facts.types.intern(Kind::Pointer {
        pointee: ty,
        mutable: false,
    })?;
    assert_eq!(closure::reference_leaf(&facts.types, pointer)?, (None, 1));
    assert!(closure::reference_leaf(&facts.types, TypeId(u32::MAX)).is_err());
    Ok(())
}

#[test]
fn reference_depth_and_repetition_keep_distinct_work_and_shared_tables() -> Result {
    for depth_axis in [false, true] {
        let mut observations = Vec::new();
        for width in [8usize, 32, 128] {
            let depth = if depth_axis { width } else { 1 };
            let repeat = if depth_axis { 1 } else { width };
            let mut source = "fn main(){let n=7u8;let f=move|x:u8|n+x;let r0=&f;".to_owned();
            for index in 1..depth {
                write!(source, "let r{index}=&r{};", index - 1)?;
            }
            for _ in 0..repeat {
                write!(
                    source,
                    "{{let p:&dyn Fn(u8)->u8=&r{};let _=p(1);}}",
                    depth - 1
                )?;
            }
            source.push('}');
            let (facts, _) = build(&source)?;
            let storage = facts.objects.storage.as_ref().ok_or("plans")?;
            assert_eq!(storage.tables.len(), 1);
            assert_eq!(storage.captured_targets.len(), 2);
            assert!(storage.callable_targets.is_empty());
            assert!(storage.steps <= 8192 * (depth + repeat + 1));
            eprintln!(
                "reference/{depth}/{repeat}: plans={}, canonical={}",
                storage.steps, facts.body_work.canonical_steps
            );
            observations.push(storage.steps);
        }
        assert!(observations[2] - observations[1] <= 5 * (observations[1] - observations[0]));
    }
    Ok(())
}
