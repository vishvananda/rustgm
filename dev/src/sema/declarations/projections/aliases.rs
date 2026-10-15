//! Alias declarations retain dependency edges; actual uses retain instantiated proof evidence.
use super::{Checker, DefId, Error, NodeKey, regions};
use crate::{
    resolve::model::DefKind,
    sema::alias_requirements::Use,
    types::{GenericArgs, substitute::Bound},
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
            self.projections
                .alias_templates
                .insert(binder, Template { depth, ..template });
        }
        Ok(())
    }
    pub(in super::super) fn projection_alias_owner(
        &self,
        context: regions::Context,
    ) -> Option<Template> {
        if let regions::Position::Default(binding) = context.position {
            return self.defaults.owner(binding).map(|owner| Template {
                owner: binding,
                parameters: owner,
                depth: 0,
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
        })
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
            let use_ = if template.depth == 0 {
                use_
            } else {
                let mut fold = Bound::lower(template.depth);
                let arguments = self
                    .facts
                    .types
                    .arguments(arguments)
                    .ok_or_else(|| self.error(key, "missing alias proof arguments"))?
                    .collect::<Vec<_>>();
                let arguments = arguments
                    .into_iter()
                    .map(|argument| fold.apply_argument(&mut self.facts.types, argument))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| self.error(key, e))?;
                Use {
                    arguments: self
                        .facts
                        .types
                        .intern_arguments(&arguments)
                        .map_err(|e| self.error(key, e))?,
                    ..use_
                }
            };
            return self
                .facts
                .alias_requirements
                .dependency(template.owner, template.parameters, use_)
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
