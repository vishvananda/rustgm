//! Native identities for evaluated promotions. A relocation has no execution permission.
use super::{CrateIdentity, Linkage};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    sema::{
        members::VariantId,
        model::Declarations,
        values::{Constant, Evaluator, Failure, LocationId},
    },
    support::{
        crates::{
            generics::{Range, source as s},
            owner_catalog::DeclarationRef,
        },
        hash::HashMap,
    },
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId,
        closure::{CallKind, ClosureId},
        instance::Instance,
    },
};

#[derive(PartialEq, Eq)]
pub(crate) struct AllocationBinding {
    pub symbol: String,
    pub anchor: DeclarationRef,
    pub imported: bool,
}

#[derive(PartialEq, Eq)]
pub(crate) struct HandlerBinding {
    pub symbol: String,
    pub identity: u32,
    pub signature: TypeId,
    pub source: Option<NodeKey>,
}

#[derive(Default)]
pub(super) struct Registry {
    allocations: HashMap<LocationId, AllocationBinding>,
    handlers: HashMap<TypeId, HandlerBinding>,
    exported_allocations: Vec<LocationId>,
    exported_handlers: Vec<TypeId>,
    exported_closures: HashMap<ClosureId, TypeId>,
}

impl Linkage {
    pub(crate) fn promoted_allocation(&self, location: LocationId) -> Option<&AllocationBinding> {
        self.promotions.as_deref()?.allocations.get(&location)
    }

    pub(crate) fn promoted_handler(&self, ty: TypeId) -> Option<&HandlerBinding> {
        self.promotions.as_deref()?.handlers.get(&ty)
    }

    pub(crate) fn promoted_closure(&self, closure: ClosureId) -> Option<&HandlerBinding> {
        let registry = self.promotions.as_deref()?;
        registry
            .handlers
            .get(registry.exported_closures.get(&closure)?)
    }

    pub(crate) fn exported_promotions(&self) -> &[LocationId] {
        self.promotions
            .as_deref()
            .map_or(&[], |registry| &registry.exported_allocations)
    }

    pub(crate) fn exported_handlers(&self) -> &[TypeId] {
        self.promotions
            .as_deref()
            .map_or(&[], |registry| &registry.exported_handlers)
    }

    pub(super) fn bind_promotion(
        &mut self,
        identity: &CrateIdentity,
        location: LocationId,
        anchor: DeclarationRef,
        imported: bool,
    ) -> Result<(), &'static str> {
        let registry = self.promotions.get_or_insert_with(Box::default);
        let binding = AllocationBinding {
            symbol: format!("{}$promotion${}", identity.symbol_prefix(), anchor.0),
            anchor,
            imported,
        };
        if registry.allocations.contains_key(&location) {
            return Err("source promotion already has an actual native allocation");
        }
        registry.allocations.insert(location, binding);
        if !imported {
            registry.exported_allocations.push(location);
        }
        Ok(())
    }

    pub(super) fn bind_promoted_handler(
        &mut self,
        identity: &CrateIdentity,
        ty: TypeId,
        ordinal: u32,
        signature: TypeId,
        source: Option<(ClosureId, NodeKey)>,
    ) -> Result<(), &'static str> {
        let registry = self.promotions.get_or_insert_with(Box::default);
        let binding = HandlerBinding {
            symbol: format!("{}$promotion_fn${ordinal}", identity.symbol_prefix()),
            identity: ordinal,
            signature,
            source: source.map(|(_, source)| source),
        };
        if registry.handlers.contains_key(&ty) {
            return Err("source promotion already has an actual native handler");
        }
        registry.handlers.insert(ty, binding);
        if let Some((closure, _)) = source {
            if registry.exported_closures.insert(closure, ty).is_some() {
                return Err("source promotion closure has duplicate native handlers");
            }
            registry.exported_handlers.push(ty);
        }
        Ok(())
    }
}

pub(super) fn record(facts: &Declarations, ty: TypeId) -> Option<(TypeId, ItemId, VariantId)> {
    let Kind::Reference {
        region: RegionId::STATIC,
        referent,
        mutable: false,
    } = facts.types.kind(ty)?
    else {
        return None;
    };
    let Kind::Adt(owner, GenericArgs::EMPTY) = facts.types.kind(referent)? else {
        return None;
    };
    if facts.aggregates.get(owner)?.kind != crate::types::aggregate::Kind::Struct {
        return None;
    }
    let &[variant] = facts.members.variants(owner) else {
        return None;
    };
    let fields = facts.members.fields(variant)?;
    if fields.is_empty()
        || !fields
            .iter()
            .all(|field| matches!(facts.types.kind(field.ty), Some(Kind::Function { .. })))
    {
        return None;
    }
    Some((referent, owner, variant))
}

