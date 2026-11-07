//! Public macro bindings publish authenticated rows at their module placement.
//! Root `#[macro_export]` definitions publish their canonical body trees, and
//! `#[rustc_builtin_macro]` definitions publish their expander identity.
use super::{DefKind, Error, Report, error};
use crate::{
    resolve::{
        expansion::{Body, Definition},
        model::{Access, Binding, Namespace, Origin, ScopeId},
        package::ModuleId,
        unit::NodeKey,
    },
    support::{
        crates::exported_macros::{BuiltinMacro, Macro, Macros, Syntax},
        hash::{HashMap, HashSet},
    },
    syntax::{ast::Kind as AstKind, item::ItemKind},
};

/// A public binding of a local macro definition travels in the macro table,
/// never in the scalar public-name table.
pub(super) fn exported(report: &Report, namespace: Namespace, binding: &Binding) -> bool {
    let target = &report.resolution.definitions[binding.target.0 as usize];
    namespace == Namespace::Macro
        && binding.access == Access::Public
        && target.kind == DefKind::Macro
        && target.node.is_some()
}

/// The canonical path of a local module, from its crate root.
fn module_path(report: &Report, mut module: ModuleId) -> Result<Vec<String>, Error> {
    let mut names = Vec::new();
    while let Some(parent) = report.package.modules[module.0 as usize].parent {
        if names.len() >= 64 {
            return Err(error("exported macro placement exceeds profile budget"));
        }
        names.push(
            report
                .package
                .names
                .get(report.package.modules[module.0 as usize].name)
                .to_owned(),
        );
        module = parent;
    }
    names.reverse();
    Ok(names)
}

/// Visit every module reachable from the root through public direct module
/// bindings once, and every public macro binding in those modules.
pub(super) fn collect(report: &Report) -> Result<Option<Macros>, Error> {
    let root = report.resolution.module_scopes[0];
    let mut rows = Vec::new();
    let mut builtins = Vec::new();
    let mut sites = None;
    let mut work = vec![root];
    let mut seen = HashSet::from_iter([root]);
    while let Some(scope) = work.pop() {
        for (name, namespace, binding) in report.resolution.bindings.scope(scope) {
            let target = &report.resolution.definitions[binding.target.0 as usize];
            if binding.access != Access::Public {
                continue;
            }
            if namespace == Namespace::Type && target.kind == DefKind::Module {
                if let Some(members) = target.members.filter(|_| target.node.is_some())
                    && seen.insert(members)
                {
                    work.push(members);
                }
                continue;
            }
            if !exported(report, namespace, binding) {
                continue;
            }
            if binding.origin != Origin::Direct {
                return Err(error(
                    "macro re-exports are outside the macro metadata profile",
                ));
            }
            let sites = sites.get_or_insert_with(|| {
                report
                    .definitions
                    .iter()
                    .enumerate()
                    .filter_map(|(index, definition)| definition.node.map(|node| (node, index)))
                    .collect::<HashMap<_, _>>()
            });
            let node = target
                .node
                .ok_or_else(|| error("exported macro has no source definition"))?;
            let definition = sites
                .get(&node)
                .map(|&index| &report.definitions[index])
                .ok_or_else(|| error("missing exported macro definition"))?;
            let name = report.package.names.get(name);
            if definition.introduction.0 != 0 {
                return Err(error(
                    "macro-generated exported macros are outside the macro metadata profile",
                ));
            }
            if let Some(builtin) = definition.exported_builtin().map_err(error)? {
                let module = report.resolution.scopes[scope.0 as usize].module;
                builtins.push(
                    BuiltinMacro::new(
                        module_path(report, module)?,
                        name.to_owned(),
                        syntax(report, node)?,
                        builtin.identity.to_owned(),
                        builtin.kind,
                        builtin.helpers,
                    )
                    .map_err(error)?,
                );
            } else {
                rows.push(row(report, scope, name, definition, node)?);
            }
        }
    }
    if rows.is_empty() && builtins.is_empty() {
        return Ok(None);
    }
    rows.sort_unstable_by(|left, right| left.name().cmp(right.name()));
    builtins.sort_unstable_by(|left, right| {
        (left.placement(), left.name()).cmp(&(right.placement(), right.name()))
    });
    Macros::with_builtins(rows, builtins)
        .map(Some)
        .map_err(error)
}

fn syntax(report: &Report, node: NodeKey) -> Result<Syntax, Error> {
    let unit = report.package.unit(node.file);
    let AstKind::Item(index) = unit.configured.ast.node(node.node).kind else {
        return Err(error("exported macro definition is not an item"));
    };
    match unit.configured.ast.items[index as usize].kind {
        ItemKind::MacroRules { .. } => Ok(Syntax::MacroRules),
        ItemKind::DeclarativeMacro { .. } => Ok(Syntax::Macro),
        _ => Err(error(
            "exported macro definition is not a macro declaration",
        )),
    }
}

fn row(
    report: &Report,
    scope: ScopeId,
    name: &str,
    definition: &Definition,
    node: NodeKey,
) -> Result<Macro, Error> {
    let compiled = match &definition.body {
        Body::Declarative(compiled) => compiled,
        Body::Scoped { .. } => {
            return Err(error(
                "macros 2.0 definitions are outside the macro metadata profile",
            ));
        }
        Body::Builtin(_) => return Err(error("exported builtin macro lost its identity")),
    };
    if scope != report.resolution.module_scopes[0] {
        return Err(error("exported macro_rules rows belong to the crate root"));
    }
    let internal_unstable = definition.exported_internal_unstable().map_err(error)?;
    let unit = report.package.unit(node.file);
    let AstKind::Item(index) = unit.configured.ast.node(node.node).kind else {
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
    .and_then(|row| row.with_internal_unstable(internal_unstable))
    .map_err(error)
}
