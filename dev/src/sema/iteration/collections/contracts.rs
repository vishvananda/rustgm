//! Factory validation independently reconstructs the canonical role's type contract.
use super::{ItemId, Original, TypeId};
use crate::{
    sema::model::Declarations,
    types::{
        Kind, RegionId,
        arguments::{Argument, ConstKind},
        parameters::ParameterKind,
    },
};
impl Original {
    pub(super) fn validate_contract(self, facts: &Declarations) -> Result<(), &'static str> {
        let source = RegionId::bound(0, 0)?;
        let Some(Kind::Reference {
            referent,
            mutable,
            region,
        }) = facts.types.kind(self.item)
        else {
            return Err("collection Item must be a shared reference");
        };
        if referent != self.parameter
            || region != source
            || mutable != self.declaration.kind.mutable()
        {
            return Err("collection Item has a different source lifetime/element");
        }
        let Some(Kind::Adt(_, arguments)) = facts.types.kind(self.iterator) else {
            return Err("collection iterator is not nominal");
        };
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or("invalid collection iterator arguments")?
            .collect::<Vec<_>>();
        if arguments != [Argument::Lifetime(source), Argument::Type(self.parameter)] {
            return Err("collection iterator has a different source lifetime/element");
        }
        self.validate_subject(facts)?;
        if let Some(trait_) = self.declaration.kind.trait_() {
            let canonical = facts
                .iteration
                .as_deref()
                .ok_or("missing iteration registry")?
                .canonical;
            let Some(predicate) = self.predicate else {
                return Err("collection impl lacks its trait contract");
            };
            let crate::types::parameters::predicates::Bound::Trait(application) = predicate.bound
            else {
                return Err("collection impl lacks a trait application");
            };
            let application = facts
                .types
                .trait_application(application)
                .ok_or("invalid collection trait application")?;
            if predicate.ty != self.subject
                || application.item != ItemId(canonical.get(trait_).definition.0)
                || facts
                    .types
                    .arguments(application.arguments)
                    .is_none_or(|mut args| args.next().is_some())
            {
                return Err("collection impl has a different trait identity/Self");
            }
        }

        let owner = self.declaration.definition;
        let schema = facts
            .associated
            .schema(owner)
            .ok_or("missing collection impl schema")?;
        if schema.self_type != self.subject
            || schema.captures != u32::from(self.declaration.lifetime.is_some())
            || !schema.bounds.is_empty()
        {
            return Err("collection impl schema differs from its canonical role");
        }
        if facts
            .generics
            .declared_predicates_in_scope(self.declaration.method)
            .next()
            .is_some()
        {
            return Err("collection factory has unexpected declared assumptions");
        }
        if self
            .declaration
            .item
            .is_some_and(|d| facts.definitions[d.0 as usize] != Some(self.item))
            || self
                .declaration
                .into_iter
                .is_some_and(|d| facts.definitions[d.0 as usize] != Some(self.iterator))
        {
            return Err("collection associated definitions differ from their contract");
        }
        self.validate_signature(facts)
    }
    fn validate_subject(self, facts: &Declarations) -> Result<(), &'static str> {
        if self.declaration.kind.next() {
            if self.subject != self.iterator {
                return Err("slice iterator impl has different Self");
            }
            return Ok(());
        }
        let referent = if self.declaration.kind.method() {
            self.subject
        } else {
            let Some(Kind::Reference {
                referent,
                mutable,
                region,
            }) = facts.types.kind(self.subject)
            else {
                return Err("collection Self is not a shared reference");
            };
            if region != RegionId::bound(0, 0)? || mutable != self.declaration.kind.mutable() {
                return Err("collection Self has a different impl lifetime");
            }
            referent
        };
        if self.declaration.kind.array() {
            let Some(Kind::ArrayTerm { element, length }) = facts.types.kind(referent) else {
                return Err("array impl lacks its declared extent");
            };
            let Some(ConstKind::Parameter(parameter)) = facts.types.const_kind(length) else {
                return Err("array impl extent is not a parameter");
            };
            let formal = facts
                .types
                .parameter(parameter)
                .ok_or("missing array extent parameter")?;
            if element != self.parameter
                || formal.owner != ItemId(self.declaration.definition.0)
                || formal.index != 2
                || formal.kind
                    != (ParameterKind::Const {
                        ty: TypeId::primitive(crate::types::Primitive::Usize),
                    })
                || self
                    .declaration
                    .extent
                    .and_then(|d| facts.generics.constants.get(&d).copied())
                    != Some(length)
            {
                return Err("array impl extent/element differs from its canonical schema");
            }
        } else if facts.types.kind(referent) != Some(Kind::Slice(self.parameter)) {
            return Err("slice impl has a different element");
        }
        Ok(())
    }
    fn validate_signature(self, facts: &Declarations) -> Result<(), &'static str> {
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
            bound_regions,
        }) = facts.types.kind(self.signature)
        else {
            return Err("collection method is not a safe Rust function");
        };
        let [receiver] = facts
            .types
            .list(parameters)
            .ok_or("missing collection parameters")?
        else {
            return Err("collection method has a different arity");
        };
        self.validate_lifetimes(facts)?;
        let next = self.declaration.kind.next();
        if bound_regions != if next { 2 } else { 1 } {
            return Err("collection method has a different lifetime binder");
        }
        if self.declaration.kind.method() || self.declaration.kind.next() {
            if !matches!(facts.types.kind(*receiver),Some(Kind::Reference {referent,mutable,region}) if referent==self.subject&&mutable==(next || self.declaration.kind.mutable())&&region==RegionId::bound(0,u16::from(next))?)
            {
                return Err("collection receiver differs from checked Self/permission");
            }
        } else if *receiver != self.subject {
            return Err("collection receiver differs from its owned Self");
        }
        if next {
            let canonical = facts
                .iteration
                .as_deref()
                .ok_or("missing iteration registry")?
                .canonical;
            let Some(Kind::Adt(owner, arguments)) = facts.types.kind(result) else {
                return Err("next result is not Option");
            };
            if owner != ItemId(canonical.option.0)
                || facts.types.arguments(arguments).is_none_or(|mut args| {
                    args.next() != Some(Argument::Type(self.item)) || args.next().is_some()
                })
            {
                return Err("next result has a different Option/Item");
            }
        } else if result != self.iterator {
            return Err("collection method returns a different iterator");
        }
        Ok(())
    }
    fn validate_lifetimes(self, facts: &Declarations) -> Result<(), &'static str> {
        let scheme = facts
            .lifetimes
            .functions
            .get(&self.declaration.method)
            .ok_or("missing collection lifetime scheme")?;
        let next = self.declaration.kind.next();
        let late: &[u16] = if next {
            &[1]
        } else if self.declaration.kind.method() {
            &[0]
        } else {
            &[]
        };
        if scheme.total != if next { 2 } else { 1 }
            || scheme.captures != u32::from(self.declaration.lifetime.is_some())
            || !scheme.early.is_empty()
            || scheme.late != late
            || !scheme.bounds.is_empty()
            || scheme.has_declared_late
        {
            return Err("collection lifetime scheme differs from its canonical role");
        }
        Ok(())
    }
}
