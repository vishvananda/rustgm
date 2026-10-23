//! Iteration plans retain distinct complete call contexts after ordinary selection.
use crate::support::hash::HashMap;
use crate::{
    resolve::{
        model::{BuiltinIteration, BuiltinIterationBlanket, DefId},
        unit::NodeKey,
    },
    types::{TypeId, parameters::predicates::Predicate},
};
pub(crate) mod collections;
pub(crate) mod range_originals;

#[derive(Clone, Copy)]
pub(crate) struct Call {
    pub callable: TypeId,
    pub signature: TypeId,
}
#[derive(Clone, Copy)]
pub(crate) struct ForPlan {
    pub into_iter: Call,
    pub next: Call,
    pub item: TypeId,
    pub iterator: TypeId,
    pub option: TypeId,
    pub some: super::members::VariantId,
    pub none: super::members::VariantId,
}
#[derive(Clone, Copy)]
pub(crate) struct Original {
    pub declaration: BuiltinIterationBlanket,
    pub parameter: TypeId,
    pub predicate: Predicate,
    pub signature: TypeId,
}
pub(crate) struct Registry {
    pub canonical: BuiltinIteration,
    pub originals: [Option<Original>; 2],
    pub plans: HashMap<NodeKey, ForPlan>,
    pub collections: Option<Box<collections::Registry>>,
    pub ranges: Option<Box<range_originals::Registry>>,
}
impl Registry {
    pub fn new(canonical: BuiltinIteration) -> Self {
        Self {
            canonical,
            originals: [None; 2],
            plans: HashMap::default(),
            collections: None,
            ranges: None,
        }
    }
    pub fn original(&self, definition: DefId) -> Option<Original> {
        self.originals
            .iter()
            .flatten()
            .find(|o| o.declaration.method == definition)
            .copied()
    }
    pub fn owns_impl(&self, definition: DefId) -> bool {
        self.originals
            .iter()
            .flatten()
            .any(|o| o.declaration.definition == definition)
    }
    pub fn owns_trait(&self, definition: DefId) -> bool {
        self.canonical
            .declarations
            .iter()
            .any(|declaration| declaration.definition == definition)
    }
    pub fn associated_definition(
        &self,
        owner: DefId,
        item: crate::types::ItemId,
    ) -> Result<Option<DefId>, &'static str> {
        use crate::support::iteration::{Blanket, Trait};
        if let Some(ranges) = self.ranges.as_deref()
            && let Some(original) = ranges
                .originals
                .iter()
                .find(|o| o.declaration.definition == owner)
        {
            if item == crate::types::ItemId(self.canonical.get(Trait::Iterator).item.0) {
                return Ok(Some(original.declaration.item));
            }
            return Err("integer range does not own the requested associated declaration");
        }
        if let Some(collections) = self.collections.as_deref()
            && let Some(definition) = collections.associated(self.canonical, owner, item)?
        {
            return Ok(Some(definition));
        }
        let Some(original) = self
            .originals
            .iter()
            .flatten()
            .find(|original| original.declaration.definition == owner)
        else {
            return Ok(None);
        };
        let declaration = self
            .canonical
            .get(if original.declaration.kind == Blanket::Identity {
                Trait::IntoIterator
            } else {
                Trait::Iterator
            });
        if item.0 == declaration.item.0 {
            return Ok(Some(original.declaration.item));
        }
        if declaration.into_iter == Some(DefId(item.0)) {
            return original
                .declaration
                .into_iter
                .map(Some)
                .ok_or("missing canonical IntoIter definition");
        }
        Err("iteration implementation does not own the requested associated declaration")
    }
}
use crate::{
    support::iteration::{Blanket, Trait},
    types::{
        ItemId, Kind, RegionId,
        parameters::{ParameterKind, predicates::Bound},
    },
};

