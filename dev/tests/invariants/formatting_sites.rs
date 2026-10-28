//! Actual formatting frames authenticate each special site and preserve input identity.
use super::*;
use crate::resolve::expansion::Report;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn expanded() -> Result<Report> {
    expanded_arguments("format_args!(\"{:?} (1 << {:?})\", 1u32, 2u32)")
}
fn expanded_arguments(arguments: &str) -> Result<Report> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-format-macros/literal-complete-basic.rs",
    )?;
    let source = source.replace("format_args!(\"hello\")", arguments);
    Ok(crate::test_source::source_provider(
        &source,
        "formatting_sites",
    )?)
}
fn output_root(report: &Report) -> Result<NodeKey> {
    for (file, unit) in report.package.units.iter().enumerate() {
        for (node, actual) in unit.configured.ast.nodes.iter().enumerate() {
            if unit.configured.active[node]
                && matches!(
                    actual.kind,
                    Kind::LibraryValue {
                        kind: LibraryValue::FormatOutput { .. },
                        ..
                    }
                )
            {
                return Ok(NodeKey {
                    file: crate::resolve::unit::FileId(u32::try_from(file)?),
                    node: NodeId::new(u32::try_from(node)?),
                });
            }
        }
    }
    Err("missing actual formatting output".into())
}
#[test]
fn actual_format_sites_keep_original_inputs_and_reject_changed_root_site_member_and_child() -> Result
{
    let report = expanded()?;
    let root = output_root(&report)?;
    report.formatting_value(root)?;
    let unit = report.package.unit(root.file);
    let Kind::LibraryValue {
        kind: LibraryValue::FormatOutput { sites },
        ..
    } = unit.configured.ast.node(root.node).kind
    else {
        return Err("actual root".into());
    };
    let nodes = unit
        .configured
        .ast
        .list(crate::syntax::ast::List {
            start: sites.start,
            end: sites.end,
        })
        .to_vec();
    assert_eq!(nodes.len(), 7);
    for &node in &nodes[..5] {
        assert_eq!(report.formatting_root(NodeKey { node, ..root })?, root);
    }
    for &node in &nodes[5..] {
        let Kind::Group(input) = unit.configured.ast.node(node).kind else {
            return Err("input was not adopted".into());
        };
        let Kind::Literal(index) = unit.configured.ast.node(input).kind else {
            return Err("parsed input changed".into());
        };
        assert!(matches!(
            unit.literal(index).map_err(|error| error.message)?,
            crate::literal::Value::Integer(_)
        ));
        assert!(
            unit.configured.ast.node(input).span.start < unit.configured.ast.node(node).span.start
        );
        assert!(report.formatting_value(NodeKey { node, ..root }).is_err());
    }
    for opponent in 0..6 {
        let mut report = expanded()?;
        let unit = &mut report.package.units[root.file.0 as usize];
        match opponent {
            0 => {
                if let Kind::LibraryValue { value, .. } =
                    &mut unit.configured.ast.nodes[root.node.get() as usize].kind
                {
                    *value = Some(nodes[0]);
                }
            }
            1 => {
                unit.configured.ast.edges[sites.start as usize + 3] = nodes[2];
            }
            2 => {
                if let Kind::LibraryValue {
                    kind: LibraryValue::FormatConstructor { constructor, .. },
                    ..
                } = &mut unit.configured.ast.nodes[nodes[3].get() as usize].kind
                {
                    *constructor = FormatConstructor::Arguments;
                }
            }
            3 => {
                if let Kind::LibraryValue { value, .. } =
                    &mut unit.configured.ast.nodes[nodes[0].get() as usize].kind
                {
                    *value = Some(nodes[1]);
                }
            }
            4 => {
                unit.configured.ast.nodes[root.node.get() as usize].kind = Kind::LibraryValue {
                    kind: LibraryValue::FormatSuperLet { slot: 0 },
                    value: Some(nodes[0]),
                }
            }
            _ => {
                if let Kind::LibraryValue { value, .. } =
                    &mut unit.configured.ast.nodes[root.node.get() as usize].kind
                {
                    *value = None;
                }
            }
        }
        assert!(
            report
                .formatting_value(NodeKey {
                    node: nodes[3],
                    ..root
                })
                .is_err(),
            "opponent {opponent}"
        );
    }
    let report = expanded_arguments(
        "{ macro_rules! forward {($x:expr,$y:expr)=>{format_args!(\"{:?} (1 << {:?})\",$x,$y)}} forward!(1u32,2u32) }",
    )?;
    let forwarded = output_root(&report)?;
    report.formatting_value(forwarded)?;
    Ok(())
}

