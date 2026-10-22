//! Declaration walks skip only subtrees whose nodes have no traversal effect.
use super::{is_inert, is_leaf};
use crate::syntax::ast::{Kind, NodeId, Syntax};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SOURCE: &str = r"
mod m { pub fn f<T: Copy>(x: T) -> T { x } }
struct S<'a> { r: &'a u8, f: fn(&u8) -> u8 }
enum E { A(bool, u64), B { v: u64 }, C }
macro_rules! m { ($e:expr) => { $e } }
fn g(x: (E, [u8; 2]), y: &dyn for<'b> Fn(&'b u8) -> u8) -> u64 {
    let s = S { r: &1, f: |b| *b };
    let v = match x {
        (E::A(true, n) | E::B { v: n }, [0, m] | [m, 0]) => n + m as u64,
        (E::A(_, n), _) => n,
        _ => 0,
    };
    let w = m!(v) + [v; 2][0] + (s.f)(&2) as u64 + Vec::<fn(u8)>::new().len() as u64;
    'l: loop { if w > v { break 'l; } else { continue 'l; } }
    let c = |k: u64| -> u64 { k * { let z = k; z } };
    c(w) + f(x.1[0]) as u64 + y(&3) as u64 + core::mem::size_of::<[u8; { 2 }]>() as u64
}
";

/// Kinds that a declaration walk acts on or that change its traversal state.
fn effective(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Item(_)
            | Kind::Macro { .. }
            | Kind::Block { .. }
            | Kind::Closure { .. }
            | Kind::Arm { .. }
            | Kind::For { .. }
            | Kind::If { .. }
            | Kind::While { .. }
            | Kind::Label { .. }
            | Kind::Bound { .. }
            | Kind::Where { .. }
            | Kind::FunctionType(_)
            | Kind::Generic(_)
            | Kind::Attributed { .. }
    )
}

fn subtree<L>(ast: &Syntax<L>, root: NodeId) -> Vec<NodeId> {
    let mut nodes = vec![root];
    let mut index = 0;
    while let Some(&node) = nodes.get(index) {
        ast.push_children(node, &mut nodes);
        index += 1;
    }
    nodes
}

#[test]
fn skipped_subtrees_have_no_declaration_walk_effect() -> TestResult {
    let report =
        crate::expand::crate_syntax::analyze(SOURCE, &crate::expand::config::Config::default())
            .map_err(|e| format!("{}: {}", e.offset, e.message))?;
    let ast = &report.ast;
    let (mut leaves, mut inert) = (0, 0);
    for index in 0..ast.nodes.len() {
        let node = NodeId::new(u32::try_from(index)?);
        let mut children = Vec::new();
        ast.push_children(node, &mut children);
        if is_leaf(&ast.node(node).kind) {
            leaves += 1;
            assert!(children.is_empty(), "leaf node {index} has children");
        }
        if is_inert(ast, node) {
            inert += 1;
            for descendant in subtree(ast, node) {
                assert!(
                    !effective(ast.node(descendant).kind),
                    "inert node {index} contains an effective node {}",
                    descendant.get()
                );
            }
        }
    }
    assert!(
        leaves > 0 && inert > leaves,
        "fixture exercises compound inert subtrees"
    );
    Ok(())
}