#[derive(Clone, Copy)]
pub(super) struct Imported<'rows> {
    pub provider: &'rows CrateIdentity,
    pub rows: s::ValidatedView<'rows>,
    pub definition: DefId,
    pub declaration: DeclarationRef,
    pub anchor: DeclarationRef,
    pub handlers: Range,
}

#[derive(Default)]
struct ImportedBindings<'rows> {
    allocations: HashMap<(&'rows CrateIdentity, DeclarationRef), Constant>,
    handlers: HashMap<(&'rows CrateIdentity, u32), Instance>,
}

pub(super) fn seed(
    report: &Report,
    facts: &mut Declarations,
    promotions: &[Imported<'_>],
) -> Result<(), Error> {
    let mut bindings = ImportedBindings::default();
    for &promotion in promotions {
        let constant = bindings
            .reference(report, facts, promotion)
            .map_err(Error::unlocated)?;
        let mut evaluator = Evaluator::new(report, facts);
        super::imported_values::complete_value(&mut evaluator, promotion.definition, constant)?;
    }
    Ok(())
}

impl<'rows> ImportedBindings<'rows> {
    fn reference(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        promotion: Imported<'rows>,
    ) -> Result<Constant, &'static str> {
        let key = (promotion.provider, promotion.anchor);
        if promotion.anchor != promotion.declaration {
            return self
                .allocations
                .get(&key)
                .copied()
                .ok_or("imported promotion alias has no actual allocation");
        }
        let ty = facts.definitions[promotion.definition.0 as usize]
            .ok_or("imported promotion has no actual reference type")?;
        let (referent, _, variant) =
            record(facts, ty).ok_or("imported promotion lost its actual nominal fields")?;
        let fields: Vec<_> = facts
            .members
            .fields(variant)
            .ok_or("imported promotion lost its actual member inventory")?
            .iter()
            .map(|field| field.ty)
            .collect();
        let targets = promotion
            .handlers
            .slice(&promotion.rows.cold().promoted_handlers)?;
        if fields.len() != targets.len() {
            return Err("imported promotion changed its actual record arity");
        }
        let values = fields
            .into_iter()
            .zip(targets)
            .map(|(signature, target)| {
                self.handler(facts, promotion.provider, target.identity, signature)
                    .map(Constant::function)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut evaluator = Evaluator::new(report, facts);
        let product = evaluator
            .fields(&values, Some(variant))
            .map_err(Failure::message)?;
        let reference = evaluator
            .allocation(Some(product), referent)
            .map_err(Failure::message)?
            .restrict_shared();
        let location = reference
            .location()
            .ok_or("imported promotion has no typed address")?;
        facts.crates.get_or_insert_default().bind_promotion(
            promotion.provider,
            location,
            promotion.anchor,
            true,
        )?;
        self.allocations.insert(key, reference);
        Ok(reference)
    }

    fn handler(
        &mut self,
        facts: &mut Declarations,
        provider: &'rows CrateIdentity,
        ordinal: u32,
        signature: TypeId,
    ) -> Result<Instance, &'static str> {
        let key = (provider, ordinal);
        if let Some(&handler) = self.handlers.get(&key) {
            if !matches!(facts.types.kind(handler.ty()), Some(Kind::Closure { signature: actual, .. }) if actual == signature)
            {
                return Err("imported handler changed its canonical signature");
            }
            return Ok(handler);
        }
        let identity = facts.types.declare_closure(CallKind::Shared)?;
        let ty = facts.types.intern(Kind::Closure {
            identity,
            arguments: GenericArgs::EMPTY,
            signature,
            environment: TypeId::UNIT,
        })?;
        let handler = Instance::closure(&facts.types, ty)?;
        facts
            .crates
            .get_or_insert_default()
            .bind_promoted_handler(provider, ty, ordinal, signature, None)?;
        self.handlers.insert(key, handler);
        Ok(handler)
    }
}
