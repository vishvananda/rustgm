//! Public macro bindings publish authenticated rows at their module placement.
//! Root `#[macro_export]` definitions publish their canonical body trees, and
//! `#[rustc_builtin_macro]` definitions publish their expander identity. Every
//! further public binding of a definition, local or imported, is a re-export
//! row naming that definition row's identity. Definition rows also carry their
//! `allow_internal_unstable` list, `#[unstable]` feature and
//! `allow_internal_unsafe` flag. Semiopaque `pub macro` definitions publish
//! their canonical body trees at their own placement. A public glob of a
//! dependency's module publishes one glob row instead of the names it gave.
use super::{DefKind, Error, Report, dependency_globs, error};
use crate::{
    resolve::{
        expansion::{Body, Definition},
        lint_policy::SourceDeprecation,
        model::{Access, Binding, DefId, Namespace, Origin, ScopeId},
        package::ModuleId,
        unit::NodeKey,
    },
    support::{
        crates::{
            CrateIdentity,
            exported_macros::{
                BuiltinMacro, DefinitionRow, GlobReexport, Macro, MacroDefinitionSite,
                MacroDeprecation, Macros, ReexportMacro, ScopedMacro, Syntax,
            },
        },
        hash::{HashMap, HashSet},
    },
    syntax::{ast::Kind as AstKind, item::ItemKind},
};
use std::{cmp::Reverse, collections::BinaryHeap, rc::Rc};

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

/// One public macro binding in its provider's reachable module.
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

/// The public macro bindings of a crate and each reachable placement.
#[derive(Default)]
struct Placed<'a> {
    bound: Vec<Bound<'a>>,
    placements: HashMap<ScopeId, Rc<[String]>>,
    /// Modules that only public re-exports reach.
    aliased: HashSet<ScopeId>,
}
impl<'a> Placed<'a> {
    fn binding(
        &mut self,
        scope: ScopeId,
        placement: &Rc<[String]>,
        name: &'a str,
        binding: &Binding,
    ) -> Result<(), Error> {
        if binding.ambiguous {
            return Err(error(
                "ambiguous public macro bindings are outside the macro metadata profile",
            ));
        }
        self.bound.push(Bound {
            placement: Rc::clone(placement),
            name,
            scope,
            target: binding.target,
            direct: binding.origin == Origin::Direct,
        });
        Ok(())
    }
}

/// Direct module paths keep their canonical spelling. For modules reached
/// only by aliases, visit the shortest public paths in lexical order.
type AliasPaths = BinaryHeap<Reverse<(usize, Rc<[String]>, u32)>>;

fn alias_path(aliases: &mut AliasPaths, parent: &[String], name: &str, scope: ScopeId) {
    let mut path = parent.to_vec();
    path.push(name.to_owned());
    aliases.push(Reverse((path.len(), path.into(), scope.0)));
}

fn bindings<'a>(
    report: &'a Report,
    globs: &dependency_globs::Globs,
    source_modules: bool,
) -> Result<Placed<'a>, Error> {
    let resolution = &report.resolution;
    let root = resolution.module_scopes[0];
    let mut placed = Placed::default();
    let mut aliases = AliasPaths::new();
    let mut work = vec![root];
    let mut seen = HashSet::from_iter([root]);
    while let Some(scope) = work.pop() {
        let module = resolution.scopes[scope.0 as usize].module;
        let placement: Rc<[String]> = module_path(report, module)?.into();
        placed.placements.insert(scope, Rc::clone(&placement));
        for (name, namespace, binding) in resolution.bindings.scope(scope) {
            if binding.access != Access::Public || globs.covered(scope, binding) {
                continue;
            }
            let name = report.package.names.get(name);
            let target = &resolution.definitions[binding.target.0 as usize];
            if namespace == Namespace::Type && target.kind == DefKind::Module {
                let Some(members) = target.members.filter(|_| target.node.is_some()) else {
                    continue;
                };
                if binding.origin != Origin::Direct {
                    alias_path(&mut aliases, &placement, name, members);
                } else if seen.insert(members) {
                    work.push(members);
                }
            } else if namespace == Namespace::Macro {
                placed.binding(scope, &placement, name, binding)?;
            }
        }
    }
    alias_bindings(report, globs, source_modules, aliases, seen, &mut placed)?;
    Ok(placed)
}

