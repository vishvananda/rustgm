//! Receiver syntax binds the whole value or reference, independently of patterns.
use super::{BinderId, Checker, DefId, Error, Kind, List, NodeKey, RegionId, Type, Work, regions};
use crate::{resolve::model::Target, sema::associated::Receiver, syntax::token_tree::Span};

impl Checker<'_> {
    pub(super) fn schedule_parameter(
        &mut self,
        key: NodeKey,
        owner: Option<DefId>,
        binder: BinderId,
        first: bool,
    ) -> Result<(), Error> {
        let key = self.unattributed(key);
        self.work.push(Work::Parameter(binder, true));
        let receiver = if let Kind::Parameter { pattern, .. } = self.kind(key) {
            self.receiver_binding(NodeKey {
                node: pattern,
                ..key
            })?
        } else {
            None
        };
        if receiver.is_some() {
            let owner = owner
                .filter(|id| self.facts.associated.owner(*id).is_some())
                .ok_or_else(|| self.error(key, "self receiver requires an associated method"))?;
            if self.facts.associated.schema(owner).is_none() {
                return Err(self.error(key, "trait method receivers require trait semantics"));
            }
            if !first {
                return Err(self.error(key, "self receiver must be the first parameter"));
            }
            self.prepare_receiver(key, owner, binder)?;
        } else {
            let ty = super::parameter_type(self.report, key)?;
            self.node_work(
                key.file,
                ty,
                regions::Context {
                    binder: Some(binder),
                    position: regions::Position::Input,
                },
            );
        }
        self.work.push(Work::Parameter(binder, false));
        Ok(())
    }
    pub(super) fn schedule_drop_receiver(
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
            .filter(|node| unit.configured.active[node.get() as usize]);
        let first = parameters.next();
        if first.is_none() || parameters.next().is_some() {
            return Err(self.error(key, "Drop requires exactly one receiver parameter"));
        }
        let node = first.ok_or_else(|| self.error(key, "missing Drop receiver"))?;
        self.prepare_receiver(self.unattributed(NodeKey { node, ..key }), owner, binder)
    }
    fn prepare_receiver(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
    ) -> Result<(), Error> {
        let Kind::Parameter { pattern, ty } = self.kind(key) else {
            return Err(self.error(key, "missing receiver syntax"));
        };
        let binding = self
            .receiver_binding(NodeKey {
                node: pattern,
                ..key
            })?
            .ok_or_else(|| self.error(key, "Drop method requires a self receiver"))?;
        self.work.push(Work::Receiver(key, owner, binder));
        let context = regions::Context {
            binder: Some(binder),
            position: regions::Position::Input,
        };
        if let Some(ty) = ty {
            if binding.node != pattern {
                return Err(self.error(key, "explicit receiver type requires a self binding"));
            }
            self.node_work(key.file, ty, context);
            return Ok(());
        }
        let reference = match self.kind(NodeKey {
            node: pattern,
            ..key
        }) {
            Kind::ReferencePattern { mutable, .. } => Some((
                mutable,
                self.regions
                    .elided(context)
                    .map_err(|e| self.error(key, e))?,
            )),
            Kind::Receiver {
                mutable, lifetime, ..
            } => Some((mutable, self.receiver_region(binding, lifetime, context)?)),
            _ if self.facts.destructors.method(owner).is_some() => {
                return Err(self.error(key, "Drop receiver must borrow self mutably"));
            }
            _ => None,
        };
        let target = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| self.error(key, "missing associated receiver environment"))?
            .self_type;
        let ty = if let Some((mutable, region)) = reference {
            self.intern(
                key,
                Type::Reference {
                    mutable,
                    region,
                    referent: target,
                },
            )?
        } else {
            target
        };
        self.store(key, ty)
    }
    fn receiver_binding(&self, key: NodeKey) -> Result<Option<NodeKey>, Error> {
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
            return Ok(None);
        };
        Ok((self
            .report
            .package
            .unit(key.file)
            .spelling(name)
            .map_err(|e| self.error(key, e.message))?
            == "self")
            .then_some(key))
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
    pub(super) fn complete_receiver(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
    ) -> Result<(), Error> {
        let Kind::Parameter {
            pattern,
            ty: annotation,
        } = self.kind(key)
        else {
            return Err(self.error(key, "invalid receiver syntax"));
        };
        let ty = self.ty(annotation.map_or(key, |node| NodeKey { node, ..key }))?;
        let (base, references) = if let Some(implementation) = self.facts.destructors.method(owner)
        {
            let Some(Type::Reference {
                mutable: true,
                region,
                referent,
            }) = self.facts.types.kind(ty)
            else {
                return Err(self.error(key, "Drop receiver must have type &mut Self"));
            };
            if referent != implementation.target
                || !matches!(region.bound_parts(),Some((0,index)) if u32::from(index)>=implementation.parameters && u32::from(index)<self.regions.count(binder))
            {
                return Err(self.error(
                    key,
                    "Drop receiver requires an independent lifetime and the exact Self type",
                ));
            }
            (referent, 1)
        } else {
            self.receiver_base(key, owner, binder, ty)?
        };
        let binding = self
            .receiver_binding(NodeKey {
                node: pattern,
                ..key
            })?
            .ok_or_else(|| self.error(key, "method requires a self receiver"))?;
        let definition = self
            .facts
            .node(binding)
            .and_then(|f| f.binding)
            .ok_or_else(|| self.error(key, "missing receiver binding definition"))?;
        let mutable = matches!(self.kind(binding), Kind::Binding { mutable: true, .. });
        self.facts.receivers.insert(
            key,
            Receiver {
                binding,
                definition,
                ty,
                base,
                references,
                mutable,
            },
        );
        self.store(key, ty)
    }
    fn receiver_base(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
        ty: crate::types::TypeId,
    ) -> Result<(crate::types::TypeId, u32), Error> {
        let target = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| self.error(key, "missing receiver Self type"))?
            .self_type;
        let mut terminal = ty;
        let mut references = 0;
        while terminal != target {
            match (
                self.facts.types.kind(terminal),
                self.facts.types.kind(target),
            ) {
                (Some(Type::Adt(a, _)), Some(Type::Adt(b, _))) if a == b => break,
                (Some(Type::Reference { referent, .. }), _) => {
                    references += 1;
                    if references > 1_000_000 {
                        return Err(self.error(key, "receiver shape work budget exceeded"));
                    }
                    terminal = referent;
                }
                _ => return Err(self.error(key, "receiver must be Self or a reference to Self")),
            }
        }
        if let Some(Type::Reference {
            referent, region, ..
        }) = self.facts.types.kind(ty)
            && referent == terminal
        {
            self.regions.prefer_receiver(binder, region);
        }
        Ok((terminal, references))
    }
}
