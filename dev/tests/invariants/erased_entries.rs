//! Complete trait inputs choose slots; runtime sharing never supplies source permission.
use super::*;
use crate::{sema, support::call_traits::CallTrait};
use std::fmt::Write;

fn build(
    source: &str,
) -> Result<(sema::model::Declarations, crate::lowir::program::Program), Box<dyn std::error::Error>>
{
    let mut report = crate::test_source::resolve(source, true)?;
    let facts = sema::declarations::analyze(&report)?;
    let mut facts = sema::bodies::analyze(&report, facts)?;
    let entry = sema::entry::select(&mut report, &facts)?;
    let output = crate::lower::instances::build(&report, &mut facts, entry)?;
    Ok((facts, output))
}
#[test]
fn complete_callable_inputs_keep_distinct_slots() -> Result<(), Box<dyn std::error::Error>> {
    let source = "fn a(x:u8)->u8{x}fn b(x:u16)->u16{x}fn main(){let p:&dyn Fn(u8)->u8=&a;let q:&dyn Fn(u16)->u16=&b;let _=p(1);let _=q(2);}";
    let (facts, _) = build(source)?;
    let storage = facts
        .objects
        .storage
        .as_ref()
        .ok_or("missing object plans")?;
    assert_eq!(storage.shapes.len(), 2);
    assert_eq!(storage.callable_targets.len(), 2);
    for shape in &storage.shapes {
        assert_eq!(shape.slots.len(), 3);
        for kind in CallTrait::ALL {
            assert_eq!(
                shape
                    .slots
                    .iter()
                    .filter(|s| s.method == Method::Callable(kind))
                    .count(),
                1
            );
        }
    }
    let first = &storage.shapes[0];
    for slot in &storage.shapes[1].slots {
        assert!(!first.indices.contains_key(&(slot.method, slot.parent)));
    }
    Ok(())
}
#[test]
fn physical_reference_prefixes_share_only_equal_forwarding()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn a(x:u8)->u8{x}fn apply<F:FnOnce(u8)->u8>(f:F)->u8{f(1)}fn main(){let p:&dyn Fn(u8)->u8=&a;let _=apply(p);let _=apply(&p);let _=apply(&&p);}";
    let (facts, output) = build(source)?;
    let storage = facts
        .objects
        .storage
        .as_ref()
        .ok_or("missing object plans")?;
    assert_eq!(storage.callable_targets.len(), 1);
    assert!(storage.operators.values().any(|p| p.dereferences == 0));
    assert!(storage.operators.values().any(|p| p.dereferences == 1));
    assert_eq!(
        output
            .symbols
            .iter()
            .filter(|s| s.name.starts_with("rg.callable.virtual."))
            .count(),
        2
    );
    Ok(())
}
#[test]
fn repeated_dispatch_shares_tables_and_has_bounded_plan_work()
-> Result<(), Box<dyn std::error::Error>> {
    let mut observations = Vec::new();
    for width in [8usize, 32, 128] {
        let mut source = "fn a(x:u8)->u8{x}fn main(){let p:&dyn Fn(u8)->u8=&a;".to_owned();
        for index in 0..width {
            write!(source, "let _=p({index}u8);")?;
        }
        source.push('}');
        let (facts, output) = build(&source)?;
        let storage = facts
            .objects
            .storage
            .as_ref()
            .ok_or("missing object plans")?;
        assert_eq!(storage.tables.len(), 1);
        assert_eq!(storage.callable_targets.len(), 1);
        assert_eq!(
            output
                .symbols
                .iter()
                .filter(|s| s.name.starts_with("rg.callable.virtual."))
                .count(),
            1
        );
        assert!(storage.steps <= 4096 * (width + 1));
        eprintln!(
            "dispatch/{width}: plan_steps={}, operators={}",
            storage.steps,
            storage.operators.len()
        );
        observations.push(storage.steps);
    }
    assert!(observations[2] - observations[1] <= 5 * (observations[1] - observations[0]));
    Ok(())
}

#[test]
fn captured_targets_share_sealed_entries_with_bounded_conversion_work()
-> Result<(), Box<dyn std::error::Error>> {
    for distinct in [false, true] {
        let mut observations = Vec::new();
        for width in [8usize, 32, 128] {
            let mut source = if distinct {
                "fn run<const N:u16>(x:u16)->u16{let f=move||x+N;let p:&dyn Fn()->u16=&f;p()}fn main(){".to_owned()
            } else {
                "fn main(){let x=7u16;let f=move||x;".to_owned()
            };
            for index in 0..width {
                if distinct {
                    write!(source, "let _=run::<{index}>(7);")?;
                } else {
                    source.push_str("{let p:&dyn Fn()->u16=&f;let _=p();}");
                }
            }
            source.push('}');
            let (facts, output) = build(&source)?;
            let storage = facts
                .objects
                .storage
                .as_ref()
                .ok_or("missing captured object plans")?;
            let instances = if distinct { width } else { 1 };
            assert_eq!(storage.tables.len(), instances);
            assert_eq!(storage.shapes.len(), 1);
            assert_eq!(storage.captured_targets.len(), 2 * instances);
            assert!(storage.callable_targets.is_empty());
            assert_eq!(
                output
                    .symbols
                    .iter()
                    .filter(|s| s.name.starts_with("rg.closure.adapter."))
                    .count(),
                instances
            );
            assert_eq!(
                output
                    .symbols
                    .iter()
                    .filter(|s| s.name.starts_with("rg.callable.virtual."))
                    .count(),
                1
            );
            assert!(storage.steps <= 4096 * (width + 1));
            eprintln!(
                "captured/{distinct}/{width}: steps={}, tables={}, targets={}",
                storage.steps,
                storage.tables.len(),
                storage.captured_targets.len()
            );
            observations.push(storage.steps);
        }
        assert!(observations[2] - observations[1] <= 5 * (observations[1] - observations[0]));
    }
    Ok(())
}
