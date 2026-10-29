//! Internal endpoint permission belongs to real generated nodes and one report.
use super::*;
use std::fmt::Write as _;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn source() -> Result<String, Box<dyn std::error::Error>> {
    Ok(crate::test_source::fixture(
        "pa31/tests/spec/source-pattern-endpoints/authored-runtime.rs",
    )?)
}
fn keys(report: &Report) -> Result<Vec<NodeKey>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for (file, unit) in report.package.units.iter().enumerate() {
        for (index, node) in unit.configured.ast.nodes.iter().enumerate() {
            if unit.configured.active[index] && matches!(node.kind, Kind::PatternEndpoint { .. }) {
                result.push(NodeKey {
                    file: crate::support::source::FileId(u32::try_from(file)?),
                    node: NodeId::new(u32::try_from(index)?),
                });
            }
        }
    }
    Ok(result)
}
fn assert_endpoint_owners(report: &Report, foreign: &Report, keys: &[NodeKey]) -> TestResult {
    assert!(!keys.is_empty());
    let mut helpers = [false; 3];
    for &key in keys {
        let site = report.pattern_endpoint(key)?;
        let ast = &report.package.unit(key.file).configured.ast;
        let Kind::PatternEndpoint { gate } = ast.node(key.node).kind else {
            return Err("actual endpoint node".into());
        };
        let row = ast.endpoint(gate).ok_or("real endpoint row")?;
        let context = row.invocation.ok_or("real builtin invocation")?;
        let call = report
            .invocations
            .get(context.get() as usize - 1)
            .ok_or("selected invocation")?;
        // Derives can copy a field's macro. Each copy owes its own actual root,
        // operand edge and selected source invocation, not a guessed census.
        assert_eq!(row.node, key.node);
        assert_eq!(
            site.base(),
            NodeKey {
                node: row.base,
                ..key
            }
        );
        assert_eq!(
            call.node,
            NodeKey {
                node: row.root,
                ..key
            }
        );
        helpers[match site.helper() {
            Helper::Minimum => 0,
            Helper::Maximum => 1,
            Helper::Predecessor => 2,
        }] = true;
        assert!(site.allows(report, key, "pattern_type_range_trait"));
        assert!(!site.allows(report, key, "const_trait_impl"));
        assert!(!site.allows(foreign, key, "pattern_type_range_trait"));
    }
    assert!(helpers.into_iter().all(|found| found));
    Ok(())
}
#[test]
fn generated_endpoint_permission_rejects_copied_nodes_changed_roots_and_foreign_reports()
-> TestResult {
    let input = source()?;
    let mut report = crate::test_source::source_provider(&input, "endpoint_origins")?;
    let foreign = crate::test_source::source_provider(&input, "endpoint_foreign")?;
    let keys = keys(&report)?;
    assert_endpoint_owners(&report, &foreign, &keys)?;
    let key = keys[0];
    let Kind::PatternEndpoint { gate } = report
        .package
        .unit(key.file)
        .configured
        .ast
        .node(key.node)
        .kind
    else {
        return Err("actual generated endpoint".into());
    };
    let row = *report
        .package
        .unit(key.file)
        .configured
        .ast
        .endpoint(gate)
        .ok_or("actual endpoint row")?;
    let original = report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[row.root.get() as usize];
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[row.root.get() as usize]
        .kind = Kind::TypePattern {
        base: key.node,
        constraint: crate::syntax::ast::TypeConstraint::Pattern(row.range),
    };
    assert!(report.pattern_endpoint(key).is_err());
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[row.root.get() as usize] = original;
    let original = report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[row.base.get() as usize];
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[row.base.get() as usize]
        .kind = Kind::PatternEndpoint { gate };
    assert!(
        report
            .pattern_endpoint(NodeKey {
                node: row.base,
                ..key
            })
            .is_err()
    );
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[row.base.get() as usize] = original;
    assert!(report.pattern_endpoint(key).is_ok());
    let producer = report.lint_producer().ok_or("source payer")?;
    assert!(producer.diagnostic_payload(key, 10_000_000, 0).is_err());
    let before = producer.work().steps;
    assert!(report.pattern_endpoint(key).is_err());
    assert_eq!(producer.work().steps, before + 24);
    Ok(())
}
#[test]
fn sparse_endpoint_rows_scale_without_growing_ordinary_nodes_or_bodies() -> TestResult {
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<Endpoint>(), 32);
    assert_eq!(size_of::<crate::syntax::gate_sites::Declarations>(), 112);
    assert_eq!(size_of::<crate::syntax::gate_sites::Prefix>(), 12);
    println!(
        "endpoint cold widths: row={} gates={} prefix={} astKind={} typeKind={} constraint={}",
        size_of::<Endpoint>(),
        size_of::<crate::syntax::gate_sites::Declarations>(),
        size_of::<crate::syntax::gate_sites::Prefix>(),
        size_of::<Kind>(),
        size_of::<crate::types::Kind>(),
        size_of::<crate::types::Constraint>()
    );
    let input = source()?;
    let prefix = &input[..input
        .find("pub mod niche")
        .ok_or("authored consumer boundary")?];
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = prefix.to_owned();
        for n in 0..width {
            writeln!(source, "type Range{n}=pattern_type!(u8 is 1..);")?;
        }
        let report = crate::test_source::source_provider(&source, "endpoint_width")?;
        assert_eq!(keys(&report)?.len(), width);
        let before = report.lint_producer().ok_or("source payer")?.work();
        for key in keys(&report)? {
            report.pattern_endpoint(key)?;
        }
        let after = report.lint_producer().ok_or("source payer")?.work();
        assert_eq!(after.steps - before.steps, 24 * width);
        assert_eq!(after.bytes, before.bytes);
        let bytes = report
            .package
            .units
            .iter()
            .map(|u| u.configured.ast.gate_sites_bytes())
            .sum::<usize>();
        if let Some(old) = previous {
            assert!(bytes <= old * 5);
        }
        previous = Some(bytes);
    }
    Ok(())
}
