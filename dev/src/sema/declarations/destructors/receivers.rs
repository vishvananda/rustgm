//! Receiver syntax binds the whole reference, independently of pattern semantics.
use super::super::{
    BinderId, Checker, DefId, Error, Kind, List, NodeKey, RegionId, Type, TypeId, Work, regions,
};
use crate::{
    resolve::model::Target,
    sema::{destructors::Receiver, lifetimes::FunctionRegions},
    syntax::token_tree::Span,
    types::outlives::Pair,
};
impl Checker<'_> {
    pub(in crate::sema::declarations) fn schedule_receiver(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
        list: List,
    ) -> Result<(), Error> {
        let unit = self.report.package.unit(key.file);
        let mut parameters = unit
            .configured
            .ast
            .list(list)
            .iter()
            .copied()
            .filter(|n| unit.configured.active[n.0 as usize]);
        let first = parameters.next();
        if first.is_none() || parameters.next().is_some() {
            return Err(self.error(key, "Drop requires exactly one receiver parameter"));
        }
        let node = first.ok_or_else(|| self.error(key, "missing Drop receiver"))?;
        let key = self.unattributed(NodeKey { node, ..key });
        let Kind::Parameter { pattern, ty } = self.kind(key) else {
            return Err(self.error(key, "missing Drop receiver syntax"));
        };
        let binding = self.receiver_binding(NodeKey {
            node: pattern,
            ..key
        })?;
        self.work.push(Work::DropReceiver(key, owner, binder));
        let context = regions::Context {
            binder: Some(binder),
            position: regions::Position::Input,
        };
        if let Some(ty) = ty {
            if binding.node != pattern {
                return Err(self.error(key, "explicit receiver type requires a self binding"));
            }
            self.node_work(key.file, ty, context);
        } else {
            let (mutable, region) = match self.kind(NodeKey {
                node: pattern,
                ..key
            }) {
                Kind::ReferencePattern { mutable, .. } => (
                    mutable,
                    self.regions
                        .elided(context)
                        .map_err(|e| self.error(key, e))?,
                ),
                Kind::Receiver {
                    mutable, lifetime, ..
                } => (mutable, self.receiver_region(binding, lifetime, context)?),
                _ => return Err(self.error(key, "Drop receiver must borrow self mutably")),
            };
            let referent = self
                .facts
                .destructors
                .method(owner)
                .ok_or_else(|| self.error(key, "missing Drop contract"))?
                .target;
            let ty = self.intern(
                key,
                Type::Reference {
                    mutable,
                    region,
                    referent,
                },
            )?;
            self.store(key, ty)?;
        }
        Ok(())
    }
    fn receiver_binding(&self, key: NodeKey) -> Result<NodeKey, Error> {
        let key = if let Kind::ReferencePattern { pattern, .. } = self.kind(key) {
            NodeKey {
                node: pattern,
                ..key
            }
        } else {
            key
        };
        let (Kind::Binding {
            name,
            by_ref: false,
            subpattern: None,
            ..
        }
        | Kind::Receiver { name, .. }) = self.kind(key)
        else {
            return Err(self.error(key, "Drop method requires a self receiver"));
        };
        if self
            .report
            .package
            .unit(key.file)
            .spelling(name)
            .map_err(|e| self.error(key, e.message))?
            != "self"
        {
            return Err(self.error(key, "Drop method requires a self receiver"));
        }
        Ok(key)
    }
    fn receiver_region(
        &mut self,
        key: NodeKey,
        lifetime: Span,
        context: regions::Context,
    ) -> Result<RegionId, Error> {
        let name = self
            .report
            .package
            .unit(key.file)
            .spelling(lifetime)
            .map_err(|e| self.error(key, e.message))?;
        match name {
            "'static" => Ok(RegionId::STATIC),
            "'_" => self.regions.elided(context).map_err(|e| self.error(key, e)),
            _ => {
                let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference)
                else {
                    return Err(self.error(key, "missing receiver lifetime resolution"));
                };
                self.regions
                    .named(context.binder, id)
                    .map_err(|e| self.error(key, e))
            }
        }
    }
    pub(in crate::sema::declarations) fn complete_receiver(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
    ) -> Result<(), Error> {
        let Kind::Parameter { pattern, ty } = self.kind(key) else {
            return Err(self.error(key, "invalid Drop receiver syntax"));
        };
        let ty = self.ty(ty.map_or(key, |node| NodeKey { node, ..key }))?;
        let implementation = self
            .facts
            .destructors
            .method(owner)
            .ok_or_else(|| self.error(key, "missing Drop contract"))?;
        let Some(Type::Reference {
            mutable: true,
            region,
            referent,
        }) = self.facts.types.kind(ty)
        else {
            return Err(self.error(key, "Drop receiver must have type &mut Self"));
        };
        if referent != implementation.target
            || !matches!(region.bound_parts(), Some((0, index)) if u32::from(index) >= implementation.parameters && u32::from(index) < self.regions.count(binder))
        {
            return Err(self.error(
                key,
                "Drop receiver requires an independent lifetime and the exact Self type",
            ));
        }
        let binding = self.receiver_binding(NodeKey {
            node: pattern,
            ..key
        })?;
        let definition = self
            .facts
            .node(binding)
            .and_then(|f| f.binding)
            .ok_or_else(|| self.error(key, "missing receiver binding definition"))?;
        let mutable = matches!(self.kind(binding), Kind::Binding { mutable: true, .. });
        self.facts.destructors.receivers.insert(
            key,
            Receiver {
                binding,
                definition,
                ty,
                mutable,
            },
        );
        self.store(key, ty)
    }
    pub(in crate::sema::declarations) fn drop_method_regions(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
        declared: List,
        signature: TypeId,
        bounds: &[Pair],
    ) -> Result<(), Error> {
        let Some(Type::Function {
            parameters,
            result: TypeId::UNIT,
            unsafe_: false,
            ..
        }) = self.facts.types.kind(signature)
        else {
            return Err(self.error(key, "Drop method must return unit"));
        };
        let Some(&receiver) = self.facts.types.list(parameters).and_then(|p| p.first()) else {
            return Err(self.error(key, "missing typed Drop receiver"));
        };
        let Some(Type::Reference { region, .. }) = self.facts.types.kind(receiver) else {
            return Err(self.error(key, "invalid typed Drop receiver"));
        };
        let declared = self.declared_regions(key, declared)?;
        let mut method_slots = std::collections::HashSet::new();
        for &definition in &declared {
            method_slots.insert(
                self.regions
                    .named(Some(binder), definition)
                    .map_err(|e| self.error(key, e))?,
            );
        }
        if bounds
            .iter()
            .any(|(a, b)| method_slots.contains(a) || method_slots.contains(b))
        {
            return Err(self.error(
                key,
                "Drop method cannot introduce early-bound lifetime parameters",
            ));
        }
        let (_, receiver_slot) = region
            .bound_parts()
            .ok_or_else(|| self.error(key, "invalid Drop receiver region"))?;
        self.facts.destructors.extend_bounds(owner, bounds);
        let implementation = self
            .facts
            .destructors
            .method(owner)
            .ok_or_else(|| self.error(key, "missing Drop contract"))?;
        self.facts.lifetimes.functions.insert(
            owner,
            FunctionRegions {
                total: self.regions.count(binder),
                early: Vec::new(),
                late: vec![receiver_slot],
                has_declared_late: !declared.is_empty(),
                bounds: implementation.bounds.clone(),
            },
        );
        self.function_binders.insert(key, binder);
        Ok(())
    }
}
