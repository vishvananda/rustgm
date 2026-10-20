//! Object defaults distinguish callable quantifiers from early lifetime arguments.
use super::{Checker, DefId, Error, Kind, NodeKey, RegionId, Work, regions};
use crate::{
    resolve::model::{DefKind, Namespace, Target},
    types::{arguments::Argument, bound_uses::Uses, parameters::uses::Uses as Parameters},
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct Times {
    prepared: HashSet<DefId>,
    early: HashMap<DefId, HashSet<DefId>>,
    aliases: HashMap<DefId, Vec<bool>>,
}

impl Checker<'_> {
    pub(super) fn prepare_object_binding_times(
        &mut self,
        context: regions::Context,
    ) -> Result<(), Error> {
        let mut binder = context.binder;
        while let Some(current) = binder {
            binder = self.regions.parent(current);
            let Some(owner) = self.regions.owner(current) else {
                continue;
            };
            let definition = &self.report.resolution.definitions[owner.0 as usize];
            if definition.kind != DefKind::Function
                || !self.object_syntax.times.prepared.insert(owner)
            {
                continue;
            }
            let key = definition.node.ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing callable source",
                )
            })?;
            let parameters = self.signature_syntax(key)?.parameters;
            let ast = &self.report.package.unit(key.file).configured.ast;
            let mut work = ast.list(parameters).to_vec();
            let mut seen = HashSet::new();
            while let Some(node) = work.pop() {
                if !seen.insert(node) {
                    continue;
                }
                if seen.len() + work.len() > 1_000_000 {
                    return Err(self.error(key, "binding-time dependency work budget exceeded"));
                }
                let source = NodeKey { node, ..key };
                if let Some(alias) = self.object_input_alias(source) {
                    self.work.push(Work::Definition(alias, false));
                }
                ast.push_children(node, &mut work);
            }
        }
        Ok(())
    }

    pub(super) fn object_early_region(
        &mut self,
        key: NodeKey,
        region: RegionId,
        context: regions::Context,
    ) -> Result<bool, Error> {
        let Some((depth, slot)) = region.bound_parts() else {
            return Ok(true);
        };
        let mut binder = context.binder;
        for _ in 0..depth {
            binder = binder.and_then(|id| self.regions.parent(id));
        }
        let Some(binder) = binder else {
            return Err(self.error(key, "object default lifetime escapes its lexical context"));
        };
        let Some(owner) = self.regions.owner(binder) else {
            return Ok(false);
        };
        if self.report.resolution.definitions[owner.0 as usize].kind != DefKind::Function
            || matches!(
                context.position,
                regions::Position::Local(_) | regions::Position::Constant(_)
            )
            || self
                .facts
                .associated
                .schema(owner)
                .is_some_and(|s| u32::from(slot) < s.captures)
        {
            return Ok(true);
        }
        let Some(name) = self.regions.definition_at(binder, slot) else {
            return Ok(false);
        };
        if !self.object_syntax.times.early.contains_key(&owner) {
            let early = self.object_early_parameters(owner)?;
            self.object_syntax.times.early.insert(owner, early);
        }
        Ok(self.object_syntax.times.early[&owner].contains(&name))
    }

    fn object_early_parameters(&mut self, owner: DefId) -> Result<HashSet<DefId>, Error> {
        let key = self.report.resolution.definitions[owner.0 as usize]
            .node
            .ok_or_else(|| {
                Error::at(crate::resolve::unit::FileId(0), 0, "missing lifetime owner")
            })?;
        let signature = self.signature_syntax(key)?;
        let item = self.item(key)?;
        let ast = &self.report.package.unit(key.file).configured.ast;
        let parameters = ast.list(signature.parameters).to_vec();
        let mut generics = ast.list(item.generics).to_vec();
        generics.extend_from_slice(ast.list(item.where_clause));
        let inputs = self.object_lifetime_uses(key, &parameters, true)?;
        let mut outputs = self.object_lifetime_uses(
            key,
            &signature.result.into_iter().collect::<Vec<_>>(),
            false,
        )?;
        // Edition 2024 opaque outputs capture the function's named lifetimes.
        if outputs.1 {
            outputs
                .0
                .extend(self.declared_regions(key, signature.binders)?);
        }
        let mut early = self.object_lifetime_uses(key, &generics, false)?.0;
        early.extend(outputs.0.difference(&inputs.0).copied());
        Ok(early)
    }

    fn object_lifetime_uses(
        &mut self,
        key: NodeKey,
        roots: &[crate::syntax::ast::NodeId],
        constrained: bool,
    ) -> Result<(HashSet<DefId>, bool), Error> {
        let mut work = roots.to_vec();
        let mut seen = HashSet::new();
        let mut names = HashSet::new();
        let mut opaque = false;
        while let Some(node) = work.pop() {
            if !seen.insert(node)
                || !self.report.package.unit(key.file).configured.active[node.0 as usize]
            {
                continue;
            }
            if seen.len() + work.len() > 1_000_000 {
                return Err(self.error(key, "binding-time lifetime work budget exceeded"));
            }
            let source = NodeKey { node, ..key };
            if let Some(Target::Definition(id)) = self.facts.node(source).and_then(|n| n.reference)
                && self.report.resolution.definitions[id.0 as usize].namespace
                    == Namespace::Lifetime
            {
                names.insert(id);
            }
            if let Kind::TraitType {
                opaque: true,
                bounds,
            } = self.kind(source)
            {
                let ast = &self.report.package.unit(key.file).configured.ast;
                opaque |= !ast
                    .list(bounds)
                    .iter()
                    .any(|&bound| matches!(ast.node(bound).kind, Kind::PreciseCapture(_)));
            }
            if constrained {
                if self.report.resolution.qualified_paths.contains_key(&source)
                    || self
                        .report
                        .resolution
                        .associated_paths
                        .contains_key(&source)
                {
                    continue;
                }
                if let Some(alias) = self.object_input_alias(source) {
                    self.object_alias_inputs(source, alias, &mut work)?;
                    continue;
                }
            }
            self.report
                .package
                .unit(key.file)
                .configured
                .ast
                .push_children(node, &mut work);
        }
        Ok((names, opaque))
    }

    fn object_input_alias(&self, key: NodeKey) -> Option<DefId> {
        if !matches!(
            self.kind(key),
            Kind::Path { .. } | Kind::Applied { .. } | Kind::Name(_)
        ) {
            return None;
        }
        let Target::Definition(id) = self.facts.node(key)?.reference? else {
            return None;
        };
        (self.report.resolution.definitions[id.0 as usize].kind == DefKind::TypeAlias
            && self.facts.associated.owner(id).is_none())
        .then_some(id)
    }

    fn object_alias_inputs(
        &mut self,
        key: NodeKey,
        owner: DefId,
        work: &mut Vec<crate::syntax::ast::NodeId>,
    ) -> Result<(), Error> {
        if !self.object_syntax.times.aliases.contains_key(&owner) {
            let mask = self.object_alias_mask(key, owner)?;
            self.object_syntax.times.aliases.insert(owner, mask);
        }
        let (arguments, omitted) = if self.facts.generics.parameters.contains_key(&owner) {
            self.supplied_arguments(key, owner)?
        } else {
            (
                crate::sema::lifetimes::arguments(self.report, key, false)?,
                0,
            )
        };
        let Some(arguments) = arguments else {
            return Ok(());
        };
        let nodes = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(arguments);
        for (index, &used) in self.object_syntax.times.aliases[&owner].iter().enumerate() {
            if used && let Some(node) = index.checked_sub(omitted).and_then(|i| nodes.get(i)) {
                work.push(*node);
            }
        }
        Ok(())
    }

    fn object_alias_mask(&self, key: NodeKey, owner: DefId) -> Result<Vec<bool>, Error> {
        let ty = self
            .facts
            .unreduced
            .definition(owner)
            .or(self.facts.definitions[owner.0 as usize])
            .ok_or_else(|| self.error(key, "unfinished binding-time alias dependency"))?;
        let mut regions = Uses::constraining();
        regions
            .collect(&self.facts.types, ty)
            .map_err(|e| self.error(key, e))?;
        let mut parameters = Parameters::constraining();
        parameters
            .collect(&self.facts.types, ty)
            .map_err(|e| self.error(key, e))?;
        let Some(formals) = self.facts.generics.parameters.get(&owner) else {
            let count = self
                .facts
                .lifetimes
                .parameters
                .get(&owner)
                .map_or(0, Vec::len);
            return (0..count)
                .map(|index| {
                    u16::try_from(index)
                        .map(|slot| regions.indices.contains(&slot))
                        .map_err(|_| self.error(key, "too many alias lifetime parameters"))
                })
                .collect();
        };
        Ok(formals.iter().map(|parameter| {
            match parameter.argument {
                Argument::Lifetime(region) => region
                    .bound_parts()
                    .is_some_and(|(depth, slot)| depth == 0 && regions.indices.contains(&slot)),
                Argument::Type(ty) => {
                    matches!(self.facts.types.kind(ty), Some(crate::types::Kind::Parameter(id)) if parameters.parameters.contains(&id))
                }
                Argument::Const(_) => false,
            }
        }).collect())
    }
}
