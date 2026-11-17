//! An output-only impl parameter carries context without proving its premises.
use rustgm::{
    resolve::unit::FileId,
    sema::model::Scalar,
    types::{Primitive, TypeId},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/output-only-trait-inference";

fn accepts_u32_literal(name: &str, spelling: &str) -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/{name}.rs"))?;
    let report = crate::resolve::source_provider(&source, "output_parameters")?;
    let facts = rustgm::pipeline::analyze_source(&report)?.facts;
    let site = format!("Range({spelling})");
    let offset = source.rfind(&site).ok_or("literal call")? + 6;
    let ast = &report.package.unit(FileId(0)).configured.ast;
    let nodes = facts.nodes.first().ok_or("source facts")?;
    let literal = ast
        .nodes
        .iter()
        .zip(nodes)
        .filter_map(|(node, fact)| {
            (node.span.start as usize == offset
                && node.span.end as usize == offset + spelling.len()
                && fact.ty.is_some())
            .then_some(fact)
        })
        .collect::<Vec<_>>();
    assert_eq!(literal.len(), 1, "{name}: actual literal fact");
    let fact = literal.first().ok_or("literal fact")?;
    assert_eq!(fact.scalar, Some(Scalar::Integer(1)), "{name}");
    assert_eq!(fact.ty, Some(TypeId::primitive(Primitive::U32)), "{name}");
    Ok(())
}

#[test]
fn output_only_impl_parameter_infers_integer_before_fallback() -> Result {
    accepts_u32_literal("inferred", "1")
}

#[test]
fn output_only_impl_parameter_preserves_explicit_u32() -> Result {
    accepts_u32_literal("typed-u32", "1u32")
}

#[test]
fn output_only_impl_parameter_rejects_closed_item_mismatch() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/wrong-i32.rs"))?;
    let report = crate::resolve::source_provider(&source, "output_parameters")?;
    let error = rustgm::pipeline::analyze_source(&report)
        .err()
        .ok_or("closed i32/u32 mismatch accepted")?;
    let call = source.rfind("collect(Map(").ok_or("closed mismatch call")?;
    assert_eq!(error.file.source(), Some(FileId(0)));
    assert_eq!(error.span.start as usize, call, "{error}");
    assert!(
        error.message.starts_with("collecting i32")
            || error.message.starts_with("trait obligation requires "),
        "{error}"
    );
    Ok(())
}
