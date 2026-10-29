//! Calls retain actual callee/generic/argument identities, never const-execution permission.
use super::constant_tests::{selected, typed};
use super::*;
use crate::{lex::token::Punctuation, syntax::ast::TypeFlavor};
use std::cell::Cell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn generic_pointer_call(
    ast: &Syntax<StoredValue>,
    callee: NodeId,
    names: &[&str],
    source: &str,
) -> TestResult {
    let Kind::Path {
        absolute: false,
        segments,
    } = ast.node(callee).kind
    else {
        return Err(format!("actual callee {:?}", ast.node(callee).kind).into());
    };
    let parts = ast.list(segments);
    assert_eq!(parts.len(), names.len());
    for (index, (&part, name)) in parts.iter().zip(names).enumerate() {
        let base = if index + 1 == names.len() {
            let Kind::Applied {
                base,
                arguments,
                output: None,
                parenthesized: false,
            } = ast.node(part).kind
            else {
                return Err(format!("actual applied segment {:?}", ast.node(part).kind).into());
            };
            let [pointer] = ast.list(arguments) else {
                return Err("actual pointer argument".into());
            };
            let Kind::Type {
                flavor: TypeFlavor::PointerConst,
                elements,
            } = ast.node(*pointer).kind
            else {
                return Err("actual pointer type".into());
            };
            let [unit] = ast.list(elements) else {
                return Err("actual pointer child".into());
            };
            let Kind::Type {
                flavor: TypeFlavor::Tuple,
                elements,
            } = ast.node(*unit).kind
            else {
                return Err("actual unit type".into());
            };
            assert!(ast.list(elements).is_empty());
            base
        } else {
            part
        };
        assert!(matches!(ast.node(base).kind, Kind::Name(..)));
        let span = ast.node(base).span;
        assert_eq!(&source[span.start as usize..span.end as usize], *name);
    }
    Ok(())
}
#[test]
fn unchanged_type_id_field_has_the_actual_applied_call_child() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/derive-call-counts/original-type-id.rs")?;
    let (unit, root) = typed(&source, TypeFlavor::Array)?;
    let ast = &unit.configured.ast;
    let Kind::Type { elements, .. } = ast.node(root).kind else {
        return Err("array field".into());
    };
    let [_, count] = ast.list(elements) else {
        return Err("array children".into());
    };
    let Kind::Binary {
        op: Punctuation::Slash,
        right,
        ..
    } = ast.node(*count).kind
    else {
        return Err("actual division".into());
    };
    let Kind::Call { callee, arguments } = ast.node(right).kind else {
        return Err(format!("actual right child {:?}", ast.node(right).kind).into());
    };
    assert!(ast.list(arguments).is_empty());
    generic_pointer_call(ast, callee, &["size_of"], &source)?;
    println!(
        "actual TypeId count={:?} call={:?} callee={:?} source={}",
        ast.node(*count).kind,
        ast.node(right).kind,
        ast.node(callee).kind,
        &source[ast.node(right).span.start as usize..ast.node(right).span.end as usize]
    );
    let copied = field(ast, root, &|_, _| Ok(())).map_err(|e| e.message)?;
    let copied_callee = copied
        .ast
        .nodes
        .iter()
        .find_map(|n| {
            if let Kind::Call { callee, .. } = n.kind {
                Some(callee)
            } else {
                None
            }
        })
        .ok_or("copied actual call")?;
    generic_pointer_call(&copied.ast, copied_callee, &["size_of"], &source)?;
    assert_eq!(
        copied
            .ast
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, Kind::Call { .. }))
            .count(),
        1
    );
    assert!(
        matches!(copied.ast.literals.as_slice(),[StoredValue::Integer(value)] if value.magnitude==16)
    );
    Ok(())
}
#[test]
fn callee_arguments_generic_types_and_selected_work_remain_exact() -> TestResult {
    use std::fmt::Write as _;
    let mut costs = Vec::new();
    for count in [8, 32, 128] {
        let mut shape = String::from("struct Holder((");
        for _ in 0..count {
            shape.push_str("[u8;module::take::<*const ()>(2usize,0usize)],[*const ();16/core::mem::size_of::<*const ()>()],");
        }
        shape.push_str("));");
        let mut measured = None;
        for noise in [0, 1024] {
            let mut source = String::new();
            for i in 0..noise {
                write!(&mut source, "const NOISE{i}:usize={i};")?;
            }
            source.push_str(&shape);
            let (unit, root) = selected(&source)?;
            let paid = Cell::new((0_usize, 0_usize));
            let copied = field(&unit.configured.ast, root, &|w, b| {
                let (old, bytes) = paid.get();
                paid.set((old + w, bytes + b));
                Ok(())
            })
            .map_err(|e| e.message)?;
            let calls: Vec<_> = copied
                .ast
                .nodes
                .iter()
                .filter_map(|n| {
                    if let Kind::Call { callee, arguments } = n.kind {
                        Some((callee, arguments))
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(calls.len(), 2 * count);
            assert_eq!(copied.ast.literals.len(), 3 * count);
            assert_eq!(
                calls
                    .iter()
                    .filter(|(_, args)| copied.ast.list(*args).len() == 2)
                    .count(),
                count
            );
            assert_eq!(
                calls
                    .iter()
                    .filter(|(_, args)| copied.ast.list(*args).is_empty())
                    .count(),
                count
            );
            for (callee, arguments) in calls {
                let names = if copied.ast.list(arguments).is_empty() {
                    &["core", "mem", "size_of"][..]
                } else {
                    &["module", "take"][..]
                };
                generic_pointer_call(&copied.ast, callee, names, &source)?;
                for (index, child) in copied.ast.list(arguments).iter().enumerate() {
                    let Kind::Literal(value) = copied.ast.node(*child).kind else {
                        return Err("copied argument literal".into());
                    };
                    assert!(
                        matches!(copied.ast.literals[value as usize],StoredValue::Integer(value) if value.magnitude==[2,0][index])
                    );
                }
            }
            if let Some(expected) = measured {
                assert_eq!(paid.get(), expected);
            } else {
                measured = Some(paid.get());
            }
            println!(
                "calls={} noise={noise} nodes={} work/bytes={:?}",
                2 * count,
                copied.ast.nodes.len(),
                paid.get()
            );
        }
        costs.push(measured.ok_or("paid cost")?);
    }
    assert!(costs[2].0 - costs[1].0 <= 4 * (costs[1].0 - costs[0].0) + 32);
    assert!(costs[2].1 <= 5 * costs[1].1);
    assert_eq!(size_of::<Node>(), 36);
    Ok(())
}
#[test]
fn malformed_call_children_and_failed_payment_publish_no_selected_tree() -> TestResult {
    let (unit, root) = selected("struct S(([u8;take::<u8>(2usize,0usize)],));")?;
    let ast = &unit.configured.ast;
    let attempts = Cell::new(0);
    assert!(
        field(ast, root, &|_, _| {
            attempts.set(attempts.get() + 1);
            if attempts.get() == 8 {
                Err(error(0, "declared call denial"))
            } else {
                Ok(())
            }
        })
        .is_err()
    );
    assert_eq!(attempts.get(), 8);
    for invalid in [0, 1, 2] {
        let mut copied = field(ast, root, &|_, _| Ok(())).map_err(|e| e.message)?;
        let call = copied
            .ast
            .nodes
            .iter_mut()
            .find(|n| matches!(n.kind, Kind::Call { .. }))
            .ok_or("call node")?;
        let Kind::Call { callee, arguments } = &mut call.kind else {
            return Err("call".into());
        };
        if invalid == 0 {
            *callee = NodeId::new(u32::MAX);
        } else if invalid == 1 {
            arguments.end = u32::MAX;
        } else {
            copied.ast.edges[arguments.start as usize] = NodeId::new(u32::MAX);
        }
        let work = Cell::new(0);
        let result = field(&copied.ast, copied.ast.root, &|w, _| {
            work.set(work.get() + w);
            Ok(())
        });
        assert!(result.is_err());
        assert!(work.get() > 0);
    }
    let complete = field(ast, root, &|_, _| Ok(())).map_err(|e| e.message)?;
    assert_eq!(
        complete
            .ast
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, Kind::Call { .. }))
            .count(),
        1
    );
    Ok(())
}