fn alias_bindings<'a>(
    report: &'a Report,
    globs: &dependency_globs::Globs,
    source_modules: bool,
    mut aliases: AliasPaths,
    mut seen: HashSet<ScopeId>,
    placed: &mut Placed<'a>,
) -> Result<(), Error> {
    let resolution = &report.resolution;
    while let Some(Reverse((depth, placement, scope))) = aliases.pop() {
        let scope = ScopeId(scope);
        if !seen.insert(scope) {
            continue;
        }
        placed.aliased.insert(scope);
        if source_modules {
            if depth > crate::support::crates::exported_macros::MAX_PLACEMENT_DEPTH {
                return Err(error("exported macro placement exceeds profile budget"));
            }
            placed.placements.insert(scope, Rc::clone(&placement));
        }
        for (name, namespace, binding) in resolution.bindings.scope(scope) {
            if binding.access != Access::Public || globs.covered(scope, binding) {
                continue;
            }
            let name = report.package.names.get(name);
            let target = &resolution.definitions[binding.target.0 as usize];
            if namespace == Namespace::Macro {
                if !source_modules {
                    return Err(error(
                        "macro bindings in a module reached only through a re-export are outside the macro metadata profile",
                    ));
                }
                placed.binding(scope, &placement, name, binding)?;
            }
            if namespace == Namespace::Type
                && target.kind == DefKind::Module
                && let Some(members) = target.members.filter(|_| target.node.is_some())
                && !seen.contains(&members)
            {
                alias_path(&mut aliases, &placement, name, members);
            }
        }
    }
    Ok(())
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
                DefinitionRow::Scoped(row) => {
                    let row = macros.scoped().get(row)?;
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

/// Each local declaration's regular `#[unstable]` feature, by its node.
type Unstable<'report> = HashMap<NodeKey, &'report str>;

fn unstable(report: &Report) -> Unstable<'_> {
    let Some(inventory) = report
        .package
        .library_policy()
        .and_then(crate::resolve::stability::Library::staged_inventory)
    else {
        return HashMap::default();
    };
    inventory
        .policies()
        .iter()
        .filter(|policy| {
            policy.kind == crate::resolve::stability::PolicyKind::Regular
                && matches!(
                    policy.level,
                    crate::resolve::stability::Level::Unstable { .. }
                )
        })
        .map(|policy| (policy.origin, &*policy.feature))
        .collect()
}

struct Deprecation<'a> {
    placement: Rc<[String]>,
    name: &'a str,
    source: SourceDeprecation<'a>,
}