#[test]
fn formatting_site_authentication_is_bounded_across_shared_input_scaling() -> Result {
    let mut previous = None;
    for count in [8, 32, 128] {
        let template = "{:?}".repeat(count);
        let inputs = vec!["1u32"; count].join(",");
        let report = expanded_arguments(&format!("format_args!(\"{template}\",{inputs})"))?;
        let key = output_root(&report)?;
        let payer = &report
            .package
            .source_core()
            .ok_or("source provider")?
            .macros;
        let before = payer.usage();
        report.formatting_value(key)?;
        let after = payer.usage();
        let auth = after.0 - before.0;
        assert_eq!(after.1, before.1);
        if let Some(previous) = previous {
            assert_eq!(auth, previous);
        }
        previous = Some(auth);
        eprintln!(
            "format sites N={count} work={} bytes={} witnesses={} root_auth={auth} nodes={} edges={}",
            before.0,
            before.1,
            before.2,
            report.package.unit(key.file).configured.ast.nodes.len(),
            report.package.unit(key.file).configured.ast.edges.len()
        );
    }
    Ok(())
}

#[test]
fn mixed_constructor_inventory_authenticates_pair_members_and_rejects_changed_trait() -> Result {
    use crate::support::library_types::FormatConstructor;
    let report = expanded_arguments("{format_args!(\"{1}|{0:?}|{1}|{1:?}|{0:?}\",1u32,2u32)}")?;
    let root = output_root(&report)?;
    report.formatting_value(root)?;
    let Kind::LibraryValue {
        kind: LibraryValue::FormatOutput { sites },
        ..
    } = report
        .package
        .unit(root.file)
        .configured
        .ast
        .node(root.node)
        .kind
    else {
        return Err("actual output".into());
    };
    let nodes = report
        .package
        .unit(root.file)
        .configured
        .ast
        .list(crate::syntax::ast::List {
            start: sites.start,
            end: sites.end,
        })
        .to_vec();
    assert_eq!(nodes.len(), 8);
    for &node in &nodes[..6] {
        report.formatting_value(NodeKey { node, ..root })?;
    }
    let mut changed =
        expanded_arguments("{format_args!(\"{1}|{0:?}|{1}|{1:?}|{0:?}\",1u32,2u32)}")?;
    let name = changed.package.names.intern("new_debug")?;
    let unit = &mut changed.package.units[root.file.0 as usize];
    let Kind::LibraryValue {
        kind:
            LibraryValue::FormatConstructor {
                constructor,
                name: actual,
                ..
            },
        ..
    } = &mut unit.configured.ast.nodes[nodes[3].get() as usize].kind
    else {
        return Err("actual ctor".into());
    };
    *constructor = FormatConstructor::Debug;
    *actual = name;
    assert!(
        changed
            .formatting_value(NodeKey {
                node: nodes[3],
                ..root
            })
            .is_err()
    );
    Ok(())
}

#[test]
fn lower_hex_and_option_pair_sites_keep_constant_root_authentication_and_owned_spelling() -> Result
{
    for count in [8, 32, 128] {
        let template = (0..count)
            .map(|i| format!("{{{i}:#034x}}|{{{i}:?}}|{{{i}}}"))
            .collect::<Vec<_>>()
            .join(";");
        let args = vec!["1u32"; count].join(",");
        let report = expanded_arguments(&format!("format_args!(\"{template}\",{args})"))?;
        let key = output_root(&report)?;
        let payer = &report
            .package
            .source_core()
            .ok_or("source provider")?
            .macros;
        let before = payer.usage();
        report.formatting_value(key)?;
        let after = payer.usage();
        assert_eq!(after.0 - before.0, 22);
        assert_eq!(after.1, before.1);
        eprintln!(
            "static options N={count} work={} bytes={} root_auth={}",
            before.0,
            before.1,
            after.0 - before.0
        );
    }
    let mut report = expanded_arguments("format_args!(\"{:#034x}\",1u32)")?;
    let root = output_root(&report)?;
    let Kind::LibraryValue {
        kind: LibraryValue::FormatOutput { sites },
        ..
    } = report
        .package
        .unit(root.file)
        .configured
        .ast
        .node(root.node)
        .kind
    else {
        return Err("actual root".into());
    };
    let node = report.package.unit(root.file).configured.ast.edges[sites.start as usize + 3];
    report.formatting_value(NodeKey { node, ..root })?;
    let name = report.package.names.intern("new_debug")?;
    let unit = &mut report.package.units[root.file.0 as usize];
    let Kind::LibraryValue {
        kind:
            LibraryValue::FormatConstructor {
                constructor,
                name: actual,
                ..
            },
        ..
    } = &mut unit.configured.ast.nodes[node.get() as usize].kind
    else {
        return Err("actual constructor".into());
    };
    *constructor = FormatConstructor::Debug;
    *actual = name;
    assert!(report.formatting_value(NodeKey { node, ..root }).is_err());
    Ok(())
}
