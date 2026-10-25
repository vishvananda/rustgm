//! Parsed implementation domains retain ordinary module identity, never visibility permission.
use super::{
    bindings::FeatureName,
    expansion::Report,
    index::{Context, Resolver},
    lookup::{Lookup, LookupMode},
    model::{DefId, DefKind, Namespace, Reference, Target},
    package::{Error, ModuleId, Package},
    unit::{FileId, NodeKey},
};
use crate::syntax::{
    ast::{Kind, NodeId},
    item::ItemKind,
    token_tree::Span,
};

#[derive(Default, Debug, PartialEq, Eq)]
struct HeaderWork {
    items: usize,
    queries: usize,
}

pub(super) fn validate(report: &Report) -> Result<(), Error> {
    let mut session = report.macro_allowances();
    headers(&report.package, |file, span| {
        let feature = FeatureName::new("impl_restriction").map_err(Error::unlocated)?;
        session.allows_at(file, span, &feature)
    })
    .map(|_| ())
}

pub(super) fn validate_plain(package: &Package) -> Result<(), Error> {
    headers(package, |_, _| Ok(false)).map(|_| ())
}

fn headers(
    package: &Package,
    mut allowed: impl FnMut(FileId, Span) -> Result<bool, Error>,
) -> Result<HeaderWork, Error> {
    if package
        .requested_impl_restriction()
        .map_err(Error::unlocated)?
    {
        return Ok(HeaderWork::default());
    }
    let mut work = HeaderWork::default();
    for (index, unit) in package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(index).map_err(|_| Error::unlocated("restriction file overflow"))?,
        );
        for item in &unit.configured.ast.items {
            work.items = work.items.saturating_add(1);
            let ItemKind::Trait {
                restriction: Some(clause),
                ..
            } = item.kind
            else {
                continue;
            };
            // Syntax gates include parsed cfg-disabled headers and descendants.
            let span = keyword(package, NodeKey { file, node: clause })?;
            work.queries = work.queries.saturating_add(1);
            if !allowed(file, span)? {
                return Err(Error::at(
                    file,
                    span.start,
                    "implementation restrictions require impl_restriction or actual literal macro allowance",
                ));
            }
        }
    }
    Ok(work)
}

fn keyword(package: &Package, clause: NodeKey) -> Result<Span, Error> {
    let node = package
        .units
        .get(clause.file.0 as usize)
        .and_then(|unit| unit.configured.ast.nodes.get(clause.node.get() as usize))
        .ok_or_else(|| Error::at(clause.file, 0, "restriction clause is missing"))?;
    let end = node
        .span
        .start
        .checked_add(4)
        .filter(|end| *end <= node.span.end)
        .ok_or_else(|| {
            Error::at(
                clause.file,
                node.span.start,
                "restriction keyword span is invalid",
            )
        })?;
    if !matches!(node.kind, Kind::ImplRestriction { .. }) {
        return Err(Error::at(
            clause.file,
            node.span.start,
            "restriction clause kind changed",
        ));
    }
    Ok(Span {
        start: node.span.start,
        end,
    })
}

/// Resolve a local trait's real restriction edge without interpreting source again.
pub(crate) fn clause(report: &Report, definition: DefId) -> Result<Option<NodeKey>, Error> {
    let declaration = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .filter(|d| d.kind == DefKind::Trait)
        .ok_or_else(|| Error::unlocated("restriction policy requires its actual trait"))?;
    let Some(key) = declaration.node else {
        return Ok(None);
    };
    let ast = &report
        .package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(|| Error::at(key.file, 0, "restriction defining source is missing"))?
        .configured
        .ast;
    let node = ast
        .nodes
        .get(key.node.get() as usize)
        .ok_or_else(|| Error::at(key.file, 0, "restriction defining owner is missing"))?;
    let Kind::Item(index) = node.kind else {
        return Err(Error::at(
            key.file,
            0,
            "restriction trait declaration kind changed",
        ));
    };
    let item = ast
        .items
        .get(index as usize)
        .ok_or_else(|| Error::at(key.file, 0, "restriction defining item is missing"))?;
    let ItemKind::Trait { restriction, .. } = item.kind else {
        return Err(Error::at(
            key.file,
            0,
            "restriction owner is not its actual trait",
        ));
    };
    restriction
        .map(|node| {
            let clause = NodeKey {
                file: key.file,
                node,
            };
            keyword(&report.package, clause)?;
            Ok(clause)
        })
        .transpose()
}

