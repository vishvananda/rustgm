//! Elaborate written item arguments once; body inference owns omitted slots.
use super::{Argument, Checker, DefId, Error, Kind, NodeKey, Work, regions};
use crate::{
    sema::generics::{ItemUse, SourceArgument},
    syntax::ast::{List, NodeId},
};

impl Checker<'_> {
    fn item_parameters(&self, owner: DefId) -> &[crate::sema::generics::Parameter] {
        self.facts
            .generics
            .parameters
            .get(&owner)
            .map_or(&[], Vec::as_slice)
    }
    fn item_lifetime_count(&self, owner: DefId) -> usize {
        self.item_parameters(owner)
            .iter()
            .take_while(|p| matches!(p.argument, Argument::Lifetime(_)))
            .count()
    }
    pub(in super::super) fn generic_use_owner(&self, key: NodeKey) -> Option<DefId> {
        // Completed type annotations already consumed their written arguments.
        let fact = self.facts.node(key)?;
        if fact.ty.is_some() || self.facts.generics.constructor_uses.contains_key(&key) {
            return None;
        }
        let crate::resolve::model::Target::Definition(mut owner) = fact.reference? else {
            return None;
        };
        if let Some(&variant) = self.facts.members.constructors.get(&owner) {
            owner = DefId(self.facts.members.variants[variant.0 as usize].owner.0);
        }
        matches!(
            self.report.resolution.definitions[owner.0 as usize].kind,
            crate::resolve::model::DefKind::Function
                | crate::resolve::model::DefKind::Struct
                | crate::resolve::model::DefKind::Enum
                | crate::resolve::model::DefKind::TypeAlias
        )
        .then_some(owner)
        .filter(|id| {
            self.facts.generics.has_parameters(*id)
                || self.report.resolution.definitions[id.0 as usize].kind
                    == crate::resolve::model::DefKind::TypeAlias
        })
    }

    pub(super) fn item_argument_lists(
        &self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(Option<List>, Option<List>), Error> {
        let (lifetimes, supplied) = crate::sema::arguments::groups(
            self.report,
            key,
            self.report.resolution.definitions[owner.0 as usize].kind
                == crate::resolve::model::DefKind::Enum,
        )?;
        let count = supplied.map_or(0, |list| (list.end - list.start) as usize);
        crate::sema::arguments::arity(self.item_parameters(owner), count, |binding| {
            self.defaults.get(binding).is_some()
        })
        .map_err(|e| self.error(key, e))?;
        Ok((lifetimes, supplied))
    }

    pub(in super::super) fn schedule_item_arguments(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        if self.report.resolution.definitions[owner.0 as usize].kind
            == crate::resolve::model::DefKind::TypeAlias
        {
            if self.inferred_expression_alias(key, owner, context)? {
                return Ok(());
            }
            self.annotation_holes(key, steps, limit, scan)?;
            self.node_work(key.file, key.node, context);
            return Ok(());
        }
        if self.report.resolution.definitions[owner.0 as usize].kind
            == crate::resolve::model::DefKind::Function
        {
            self.demand_function_template(key, owner)?;
        }
        let (_, supplied) = self.item_argument_lists(key, owner)?;
        self.work.push(Work::ItemArguments(key, owner));
        let Some(list) = supplied else {
            return Ok(());
        };
        let offset = self.item_lifetime_count(owner);
        let ast = &self.report.package.unit(key.file).configured.ast;
        for parameter in self
            .facts
            .generics
            .parameters
            .get(&owner)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .skip(ast.list(list).len() + offset)
            .filter(|p| p.default.is_none())
            .rev()
        {
            if let Some(id) = self.defaults.get(parameter.binding) {
                self.work.push(Work::Default(id, false));
            }
        }
        for (index, node) in ast.list(list).iter().copied().enumerate().rev() {
            let key = NodeKey { node, ..key };
            if matches!(self.kind(key), Kind::InferType) {
                continue;
            }
            match self.item_parameters(owner)[index + offset].argument {
                Argument::Type(_) => {
                    self.annotation_holes(key, steps, limit, scan)?;
                    self.node_work(key.file, node, context);
                }
                Argument::Const(id) => self.schedule_constant_argument(key, id, owner, context)?,
                Argument::Lifetime(_) => {
                    return Err(
                        self.error(key, "function parameter metadata contains a lifetime slot")
                    );
                }
            }
        }
        Ok(())
    }
    fn demand_function_template(&mut self, key: NodeKey, owner: DefId) -> Result<(), Error> {
        if self
            .facts
            .lifetimes
            .callables
            .get(crate::types::ItemId(owner.0))
            .is_some()
        {
            return Ok(());
        }
        let ty = self.facts.definitions[owner.0 as usize]
            .ok_or_else(|| self.error(key, "missing generic function type"))?;
        let Some(crate::types::Kind::FunctionItem { signature, .. }) = self.facts.types.kind(ty)
        else {
            return Err(self.error(key, "missing generic function signature"));
        };
        let scheme = self.facts.lifetimes.functions.get(&owner).cloned();
        let Some(scheme) = scheme else {
            if self.facts.generics.has_parameters(owner) {
                return Err(self.error(key, "missing generic function lifetime scheme"));
            }
            return Ok(());
        };
        self.callable_template(key, owner, signature, &scheme)
    }
    pub(in super::super) fn complete_item_arguments(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(), Error> {
        let (lifetimes, supplied) = self.item_argument_lists(key, owner)?;
        let mut arguments = Vec::new();
        let offset = self.item_lifetime_count(owner);
        if let Some(list) = supplied {
            let count = self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .list(list)
                .len();
            for index in 0..count {
                let node = self.report.package.unit(key.file).configured.ast.list(list)[index];
                let key = NodeKey { node, ..key };
                arguments.push(if matches!(self.kind(key), Kind::InferType) {
                    SourceArgument::Infer
                } else {
                    match self.item_parameters(owner)[index + offset].argument {
                        Argument::Type(_) => SourceArgument::Type(key),
                        Argument::Const(id) => {
                            let ty =
                                self.facts.types.const_type(id).ok_or_else(|| {
                                    self.error(key, "missing const parameter type")
                                })?;
                            SourceArgument::Const(self.generic_constant(key, ty)?)
                        }
                        Argument::Lifetime(_) => {
                            return Err(self.error(key, "invalid function argument metadata"));
                        }
                    }
                });
            }
        }
        self.facts.generics.item_uses.insert(
            key,
            ItemUse {
                lifetimes,
                arguments,
            },
        );
        Ok(())
    }
}