impl Original {
    pub fn validate(self, facts: &super::model::Declarations) -> Result<(), &'static str> {
        let reference = self.validate_owner(facts)?;
        self.validate_prerequisite(facts)?;
        self.validate_signature(facts, reference)?;
        if !matches!(facts.definitions[self.declaration.method.0 as usize].and_then(|ty| facts.types.kind(ty)), Some(Kind::FunctionItem {item, signature, ..}) if item == ItemId(self.declaration.method.0) && signature == self.signature)
        {
            return Err("iteration original differs from its registered function");
        }
        Ok(())
    }

    fn validate_owner(self, facts: &super::model::Declarations) -> Result<bool, &'static str> {
        let registry = facts
            .iteration
            .as_deref()
            .ok_or("missing canonical iteration registry")?;
        if registry.canonical.blankets[self.declaration.kind as usize] != self.declaration {
            return Err("iteration original has a different canonical owner");
        }
        let reference = self.declaration.kind == Blanket::MutableReference;
        let Some(Kind::Parameter(parameter)) = facts.types.kind(self.parameter) else {
            return Err("iteration original requires a rigid parameter");
        };
        let identity = facts
            .types
            .parameter(parameter)
            .ok_or("missing iteration parameter identity")?;
        if identity.owner != ItemId(self.declaration.definition.0)
            || identity.index != u32::from(reference)
            || identity.kind != (ParameterKind::Type { sized: !reference })
        {
            return Err("iteration original has a different parameter schema");
        }
        let implementation = facts
            .generics
            .implementations
            .as_ref()
            .ok_or("missing iteration implementation inventory")?;
        if !implementation.is_validated(self.declaration.definition)
            || implementation
                .header(self.declaration.definition)
                .is_none_or(|h| h.predicate != self.predicate)
        {
            return Err("iteration original requires its exact checked implementation");
        }
        if facts
            .associated
            .schema(self.declaration.definition)
            .is_none_or(|s| {
                s.self_type != self.predicate.ty
                    || s.captures != u32::from(reference)
                    || !s.bounds.is_empty()
            })
        {
            return Err("iteration original has a different owner schema");
        }
        Ok(reference)
    }
    fn validate_prerequisite(self, facts: &super::model::Declarations) -> Result<(), &'static str> {
        let registry = facts
            .iteration
            .as_deref()
            .ok_or("missing canonical iteration registry")?;
        let iterator = registry.canonical.get(Trait::Iterator);
        let item = facts.definitions[self.declaration.item.0 as usize]
            .ok_or("missing iteration original Item")?;
        let Some(Kind::Projection(projection)) = facts.types.kind(item) else {
            return Err("iteration original Item is not its referent projection");
        };
        if projection.item != ItemId(iterator.item.0)
            || projection.subject != self.parameter
            || facts
                .types
                .arguments(projection.arguments)
                .is_none_or(|mut a| a.next().is_some())
        {
            return Err("iteration original has a different Item projection");
        }
        let prerequisite = Predicate {
            ty: self.parameter,
            bound: Bound::Trait(projection.trait_application),
        };
        let application = facts
            .types
            .trait_application(projection.trait_application)
            .ok_or("missing iteration prerequisite application")?;
        if application.item != ItemId(iterator.definition.0)
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|mut a| a.next().is_some())
        {
            return Err("iteration original has a different prerequisite owner");
        }
        let mut predicates = facts
            .generics
            .declared_predicates_in_scope(self.declaration.definition);
        if predicates.next().map(|p| p.value) != Some(prerequisite.into())
            || predicates.next().is_some()
        {
            return Err("iteration original does not retain its exact declared prerequisite");
        }
        if self
            .declaration
            .into_iter
            .is_some_and(|member| facts.definitions[member.0 as usize] != Some(self.parameter))
        {
            return Err("identity iteration original has a different IntoIter");
        }
        Ok(())
    }
    fn validate_signature(
        self,
        facts: &super::model::Declarations,
        reference: bool,
    ) -> Result<(), &'static str> {
        let registry = facts
            .iteration
            .as_deref()
            .ok_or("missing canonical iteration registry")?;
        let declaration = registry.canonical.get(if reference {
            Trait::Iterator
        } else {
            Trait::IntoIterator
        });
        let Some((application, _)) = self.predicate.bound.trait_parts() else {
            return Err("iteration original lacks a trait application");
        };
        let app = facts
            .types
            .trait_application(application)
            .ok_or("invalid iteration trait application")?;
        if app.item != ItemId(declaration.definition.0)
            || facts
                .types
                .arguments(app.arguments)
                .is_none_or(|mut a| a.next().is_some())
        {
            return Err("iteration original has a different trait application");
        }
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
            bound_regions,
        }) = facts.types.kind(self.signature)
        else {
            return Err("iteration original is not a safe Rust function");
        };
        let [receiver] = facts
            .types
            .list(parameters)
            .ok_or("missing iteration original parameters")?
        else {
            return Err("iteration original has incorrect arity");
        };
        if reference {
            self.validate_forwarding_contract(facts, *receiver, result, bound_regions)?;
        } else if *receiver != self.parameter
            || result != self.parameter
            || self.predicate.ty != self.parameter
            || bound_regions != 0
        {
            return Err("identity iteration original has a different contract");
        }
        Ok(())
    }
    fn validate_forwarding_contract(
        self,
        facts: &super::model::Declarations,
        receiver: TypeId,
        result: TypeId,
        bound_regions: u32,
    ) -> Result<(), &'static str> {
        let registry = facts
            .iteration
            .as_deref()
            .ok_or("missing canonical iteration registry")?;
        let Some(Kind::Reference {
            referent: inner,
            mutable: true,
            region: outer,
        }) = facts.types.kind(receiver)
        else {
            return Err("iteration forwarding original requires a mutable receiver");
        };
        let Some(Kind::Reference {
            referent,
            mutable: true,
            region,
        }) = facts.types.kind(inner)
        else {
            return Err("iteration forwarding original requires a mutable referent");
        };
        if referent != self.parameter
            || inner != self.predicate.ty
            || bound_regions != 2
            || outer != RegionId::bound(0, 1)?
            || region != RegionId::bound(0, 0)?
        {
            return Err("iteration forwarding original has a different binder contract");
        }
        let Some(Kind::Adt(option, arguments)) = facts.types.kind(result) else {
            return Err("iteration forwarding original requires Option");
        };
        if option != ItemId(registry.canonical.option.0) {
            return Err("iteration forwarding original result is not canonical Option");
        }
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("missing iteration result arguments")?;
        let Some(crate::types::arguments::Argument::Type(item)) = args.next() else {
            return Err("iteration result has no Item");
        };
        if args.next().is_some()
            || facts.definitions[self.declaration.item.0 as usize] != Some(item)
        {
            return Err("iteration result has a different Item");
        }
        Ok(())
    }
}
