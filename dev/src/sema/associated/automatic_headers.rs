//! Ordinary automatic declarations retain real feature and authored-header checks.
use crate::{
    resolve::{bindings::FeatureName, expansion::Report, package::Error, unit::NodeKey},
    syntax::{ast::Kind, item::ItemKind},
};

pub(super) fn validate(report: &Report, source: NodeKey) -> Result<(), Error> {
    let unit = report.package.unit(source.file);
    let node = unit.configured.ast.node(source.node);
    let error = |message| Error::at(source.file, node.span.start, message);
    let Kind::Item(index) = node.kind else {
        return Err(error("automatic trait has no actual source declaration"));
    };
    let item = unit.configured.ast.items[index as usize];
    let ItemKind::Trait {
        auto_: true,
        constant,
        bounds,
        items,
        ..
    } = item.kind
    else {
        return Err(error("automatic trait source kind changed"));
    };
    if !report.package.requested_auto_traits().map_err(error)? {
        let feature = FeatureName::new("auto_traits").map_err(error)?;
        if !report
            .macro_allowances()
            .allows_at(source.file, node.span, &feature)?
        {
            return Err(error(
                "automatic traits require their actual auto_traits feature request",
            ));
        }
    }
    let active = |list| {
        unit.configured
            .ast
            .list(list)
            .iter()
            .any(|id| unit.configured.active[id.get() as usize])
    };
    if constant {
        return Err(error("automatic traits cannot be const"));
    }
    if active(item.generics) {
        return Err(error("automatic traits cannot have generic parameters"));
    }
    if active(bounds) || active(item.where_clause) {
        return Err(error(
            "automatic traits cannot have supertrait or where bounds",
        ));
    }
    if active(items) {
        return Err(error("automatic traits cannot declare associated members"));
    }
    Ok(())
}
