//! Operator coercions retain real source duties without comparison borrow state.
use crate::{
    lex::token::Punctuation,
    resolve::{expansion::Report, unit::NodeKey},
    sema::{bodies::Work, model::Declarations},
    syntax::ast::Kind,
};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/operator-rhs";

fn analyze(name: &str) -> Result<(Report, Declarations)> {
    let source = crate::test_source::fixture(&format!("{PACKET}/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "operator_rhs")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
fn operator(report: &Report, facts: &Declarations) -> Result<(NodeKey, NodeKey, NodeKey)> {
    facts
        .methods
        .calls
        .keys()
        .find_map(|&site| {
            let syntax = &report.package.unit(site.file).configured.ast;
            match syntax.nodes[site.node.get() as usize].kind {
                Kind::Binary {
                    op: Punctuation::Plus,
                    left,
                    right,
                } => Some((
                    site,
                    NodeKey { node: left, ..site },
                    NodeKey {
                        node: right,
                        ..site
                    },
                )),
                _ => None,
            }
        })
        .ok_or_else(|| "actual source addition".into())
}

#[test]
fn selected_arithmetic_owns_real_rhs_adjustments_without_comparison_borrows() -> Result {
    for name in ["borrowed", "mutable", "nested"] {
        let (report, facts) = analyze(name)?;
        let (site, left, right) = operator(&report, &facts)?;
        let selected = facts.methods.calls[&site];
        assert_eq!(
            report
                .package
                .names
                .get(report.resolution.definitions[selected.method.0 as usize].name),
            "add"
        );
        assert!(selected.borrow.is_none());
        assert_eq!(selected.dereferences, 0);
        assert!(facts.methods.comparison_borrow(site).is_none());
        let rhs = facts.node(right).ok_or("RHS source facts")?;
        let adjustment = rhs.adjustment.ok_or("real RHS coercion")?;
        assert_ne!(rhs.ty, Some(adjustment.target));
        assert!(
            facts
                .reference_adjustments
                .get(&right)
                .is_some_and(|a| a.plan.is_some())
        );
        // Region-free identity relations are deliberately absent from the
        // lifetime ledger. The actual selected callable still owns this Self.
        let lhs = facts
            .node(left)
            .and_then(|node| node.ty)
            .ok_or("LHS type")?;
        assert_eq!(facts.types.contains_regions(lhs), Some(false));
        assert_eq!(selected.receiver, lhs);
        let Some(crate::types::Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(selected.callable)
        else {
            return Err("selected operator callable identity".into());
        };
        assert_eq!(item.0, selected.method.0);
        let Some(crate::types::Kind::Function { parameters, .. }) = facts.types.kind(signature)
        else {
            return Err("selected operator signature".into());
        };
        assert_eq!(
            facts
                .types
                .list(parameters)
                .and_then(|p| p.first())
                .copied(),
            Some(lhs)
        );
        let rhs = facts
            .coercions
            .iter()
            .find(|row| row.source == right)
            .ok_or("RHS proof")?;
        assert!(
            facts
                .functions
                .iter()
                .any(|function| function.body == Some(rhs.body))
        );
    }
    assert_eq!(size_of::<Work>(), 24);
    Ok(())
}

#[test]
fn independent_operand_lifetimes_retain_two_actual_coercion_sites() -> Result {
    let (report, facts) = analyze("independent-input-lifetimes")?;
    let (_, left, right) = operator(&report, &facts)?;
    let original = |node| {
        let ty = facts
            .node(node)
            .and_then(|fact| fact.ty)
            .ok_or("operand type")?;
        let Some(crate::types::Kind::Reference { region, .. }) = facts.types.kind(ty) else {
            return Err("original operand reference".into());
        };
        Ok::<_, Box<dyn std::error::Error>>(region)
    };
    assert_ne!(original(left)?, original(right)?);
    for source in [left, right] {
        assert!(
            facts
                .coercions
                .iter()
                .any(|row| row.source == source && row.actual != row.expected)
        );
    }
    Ok(())
}

#[test]
fn an_arithmetic_continuation_rejects_another_operand_before_mutation() -> Result {
    let (report, mut facts) = analyze("borrowed")?;
    let (site, left, _) = operator(&report, &facts)?;
    let mut checker = super::Checker::new(&report, &mut facts);
    let expected = checker.variable(site.node, crate::types::infer::Domain::Any)?;
    let before = (
        checker.state.adjustments.len(),
        checker.state.call_types.obligations.len(),
        checker.state.call_types.comparison_borrows.len(),
    );
    let error = checker
        .coerce_binary_rhs(site.node, left.node, expected)
        .err()
        .ok_or("another operand was accepted")?;
    assert!(error.message.contains("changed its actual operand"));
    assert_eq!(
        before,
        (
            checker.state.adjustments.len(),
            checker.state.call_types.obligations.len(),
            checker.state.call_types.comparison_borrows.len(),
        )
    );
    Ok(())
}

fn repeated(prefix: &str, count: usize, noise: bool) -> Result<String> {
    let mut source = prefix.to_owned();
    for index in 0..count {
        writeln!(source, "pub fn plus_{index}(x:Text,y:&Owned)->Text{{x+y}}")?;
    }
    if noise {
        for index in 0..64 {
            writeln!(
                source,
                "pub struct U{index};impl<'a>Add<&'a View> for U{index}{{type Output=Self;fn add(self,_:&'a View)->Self{{self}}}}"
            )?;
        }
    }
    Ok(source)
}

#[test]
fn selected_rhs_work_scales_with_operations_without_scanning_unrelated_impls() -> Result {
    let prefix = crate::test_source::fixture(&format!("{PACKET}/prefix.rs"))?;
    let mut previous = None;
    let mut unrelated = None;
    for count in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let source = repeated(&prefix, count, noise)?;
            let report = crate::test_source::source_provider(&source, "operator_rhs")?;
            let facts = crate::pipeline::analyze_source(&report)?.facts;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            assert!(work.into_iter().all(|value| value > 0));
            if noise {
                let added: Vec<_> = std::iter::zip(ordinary, work)
                    .map(|(old, actual)| actual.checked_sub(old))
                    .collect();
                assert!(added.iter().all(Option::is_some));
                assert_eq!(*unrelated.get_or_insert_with(|| added.clone()), added);
            } else {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(
                            actual <= 4 * old + 2048,
                            "calls={count}: {before:?}->{work:?}"
                        );
                    }
                }
                previous = Some(work);
                ordinary = work;
            }
        }
    }
    Ok(())
}
