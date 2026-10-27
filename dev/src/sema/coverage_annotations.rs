//! Coverage syntax is validated even though this driver never enables instrumentation.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::{Delimiter, Forest, TreeKind},
    },
};
pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper {
        return Err(error("coverage cannot use an unsafe wrapper"));
    }
    let Input::Delimited(group) = attribute.input else {
        return Err(error("coverage requires exactly on or off"));
    };
    let Some(tree) = forest.tree(group) else {
        return Err(error("coverage lost its parsed group"));
    };
    let TreeKind::Group {
        delimiter: Delimiter::Parenthesis,
        children,
    } = tree.kind
    else {
        return Err(error("coverage requires parenthesized on or off"));
    };
    let unit = report.package.unit(declaration.file);
    if children.end != children.start + 1
        || !matches!(
            forest.name(unit.source.text(), children.start),
            Some("on" | "off")
        )
    {
        return Err(error("coverage requires exactly on or off"));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("coverage lost its name token"))?
        .span;
    if !report
        .package
        .requested_coverage_attribute()
        .map_err(error)?
        && !allowances.allows_at(
            declaration.file,
            name,
            &FeatureName::new("coverage_attribute").map_err(error)?,
        )?
    {
        return Err(error(
            "coverage requires coverage_attribute or its actual literal allowance",
        ));
    }
    let valid = match unit.configured.ast.node(declaration.node).kind {
        Kind::Crate(_) | Kind::Closure { .. } => true,
        Kind::Item(index) => matches!(
            unit.configured.ast.items[index as usize].kind,
            ItemKind::Function { body: Some(_), .. }
                | ItemKind::Impl { .. }
                | ItemKind::Module { .. }
        ),
        _ => false,
    };
    if !valid {
        return Err(error(
            "coverage requires a function body, closure, implementation or module",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/invariants/coverage_requests.rs"]
mod tests;