/// Existing canonical module identity is shared by resolution and implementation checks.
pub(crate) fn module(
    package: &Package,
    facts: &super::model::Resolution,
    definition: DefId,
) -> Result<ModuleId, &'static str> {
    let declaration = facts
        .definitions
        .get(definition.0 as usize)
        .filter(|d| d.kind == DefKind::Module)
        .ok_or("implementation restriction must name an actual module")?;
    let scope_id = declaration
        .members
        .ok_or("implementation restriction module scope is missing")?;
    let scope = facts
        .scopes
        .get(scope_id.0 as usize)
        .ok_or("implementation restriction module scope is missing")?;
    if package.modules.get(scope.module.0 as usize).is_none()
        || scope.associated
        || facts.module_scopes.get(scope.module.0 as usize) != Some(&scope_id)
    {
        return Err("implementation restriction module identity changed");
    }
    Ok(scope.module)
}

pub(crate) fn within(
    package: &Package,
    mut current: ModuleId,
    ancestor: ModuleId,
    mut charge: impl FnMut() -> Result<(), &'static str>,
) -> Result<bool, &'static str> {
    for _ in 0..=package.modules.len() {
        charge()?;
        let module = package
            .modules
            .get(current.0 as usize)
            .ok_or("implementation restriction module disappeared")?;
        if current == ancestor {
            return Ok(true);
        }
        let Some(parent) = module.parent else {
            return Ok(false);
        };
        current = parent;
    }
    Err("implementation restriction ancestry cycle")
}

impl Resolver<'_> {
    pub(super) fn resolve_impl_restriction(
        &mut self,
        c: Context,
        owner: NodeId,
        clause: NodeId,
    ) -> Result<(), Error> {
        let Kind::ImplRestriction { path } = self.kind(c, clause) else {
            return Err(self.error(c, clause, "invalid implementation restriction clause"));
        };
        let definition = self
            .facts
            .item_definitions
            .get(&(c.module, owner, Namespace::Type))
            .and_then(|id| self.facts.definitions.get(id.0 as usize))
            .filter(|d| d.kind == DefKind::Trait)
            .ok_or_else(|| {
                self.error(
                    c,
                    owner,
                    "implementation restriction has no trait declaration",
                )
            })?;
        let context = Context {
            scope: definition.scope,
            ..c
        };
        let (absolute, names) = self.pooled_path(context, path)?;
        let found = self.lookup_path(
            context,
            path,
            absolute,
            names.view(),
            LookupMode::Module,
            None,
        );
        self.recycle_path(names);
        let Lookup::Found(binding) = found? else {
            return Err(self.error(c, path, "implementation restriction module is unresolved"));
        };
        let module = module(self.package, self.facts, binding.target)
            .map_err(|message| self.error(c, path, message))?;
        let within = within(self.package, c.module, module, || {
            if let Some(context) = self.package.macro_context()? {
                context.contracts().charge(1, 0)?;
            }
            Ok(())
        })
        .map_err(|message| self.error(c, path, message))?;
        if !within {
            return Err(self.error(
                c,
                path,
                "implementation restriction must name an ancestor module",
            ));
        }
        self.facts.references.push(Reference {
            scope: context.scope,
            node: NodeKey {
                file: c.file,
                node: path,
            },
            namespace: Namespace::Type,
            target: Target::Definition(binding.target),
        });
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/trait_domains.rs"]
mod tests;
