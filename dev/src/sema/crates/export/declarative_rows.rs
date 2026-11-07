//! Public macro bindings publish authenticated rows at their module placement.
//! Root `#[macro_export]` definitions publish their canonical body trees, and
//! `#[rustc_builtin_macro]` definitions publish their expander identity. Every
//! further public binding of a definition, local or imported, is a re-export
//! row naming that definition row's identity.
use super::{DefKind, Error, Report, error};
use crate::{
    resolve::{
        expansion::{Body, Definition},
        model::{Access, Binding, DefId, Namespace, Origin, ScopeId},
        package::ModuleId,
        unit::NodeKey,
    },
    support::{
        crates::{
            CrateIdentity,
            exported_macros::{BuiltinMacro, DefinitionRow, Macro, Macros, ReexportMacro, Syntax},
        },
        hash::{HashMap, HashSet},
    },
    syntax::{ast::Kind as AstKind, item::ItemKind},
};
use std::rc::Rc;

/// Every public macro binding travels in the macro table, never in the scalar
/// public-name table; the macro table rejects unsupported ones by name.
pub(super) fn exported(namespace: Namespace, binding: &Binding) -> bool {
    namespace == Namespace::Macro && binding.access == Access::Public
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

/// One public macro binding in a module reachable from the root through
/// public direct module declarations.
struct Bound<'a> {
    placement: Rc<[String]>,
    name: &'a str,
    scope: ScopeId,
    target: DefId,
    direct: bool,
}
impl Bound<'_> {
    fn key(&self) -> (&[String], &str) {
        (&self.placement, self.name)
    }
}

/// Visit every module reachable through public direct module declarations
/// once, and gather its public macro bindings. A module that a public
/// re-export alone reaches has no public placement, so a public macro binding
/// there is rejected by name rather than dropped.
fn bindings(report: &Report) -> Result<Vec<Bound<'_>>, Error> {
    let resolution = &report.resolution;
    let root = resolution.module_scopes[0];
    let mut bound = Vec::new();
    let mut aliased = Vec::new();
    let mut work = vec![root];
    let mut seen = HashSet::from_iter([root]);
    while let Some(scope) = work.pop() {
        let module = resolution.scopes[scope.0 as usize].module;
        let placement: Rc<[String]> = module_path(report, module)?.into();
        for (name, namespace, binding) in resolution.bindings.scope(scope) {
            if binding.access != Access::Public {
                continue;
            }
            let target = &resolution.definitions[binding.target.0 as usize];
            if namespace == Namespace::Type && target.kind == DefKind::Module {
                let Some(members) = target.members.filter(|_| target.node.is_some()) else {
                    continue;
                };
                if binding.origin != Origin::Direct {
                    aliased.push(members);
                } else if seen.insert(members) {
                    work.push(members);
                }
                continue;
            }
            if namespace != Namespace::Macro {
                continue;
            }
            if binding.ambiguous {
                return Err(error(
                    "ambiguous public macro bindings are outside the macro metadata profile",
                ));
            }
            bound.push(Bound {
                placement: Rc::clone(&placement),
                name: report.package.names.get(name),
                scope,
                target: binding.target,
                direct: binding.origin == Origin::Direct,
            });
        }
    }
    // Modules reached only through public aliases, and their public children.
    while let Some(scope) = aliased.pop() {
        if !seen.insert(scope) {
            continue;
        }
        for (_, namespace, binding) in resolution.bindings.scope(scope) {
            if binding.access != Access::Public {
                continue;
            }
            let target = &resolution.definitions[binding.target.0 as usize];
            if namespace == Namespace::Macro {
                return Err(error(
                    "macro bindings in a module reached only through a re-export are outside the macro metadata profile",
                ));
            }
            if namespace == Namespace::Type
                && target.kind == DefKind::Module
                && let Some(members) = target.members.filter(|_| target.node.is_some())
            {
                aliased.push(members);
            }
        }
    }
    Ok(bound)
}

/// Imported macro definitions by their binding identity: the owning
/// artifact's identity and the definition row in its table.
fn imported_rows(report: &Report) -> HashMap<DefId, (&CrateIdentity, &[String], &str)> {
    let Some(crates) = report.resolution.crates.as_deref() else {
        return HashMap::default();
    };
    let artifacts = crates.dependencies().artifacts();
    crates
        .macros()
        .iter()
        .filter_map(|entry| {
            let artifact = artifacts.get(entry.artifact)?;
            let macros = artifact.macros()?;
            let (placement, name) = match entry.row {
                DefinitionRow::Declarative(row) => (&[][..], macros.rows().get(row)?.name()),
                DefinitionRow::Builtin(row) => {
                    let row = macros.builtins().get(row)?;
                    (row.placement(), row.name())
                }
            };
            Some((entry.definition, (artifact.identity(), placement, name)))
        })
        .collect()
}

