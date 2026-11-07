//! Root `#[macro_export]` definitions publish their authenticated body trees.
use super::{DefKind, Error, Report, error};
use crate::{
    resolve::{
        expansion::{Body, Definition},
        model::{Access, Binding, Namespace, Origin},
        unit::NodeKey,
    },
    support::{
        crates::exported_macros::{Macro, Macros},
        hash::HashMap,
    },
    syntax::{ast::Kind as Syntax, item::ItemKind},
};

/// A direct public macro binding in the crate root is a `#[macro_export]` row.
pub(super) fn exported(report: &Report, namespace: Namespace, binding: &Binding) -> bool {
    namespace == Namespace::Macro
        && binding.origin == Origin::Direct
        && binding.access == Access::Public
        && report.resolution.definitions[binding.target.0 as usize].kind == DefKind::Macro
}

pub(super) fn collect(report: &Report) -> Result<Option<Macros>, Error> {
    let root = report.resolution.module_scopes[0];
    let mut rows = Vec::new();
    let mut sites = None;
    for (name, namespace, binding) in report.resolution.bindings.scope(root) {
        if !exported(report, namespace, binding) {
            continue;
        }
        let sites = sites.get_or_insert_with(|| {
            report
                .definitions
                .iter()
                .enumerate()
                .filter_map(|(index, definition)| definition.node.map(|node| (node, index)))
                .collect::<HashMap<_, _>>()
        });
        let node = report.resolution.definitions[binding.target.0 as usize]
            .node
            .ok_or_else(|| error("exported macro has no source definition"))?;
        let definition = sites
            .get(&node)
            .map(|&index| &report.definitions[index])
            .ok_or_else(|| error("missing exported macro definition"))?;
        rows.push(row(
            report,
            report.package.names.get(name),
            definition,
            node,
        )?);
    }
    if rows.is_empty() {
        return Ok(None);
    }
    rows.sort_unstable_by(|left, right| left.name().cmp(right.name()));
    Macros::new(rows).map(Some).map_err(error)
}

fn row(
    report: &Report,
    name: &str,
    definition: &Definition,
    node: NodeKey,
) -> Result<Macro, Error> {
    if definition.introduction.0 != 0 {
        return Err(error(
            "macro-generated exported macros are outside the macro metadata profile",
        ));
    }
    if definition.has_definition_permissions() {
        return Err(error(
            "definition-site macro permissions are outside the macro metadata profile",
        ));
    }
    let Body::Declarative(compiled) = &definition.body else {
        return Err(error(
            "builtin and scoped exported macros are outside the macro metadata profile",
        ));
    };
    let unit = report.package.unit(node.file);
    let Syntax::Item(index) = unit.configured.ast.node(node.node).kind else {
        return Err(error("exported macro definition is not an item"));
    };
    let ItemKind::MacroRules { capture } = unit.configured.ast.items[index as usize].kind else {
        return Err(error("exported macro definition is not macro_rules"));
    };
    let forest = &unit.configured.ast.captures[capture as usize];
    let (image, trees) =
        crate::resolve::crates::macro_image(unit.source.text(), forest).map_err(error)?;
    let rules = u32::try_from(compiled.definition.rules.len())
        .map_err(|_| error("exported macro has too many rules"))?;
    Macro::new(
        name.to_owned(),
        definition.edition,
        definition.local_inner_macros,
        rules,
        image,
        trees,
    )
    .map_err(error)
}
