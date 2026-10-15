//! Alias declarations retain dependency edges; actual uses retain instantiated proof evidence.
use super::{Checker, DefId, Error, NodeKey, regions};
use crate::{
    resolve::model::DefKind,
    sema::alias_requirements::Use,
    types::{GenericArgs, lifetime_proof::ScopeId},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum ScopeKey {
    Declaration(Option<regions::BinderId>),
    Body(NodeKey),
}

#[derive(Clone, Copy)]
pub(in super::super) struct Template {
    pub owner: DefId,
    pub parameters: DefId,
    pub depth: u16,
    scope: Option<ScopeId>,
}
impl Checker<'_> {
    pub(in super::super) fn inherit_alias_scope(
        &mut self,
        key: NodeKey,
        parent: regions::Context,
        binder: regions::BinderId,
    ) -> Result<(), Error> {
        if let Some(template) = self.projection_alias_owner(parent) {
            let depth = template
                .depth
                .checked_add(1)
                .ok_or_else(|| self.error(key, "alias proof binder depth exhausted"))?;
            self.projections.alias_templates.insert(
                binder,
                Template {
                    depth,
                    scope: None,
                    ..template
                },
            );
        }
        Ok(())
    }
    pub(in super::super) fn projection_alias_owner(
        &self,
        context: regions::Context,
    ) -> Option<Template> {
        if let regions::Position::Alias(owner) = context.position {
            return Some(Template {
                owner,
                parameters: owner,
                depth: 0,
                scope: None,
            });
        }
        if let regions::Position::Default(binding) = context.position {
            return self.defaults.owner(binding).map(|owner| Template {
                owner: binding,
                parameters: owner,
                depth: 0,
                scope: None,
            });
        }
        let binder = context.binder?;
        if let Some(template) = self.projections.alias_templates.get(&binder) {
            return Some(*template);
        }
        let owner = self.regions.owner(binder)?;
        (self.report.resolution.definitions[owner.0 as usize].kind == DefKind::TypeAlias
            && self.facts.associated.owner(owner).is_none())
        .then_some(Template {
            owner,
            parameters: owner,
            depth: 0,
            scope: None,
        })
    }

    pub(in super::super) fn projection_alias_scope(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        template: Template,
    ) -> Result<ScopeId, Error> {
        if let Some(scope) = template.scope {
            return Ok(scope);
        }
        let mut chain = Vec::new();
        let mut binder = context.binder;
        let mut scope = ScopeId::ROOT;
        for _ in 0..template.depth {
            let current = binder.ok_or_else(|| self.error(key, "missing alias proof binder"))?;
            if let Some(cached) = self
                .projections
                .alias_templates
                .get(&current)
                .and_then(|template| template.scope)
            {
                scope = cached;
                break;
            }
            chain.push(current);
            binder = self.regions.parent(current);
        }
        for current in chain.into_iter().rev() {
            scope = self
                .facts
                .alias_requirements
                .scope(template.owner, template.parameters, scope)
                .map_err(|e| self.error(key, e))?;
            self.projections
                .alias_templates
                .get_mut(&current)
                .ok_or_else(|| Error::at(key.file, 0, "missing alias proof scope owner"))?
                .scope = Some(scope);
        }
        Ok(scope)
    }
    pub(in super::super) fn complete_alias_scope(&mut self, binder: regions::BinderId, count: u32) {
        if let Some(template) = self.projections.alias_templates.get(&binder)
            && let Some(scope) = template.scope
        {
            self.facts
                .alias_requirements
                .complete_scope(template.owner, scope, count);
        }
    }
    pub(in super::super) fn alias_path_use(
        &mut self,
        key: NodeKey,
        owner: DefId,
        arguments: GenericArgs,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.facts.alias_requirements.contains(owner)
            && self.report.resolution.definitions[owner.0 as usize].kind == DefKind::TypeAlias
        {
            self.alias_use(key, owner, arguments, context)?;
        }
        Ok(())
    }
    pub(in super::super) fn alias_use(
        &mut self,
        key: NodeKey,
        owner: DefId,
        arguments: GenericArgs,
        context: regions::Context,
    ) -> Result<(), Error> {
        if !self.facts.alias_requirements.contains(owner) {
            return Ok(());
        }
        let use_ = Use { owner, arguments };
        if let Some(template) = self.projection_alias_owner(context) {
            let scope = self.projection_alias_scope(key, context, template)?;
            return self
                .facts
                .alias_requirements
                .dependency(template.owner, template.parameters, use_, scope)
                .map_err(|e| self.error(key, e));
        }
        let scope_key = match context.position {
            regions::Position::Local(body) | regions::Position::Constant(body) => {
                ScopeKey::Body(NodeKey { node: body, ..key })
            }
            _ => ScopeKey::Declaration(context.binder),
        };
        let first = self.projection_region_start(key, context)?;
        let scope = self.projections.alias_scopes.entry(scope_key).or_default();
        let proof = self
            .facts
            .alias_requirements
            .instantiate(&mut self.facts.types, use_, first, scope)
            .map_err(|e| self.error(key, e))?;
        self.retain_projection_proof(key, context, proof)
    }
}
