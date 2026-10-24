//! An unadjusted pending pointer fact must retain its ordinary local-value read.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn pending_pointer_and_local_calls_keep_the_real_receiver_read() -> Result {
    let report = crate::test_source::resolve("fn run(f:fn(u8)->u8)->u8{let g=f;g(7)}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "run"
        })
        .copied()
        .ok_or("run")?;
    let owner = function.body.ok_or("body")?;
    let ast = &report.package.unit(owner.file).configured.ast;
    let (node, callee) = ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| {
            if let crate::syntax::ast::Kind::Call { callee, .. } = node.kind {
                Some((index, callee))
            } else {
                None
            }
        })
        .ok_or("actual local call")?;
    let key = NodeKey {
        node: NodeId::new(u32::try_from(node)?),
        ..owner
    };
    let receiver = facts
        .node(NodeKey {
            node: callee,
            ..owner
        })
        .and_then(|fact| fact.ty)
        .ok_or("pointer")?;
    assert!(matches!(
        facts.types.kind(receiver),
        Some(crate::types::Kind::Function { .. })
    ));
    for retained in [false, true] {
        if retained {
            facts.invocations.insert(
                key,
                Invocation {
                    kind: None,
                    dereferences: 0,
                    borrow: None,
                    receiver,
                },
            );
        }
        let body = super::super::function_body(
            &report,
            &facts,
            &mut super::super::Scratch::new(&facts),
            &function,
            None,
        )?;
        let callee = body
            .values
            .iter()
            .find_map(|value| {
                if let Rvalue::Call { callee, .. } = value.kind {
                    Some(callee)
                } else {
                    None
                }
            })
            .ok_or("ordinary Call")?;
        assert!(matches!(
            body.values[callee.0 as usize].kind,
            Rvalue::Read(_)
        ));
    }
    Ok(())
}
