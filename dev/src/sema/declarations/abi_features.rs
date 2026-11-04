//! ABI headers share one macro-authority session; bodies are never rescanned.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::Report,
        package::{Error, Package},
        unit::FileId,
    },
    syntax::{
        ast::{Kind, NodeId},
        item::ItemKind,
        token_tree::Span,
    },
};

pub(super) fn validate(report: &Report) -> Result<(), Error> {
    if report
        .package
        .requested_unadjusted_abi()
        .map_err(Error::unlocated)?
    {
        return Ok(());
    }
    let feature = FeatureName::new("abi_unadjusted").map_err(Error::unlocated)?;
    let mut session = report.macro_allowances();
    visit(&report.package, |file, span| {
        if session.allows_at(file, span, &feature)? {
            return Ok(());
        }
        Err(Error::at(
            file,
            span.start,
            "unadjusted ABI requires abi_unadjusted or its actual macro allowance",
        ))
    })
}

fn visit(
    package: &Package,
    mut check: impl FnMut(FileId, Span) -> Result<(), Error>,
) -> Result<(), Error> {
    for (file, unit) in package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(u32::try_from(file).map_err(|_| Error::unlocated("ABI file overflow"))?);
        let headers = unit
            .configured
            .ast
            .items
            .iter()
            .filter_map(|item| match item.kind {
                ItemKind::Function { abi, .. } => abi.literal(),
                ItemKind::ExternBlock { abi, .. } => abi,
                _ => None,
            });
        let pointers = unit
            .configured
            .ast
            .function_types
            .iter()
            .filter_map(|ty| ty.abi.literal());
        for node in headers.chain(pointers) {
            if let Some(span) =
                unadjusted(unit, node).map_err(|message| Error::at(file, 0, message))?
            {
                check(file, span)?;
            }
        }
    }
    Ok(())
}

fn unadjusted(
    unit: &crate::resolve::unit::Unit,
    node: NodeId,
) -> Result<Option<Span>, &'static str> {
    if unit.configured.active.get(node.get() as usize) != Some(&true) {
        return Ok(None);
    }
    let constructor = unit.configured.ast.node(node);
    let literal = if let Kind::Group(value) = constructor.kind {
        unit.configured.ast.node(value)
    } else {
        constructor
    };
    let Kind::Literal(index) = literal.kind else {
        return Err("ABI requires its parsed string literal");
    };
    Ok(matches!(unit.literal(index).map_err(|e| e.message)?, crate::literal::Value::String(value) if value.as_ref() == b"unadjusted")
        .then_some(constructor.span))
}
