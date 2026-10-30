//! A completed selection identifies the whole callable, never another proof.
use super::*;
use crate::{
    body::ir,
    types::{Primitive, Store, arguments::Argument},
};
use std::{cell::Cell, fmt::Write as _};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "trait Transfer<T>{fn transfer(self)->T;}impl<T>Transfer<T> for T{fn transfer(self)->T{self}}trait Other{fn other(self)->u8;}impl Other for u8{fn other(self)->u8{self}}fn unrelated(x:u8)->u8{x}fn observed(x:u8)->u8{x.transfer()}fn main(){}";
#[derive(Clone, Copy)]
struct Call {
    source: NodeKey,
    original: TypeId,
    actual: TypeId,
}
fn prepared(text: &str) -> Result<(Declarations, Vec<Call>)> {
    let report = crate::test_source::resolve(text, true)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let calls = crate::lower::instances::inspect_prepared(
        &report,
        &mut facts,
        prepared,
        |facts, checked| -> Result<Vec<Call>> {
            let mut result = Vec::new();
            for body in &checked.program().bodies {
                for statement in &body.statements {
                    let ir::Action::Evaluate(value) = statement.action else {
                        continue;
                    };
                    let Some(ir::Value {
                        kind: ir::Rvalue::Call { callee, .. },
                        ..
                    }) = body.values.get(value.0 as usize)
                    else {
                        continue;
                    };
                    let Some(source) = body.source.proof_site(statement.source)?.source() else {
                        continue;
                    };
                    let Some(method) = facts.methods.calls.get(&source) else {
                        continue;
                    };
                    let actual = body
                        .values
                        .get(callee.0 as usize)
                        .ok_or("actual callee")?
                        .ty;
                    if actual != method.callable {
                        result.push(Call {
                            source,
                            original: method.callable,
                            actual,
                        });
                    }
                }
            }
            Ok(result)
        },
    )??;
    if calls.is_empty() {
        return Err("actual source dispatch did not rewrite a method".into());
    }
    Ok((facts, calls))
}
#[test]
fn completed_selection_reconstructs_full_generic_and_late_bound_callables() -> Result {
    for text in [
        SOURCE,
        "trait Borrow{fn borrow(&self)->&u8;}struct Token(u8);impl Borrow for Token{fn borrow(&self)->&u8{&self.0}}fn observed(x:&Token)->u8{*x.borrow()}fn main(){}",
    ] {
        let (mut facts, calls) = prepared(text)?;
        let work = Cell::new(0usize);
        let bytes = Cell::new(0usize);
        let pay = |s: usize, b: usize| {
            work.set(work.get().saturating_add(s));
            bytes.set(bytes.get().saturating_add(b));
            Ok(())
        };
        for call in calls {
            assert_eq!(
                selected_callable(&mut facts, call.source, call.original, &pay)?,
                Some(call.actual)
            );
            let goal =
                callable_requirement(&mut facts, call.actual, &pay)?.ok_or("actual impl goal")?;
            let original = callable_requirement(&mut facts, call.original, &pay)?
                .ok_or("actual trait goal")?;
            assert_eq!(goal.ty, original.ty);
            assert_eq!(
                goal.bound.trait_parts().map(|p| p.0),
                original.bound.trait_parts().map(|p| p.0)
            );
            let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(call.original) else {
                return Err("actual requested trait member".into());
            };
            for recorded in [Some(original), None] {
                assert_eq!(
                    requested_callable(&mut facts, DefId(item.0), call.actual, recorded, &pay)?,
                    call.original,
                    "real calls and reified items retain the complete formal order"
                );
            }
        }
        assert!(work.get() > 0);
        assert!(bytes.get() > 0);
    }
    Ok(())
}
#[test]
fn changed_arguments_owner_source_store_and_failed_payment_cannot_reuse_a_target() -> Result {
    for opponent in 0..5 {
        let (mut facts, calls) = prepared(SOURCE)?;
        let call = calls[0];
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(call.actual)
        else {
            return Err("actual selected item".into());
        };
        let wrong = facts
            .types
            .intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::Bool))])?;
        let wrong = facts.types.intern(Kind::FunctionItem {
            item,
            arguments: wrong,
            signature,
        })?;
        let target = selected_callable(&mut facts, call.source, call.original, &|_, _| Ok(()))?;
        assert_ne!(
            target,
            Some(wrong),
            "same impl declaration with another complete application"
        );
        let unrelated = facts
            .definitions
            .iter()
            .copied()
            .flatten()
            .find(|&ty| {
                matches!(facts.types.kind(ty), Some(Kind::FunctionItem {
                item: other, signature: shape, ..
            }) if other != item && shape == signature)
            })
            .ok_or("real unrelated declaration with the same signature")?;
        assert_ne!(target, Some(unrelated));
        reverse_opponents(&mut facts, call, wrong, unrelated)?;
        match opponent {
            0 => {
                let selected = facts
                    .generics
                    .trait_selections
                    .get_mut(&call.source)
                    .ok_or("actual site selections")?;
                selected[0].owner = DefId(u32::MAX);
            }
            1 => {
                facts.generics.trait_selections.remove(&call.source);
            }
            2 => {
                facts.types = Store::default();
            }
            3 => {
                facts.generics.trait_self_bounds.clear();
            }
            _ => {}
        }
        let attempts = Cell::new(0usize);
        let result = selected_callable(&mut facts, call.source, call.original, &|s, _| {
            attempts.set(attempts.get() + s);
            if opponent == 4 {
                Err("closed payment")
            } else {
                Ok(())
            }
        });
        assert!(result.is_err() || result? != Some(call.actual));
        assert!(attempts.get() > 0);
    }
    let (mut facts, calls) = prepared(SOURCE)?;
    let call = calls[0];
    let source = NodeKey {
        file: crate::resolve::unit::FileId(u32::MAX),
        ..call.source
    };
    assert_eq!(
        selected_callable(&mut facts, source, call.original, &|_, _| Ok(()))?,
        None
    );
    let before = facts.types.node_table_allocation().bytes;
    let attempts = Cell::new(0usize);
    for _ in 0..2 {
        assert!(
            selected_callable(&mut facts, call.source, call.original, &|s, _| {
                attempts.set(attempts.get() + s);
                Err("closed payment")
            })
            .is_err()
        );
    }
    assert_eq!(attempts.get(), 2);
    assert_eq!(facts.types.node_table_allocation().bytes, before);
    Ok(())
}
fn reverse_opponents(
    facts: &mut Declarations,
    call: Call,
    wrong: TypeId,
    unrelated: TypeId,
) -> Result {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(call.original) else {
        return Err("actual original trait member".into());
    };
    let goal = callable_requirement(facts, call.original, &|_, _| Ok(()))?
        .ok_or("actual complete method goal")?;
    for selected in [wrong, unrelated] {
        assert!(
            requested_callable(facts, DefId(item.0), selected, Some(goal), &|_, _| Ok(())).is_err()
        );
    }
    let changed = Predicate {
        ty: TypeId::primitive(Primitive::Bool),
        ..goal
    };
    assert!(
        requested_callable(
            facts,
            DefId(item.0),
            call.actual,
            Some(changed),
            &|_, _| Ok(())
        )
        .is_err()
    );
    assert!(
        requested_callable(facts, DefId(u32::MAX), call.actual, Some(goal), &|_, _| Ok(
            ()
        ))
        .is_err()
    );
    let attempted = Cell::new(0usize);
    for _ in 0..2 {
        assert!(
            requested_callable(
                facts,
                DefId(item.0),
                call.actual,
                Some(goal),
                &|steps, _| {
                    attempted.set(attempted.get().saturating_add(steps));
                    Err("denied exact structural query")
                }
            )
            .is_err()
        );
    }
    assert!(attempted.get() >= 2);
    Ok(())
}
#[test]
fn repeated_source_calls_pay_linear_reconstruction_without_a_new_lookup_search() -> Result {
    let mut previous = None;
    for width in [8usize, 32, 128] {
        let mut text = String::from(
            "trait Transfer<T>{fn transfer(self)->T;}impl<T>Transfer<T> for T{fn transfer(self)->T{self}}fn observed(x:u8)->u8{",
        );
        for _ in 0..width {
            write!(text, "let _=x.transfer();")?;
        }
        text.push_str("x.transfer()}fn main(){}");
        let (mut facts, calls) = prepared(&text)?;
        assert_eq!(calls.len(), width + 1);
        let work = Cell::new(0usize);
        let bytes = Cell::new(0usize);
        let selections = facts.dispatch_dependencies.work.selections;
        for call in &calls {
            let pay = |s, b| {
                work.set(work.get() + s);
                bytes.set(bytes.get() + b);
                Ok(())
            };
            assert_eq!(
                selected_callable(&mut facts, call.source, call.original, &pay)?,
                Some(call.actual)
            );
            let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(call.original) else {
                return Err("actual complete trait member".into());
            };
            let goal = callable_requirement(&mut facts, call.original, &pay)?
                .ok_or("actual full caller goal")?;
            assert_eq!(
                requested_callable(&mut facts, DefId(item.0), call.actual, Some(goal), &pay)?,
                call.original
            );
        }
        assert_eq!(facts.dispatch_dependencies.work.selections, selections);
        if let Some((old_work, old_bytes)) = previous {
            assert!(work.get() <= old_work * 5);
            assert!(bytes.get() <= old_bytes * 5);
        }
        eprintln!(
            "selected callable width={width} work={} attempted_bytes={}",
            work.get(),
            bytes.get()
        );
        previous = Some((work.get(), bytes.get()));
    }
    Ok(())
}