/// The rows of one table under construction.
#[derive(Default)]
struct Table<'a> {
    rows: Vec<Macro>,
    builtins: Vec<BuiltinMacro>,
    reexports: Vec<ReexportMacro>,
    scoped: Vec<ScopedMacro>,
    globs: Vec<GlobReexport>,
    deprecations: Vec<Deprecation<'a>>,
    source_modules: bool,
}
impl<'a> Table<'a> {
    /// Publish one local definition and its other public bindings. Its row
    /// sits at its own direct binding when that is public, and otherwise at
    /// its least re-export key; every other binding re-exports that row.
    fn define(
        &mut self,
        report: &'a Report,
        sites: &HashMap<NodeKey, usize>,
        unstable: &Unstable<'_>,
        target: DefId,
        entries: &[&Bound<'a>],
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
        // An introduction-zero definition lies in physical source, whose every
        // token has the root context; only a generated one can carry marks.
        if definition.introduction.0 != 0 {
            return Err(Error::at(
                node.file,
                report
                    .package
                    .unit(node.file)
                    .configured
                    .ast
                    .node(node.node)
                    .span
                    .start,
                "macro-generated exported macros are outside the macro metadata profile",
            ));
        }
        let feature = unstable.get(&node).map(|&feature| feature.to_owned());
        if let Some(builtin) = definition.exported_builtin().map_err(error)? {
            let internal_unstable = definition.exported_internal_unstable().map_err(error)?;
            self.builtins.push(
                BuiltinMacro::new(
                    canonical.placement.to_vec(),
                    canonical.name.to_owned(),
                    syntax(report, node)?,
                    builtin.identity.to_owned(),
                    builtin.kind,
                    builtin.helpers,
                )
                .and_then(|row| row.with_internal_unstable(internal_unstable))
                .and_then(|row| row.with_unstable(feature))
                .map_err(error)?
                .with_internal_unsafe(definition.exported_internal_unsafe()),
            );
        } else if matches!(definition.body, Body::Scoped { .. }) {
            self.scoped.push(scoped_row(
                report,
                canonical,
                definition,
                node,
                feature,
                self.source_modules,
            )?);
        } else {
            let root = canonical.direct && canonical.scope == report.resolution.module_scopes[0];
            self.rows.push(
                row(report, root, canonical.name, definition, node)?
                    .with_unstable(feature)
                    .map_err(error)?,
            );
        }
        self.deprecate(report, definition, node, canonical)?;
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
    fn deprecate(
        &mut self,
        report: &'a Report,
        definition: &Definition,
        node: NodeKey,
        canonical: &Bound<'a>,
    ) -> Result<(), Error> {
        let Some(producer) = report.lint_producer() else {
            return Ok(());
        };
        if let Some(source) =
            producer.source_deprecation_in(&report.package, node, definition.module)?
        {
            let bytes = [source.since, source.note, source.suggestion]
                .into_iter()
                .flatten()
                .map(str::len)
                .sum::<usize>();
            producer.diagnostic_read(
                node,
                bytes.saturating_add(
                    4 * (size_of::<Deprecation<'_>>() + size_of::<MacroDeprecation>()),
                ),
            )?;
            self.deprecations.push(Deprecation {
                placement: Rc::clone(&canonical.placement),
                name: canonical.name,
                source,
            });
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
        self.scoped.sort_unstable_by(|left, right| {
            (left.placement(), left.name()).cmp(&(right.placement(), right.name()))
        });
        let macros = Macros::with_globs(
            self.rows,
            self.builtins,
            self.reexports,
            self.scoped,
            self.globs,
        )
        .map_err(error)?;
        let mut notices = Vec::with_capacity(self.deprecations.len());
        for row in self.deprecations {
            let definition = macros
                .definition(&row.placement, row.name)
                .ok_or_else(|| error("macro deprecation lost its definition row"))?;
            notices.push(MacroDeprecation {
                definition,
                since: row.source.transport_since().map_err(error)?,
                note: row.source.note.map(Box::from),
                suggestion: row.source.suggestion.map(Box::from),
                staged: row.source.staged,
            });
        }
        notices.sort_unstable_by_key(|row| row.definition);
        macros.with_deprecations(notices).map_err(error)
    }
}

/// Publish every public macro binding of the crate: each local definition
/// once as a definition row, and every other binding, local or imported, as
/// a re-export of a definition row's identity. Each public glob of a
/// dependency's module is one glob row. Owners are this crate (0) or one of
/// `dependencies`, which is sorted.
pub(super) fn collect(
    report: &Report,
    dependencies: &[CrateIdentity],
    source_modules: bool,
) -> Result<Option<Macros>, Error> {
    let globs = dependency_globs::Globs::new(report);
    let Placed {
        bound,
        placements,
        aliased,
    } = bindings(report, &globs, source_modules)?;
    let mut table = Table {
        globs: dependency_globs::rows(report, &placements, &aliased, dependencies)?,
        source_modules,
        ..Table::default()
    };
    if bound.is_empty() && table.globs.is_empty() {
        return Ok(None);
    }
    let imported = imported_rows(report);
    let mut local: HashMap<DefId, Vec<&Bound<'_>>> = HashMap::default();
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
    let unstable = unstable(report);
    for (target, entries) in local {
        table.define(report, &sites, &unstable, target, &entries)?;
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

fn scoped_row(
    report: &Report,
    canonical: &Bound<'_>,
    definition: &Definition,
    node: NodeKey,
    feature: Option<String>,
    source_modules: bool,
) -> Result<ScopedMacro, Error> {
    if !source_modules {
        definition.exported_semiopaque().map_err(error)?;
    }
    let unit = report.package.unit(node.file);
    let AstKind::Item(index) = unit.configured.ast.node(node.node).kind else {
        return Err(error("exported macro definition is not an item"));
    };
    let ItemKind::DeclarativeMacro {
        parameters,
        capture,
    } = unit.configured.ast.items[index as usize].kind
    else {
        return Err(error("exported macro definition is not a scoped macro"));
    };
    if parameters.is_some() && !source_modules {
        return Err(error(
            "macros 2.0 parameter lists are outside the macro metadata profile",
        ));
    }
    let image = image_row(
        report,
        canonical.name,
        definition,
        node,
        (parameters, capture),
    )?
    .with_unstable(feature)
    .map_err(error)?;
    let row = ScopedMacro::new(canonical.placement.to_vec(), image).map_err(error)?;
    if !source_modules {
        return Ok(row);
    }
    row.with_definition_site(MacroDefinitionSite {
        hygiene: definition.exported_scoped_hygiene().map_err(error)?,
        module: module_path(report, definition.module)?,
    })
    .map_err(error)
}

fn row(
    report: &Report,
    root: bool,
    name: &str,
    definition: &Definition,
    node: NodeKey,
) -> Result<Macro, Error> {
    match &definition.body {
        Body::Declarative(_) => {}
        Body::Scoped { .. } => {
            return Err(error(
                "macros 2.0 definitions are outside the macro metadata profile",
            ));
        }
        Body::Builtin(_) => return Err(error("exported builtin macro lost its identity")),
    }
    if !root {
        return Err(error("exported macro_rules rows belong to the crate root"));
    }
    let unit = report.package.unit(node.file);
    let AstKind::Item(index) = unit.configured.ast.node(node.node).kind else {
        return Err(error("exported macro definition is not an item"));
    };
    let ItemKind::MacroRules { capture } = unit.configured.ast.items[index as usize].kind else {
        return Err(error("exported macro definition is not macro_rules"));
    };
    image_row(report, name, definition, node, (None, capture))
}

/// The canonical image, rule count and permissions of one macro body.
fn image_row(
    report: &Report,
    name: &str,
    definition: &Definition,
    node: NodeKey,
    (parameters, capture): (Option<u32>, u32),
) -> Result<Macro, Error> {
    let compiled = match &definition.body {
        Body::Declarative(compiled) => compiled,
        Body::Scoped { compiled, .. } => compiled.as_ref(),
        Body::Builtin(_) => return Err(error("exported builtin macro lost its identity")),
    };
    let internal_unstable = definition.exported_internal_unstable().map_err(error)?;
    let unit = report.package.unit(node.file);
    let captures = &unit.configured.ast.captures;
    let forest = &captures[capture as usize];
    let source = unit.source.text();
    let (image, trees) = match parameters {
        Some(parameters) => crate::resolve::crates::concise_macro_image(
            source,
            &captures[parameters as usize],
            forest,
        ),
        None => crate::resolve::crates::macro_image(source, forest),
    }
    .map_err(error)?;
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
    .map(|row| row.with_internal_unsafe(definition.exported_internal_unsafe()))
    .map_err(error)
}