/// The re-export row of one public binding of an imported definition. Its
/// owner is the defining artifact, which must be one of `dependencies`.
fn upstream(
    entry: &Bound<'_>,
    imported: &HashMap<DefId, (&CrateIdentity, &[String], &str)>,
    dependencies: &[CrateIdentity],
) -> Result<ReexportMacro, Error> {
    let &(owner, placement, name) = imported
        .get(&entry.target)
        .ok_or_else(|| error("public macro binding has no transported definition row"))?;
    let ordinal = dependencies
        .binary_search(owner)
        .map_err(|_| error("macro re-export target's crate is not a direct dependency"))?;
    ReexportMacro::new(
        entry.placement.to_vec(),
        entry.name.to_owned(),
        u32::try_from(ordinal + 1).map_err(|_| error("too many macro owners"))?,
        placement.to_vec(),
        name.to_owned(),
    )
    .map_err(error)
}

/// The rows of one table under construction.
#[derive(Default)]
struct Table {
    rows: Vec<Macro>,
    builtins: Vec<BuiltinMacro>,
    reexports: Vec<ReexportMacro>,
}
impl Table {
    /// Publish one local definition and its other public bindings. Its row
    /// sits at its own direct binding when that is public, and otherwise at
    /// its least re-export key; every other binding re-exports that row.
    fn define(
        &mut self,
        report: &Report,
        sites: &HashMap<NodeKey, usize>,
        target: DefId,
        entries: &[&Bound<'_>],
    ) -> Result<(), Error> {
        let canonical = entries
            .iter()
            .copied()
            .find(|entry| entry.direct)
            .or_else(|| entries.iter().copied().min_by(|l, r| l.key().cmp(&r.key())))
            .ok_or_else(|| error("missing exported macro binding"))?;
        let node = report.resolution.definitions[target.0 as usize]
            .node
            .ok_or_else(|| error("exported macro has no source definition"))?;
        let definition = sites
            .get(&node)
            .map(|&index| &report.definitions[index])
            .ok_or_else(|| error("missing exported macro definition"))?;
        if definition.introduction.0 != 0 {
            return Err(error(
                "macro-generated exported macros are outside the macro metadata profile",
            ));
        }
        if let Some(builtin) = definition.exported_builtin().map_err(error)? {
            self.builtins.push(
                BuiltinMacro::new(
                    canonical.placement.to_vec(),
                    canonical.name.to_owned(),
                    syntax(report, node)?,
                    builtin.identity.to_owned(),
                    builtin.kind,
                    builtin.helpers,
                )
                .map_err(error)?,
            );
        } else {
            let root = canonical.direct && canonical.scope == report.resolution.module_scopes[0];
            self.rows
                .push(row(report, root, canonical.name, definition, node)?);
        }
        for entry in entries
            .iter()
            .filter(|entry| !std::ptr::eq(**entry, canonical))
        {
            self.reexports.push(
                ReexportMacro::new(
                    entry.placement.to_vec(),
                    entry.name.to_owned(),
                    0,
                    canonical.placement.to_vec(),
                    canonical.name.to_owned(),
                )
                .map_err(error)?,
            );
        }
        Ok(())
    }
    fn finish(mut self) -> Result<Macros, Error> {
        self.rows
            .sort_unstable_by(|left, right| left.name().cmp(right.name()));
        self.builtins.sort_unstable_by(|left, right| {
            (left.placement(), left.name()).cmp(&(right.placement(), right.name()))
        });
        self.reexports.sort_unstable_by(|left, right| {
            (left.placement(), left.name()).cmp(&(right.placement(), right.name()))
        });
        Macros::with_reexports(self.rows, self.builtins, self.reexports).map_err(error)
    }
}

/// Publish every public macro binding of the crate: each local definition
/// once as a definition row, and every other binding, local or imported, as
/// a re-export of a definition row's identity. Owners are this crate (0) or
/// one of `dependencies`, which is sorted.
pub(super) fn collect(
    report: &Report,
    dependencies: &[CrateIdentity],
) -> Result<Option<Macros>, Error> {
    let bound = bindings(report)?;
    if bound.is_empty() {
        return Ok(None);
    }
    let imported = imported_rows(report);
    let mut local: HashMap<DefId, Vec<&Bound<'_>>> = HashMap::default();
    let mut table = Table::default();
    for entry in &bound {
        let target = &report.resolution.definitions[entry.target.0 as usize];
        if target.kind != DefKind::Macro {
            return Err(error(
                "public macro binding names a definition outside the macro metadata profile",
            ));
        }
        if target.node.is_some() {
            local.entry(entry.target).or_default().push(entry);
        } else {
            table
                .reexports
                .push(upstream(entry, &imported, dependencies)?);
        }
    }
    let sites = report
        .definitions
        .iter()
        .enumerate()
        .filter_map(|(index, definition)| definition.node.map(|node| (node, index)))
        .collect::<HashMap<_, _>>();
    for (target, entries) in local {
        table.define(report, &sites, target, &entries)?;
    }
    table.finish().map(Some)
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
    root: bool,
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
    if !root {
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
