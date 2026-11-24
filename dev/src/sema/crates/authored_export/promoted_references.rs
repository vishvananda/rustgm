//! Completed shared promotions expose only the actual allocation and handlers.
use super::{CrateIdentity, DeclarationRef, Declarations, Stage, g, s};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::values::{Constant, LocationId, Origin, Reader, View},
    types::{GenericArgs, ItemId, Kind, TypeId, closure::ClosureId},
};

pub(super) fn referent(facts: &Declarations, ty: TypeId) -> Option<(TypeId, ItemId)> {
    super::super::promotions::record(facts, ty).map(|(referent, owner, _)| (referent, owner))
}

struct Reference {
    location: LocationId,
    owner: ItemId,
    handlers: Vec<(TypeId, TypeId, ClosureId, NodeKey)>,
}

fn reference(facts: &Declarations, ty: TypeId, value: Constant) -> Result<Reference, &'static str> {
    let (referent, owner) =
        referent(facts, ty).ok_or("source promotion lost its actual shared record")?;
    let reader = Reader::new(facts);
    let View::Reference(location) = reader.view(value)? else {
        return Err("source promotion lost its actual reference");
    };
    let allocation = reader.location(location)?;
    if value.metadata().is_some()
        || allocation.ty != referent
        || !matches!(allocation.origin, Origin::Allocation)
    {
        return Err("source promotion changed its actual complete allocation");
    }
    let allocation = allocation
        .value
        .ok_or("source promotion lost its completed fields")?;
    let &[variant] = facts.members.variants(owner) else {
        return Err("source promotion lost its actual record fields");
    };
    let fields = facts
        .members
        .fields(variant)
        .ok_or("source promotion lost its actual record fields")?;
    if !matches!(reader.view(allocation)?, View::Product { length, .. }
        if length == fields.len() as u64)
    {
        return Err("source promotion changed its completed field count");
    }
    let mut handlers = Vec::with_capacity(fields.len());
    for (ordinal, field) in fields.iter().enumerate() {
        let View::Function(instance) = reader.view(reader.field(allocation, ordinal as u64)?)?
        else {
            return Err("source promoted field lost its checked function value");
        };
        let Some(Kind::Closure {
            identity,
            signature,
            environment: TypeId::UNIT,
            arguments: GenericArgs::EMPTY,
        }) = facts.types.kind(instance.ty())
        else {
            return Err("source promotion requires a checked noncapturing closure");
        };
        if signature != field.ty || facts.types.contains_parameters(instance.ty()) != Some(false) {
            return Err("source promoted closure changed its actual reified signature");
        }
        let source = facts
            .closures
            .identity(identity)
            .filter(|body| body.signature == signature)
            .ok_or("source promotion lost the actual handler body")?
            .semantic_source();
        handlers.push((instance.ty(), signature, identity, source));
    }
    Ok(Reference {
        location,
        owner,
        handlers,
    })
}

impl Stage {
    pub(super) fn promoted_reference(
        &mut self,
        facts: &mut Declarations,
        identity: &CrateIdentity,
        declaration: DeclarationRef,
        ty: TypeId,
        value: Constant,
    ) -> Result<s::ConstantBody, &'static str> {
        let Reference {
            location,
            owner,
            handlers,
        } = reference(facts, ty, value)?;
        let start = offset(self.cold()?.promoted_handlers.len())?;
        if let Some(prior) = facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.promoted_allocation(location))
        {
            if prior.imported {
                return Err("source promotion references another provider's allocation");
            }
            return Ok(s::ConstantBody::PromotedReference {
                anchor: prior.anchor,
                handlers: g::Range { start, end: start },
            });
        }
        let nominal = self.declaration(DefId(owner.0))?;
        self.charge(self.rows.nominals.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
        let index = self
            .rows
            .nominals
            .binary_search_by_key(&nominal.0, |row| row.declaration.0)
            .map_err(|_| "source promotion has no actual record header")?;
        let fields = self.rows.nominals[index].fields;
        if handlers.len() != (fields.end - fields.start) as usize {
            return Err("source promotion changed its typed record fields");
        }
        self.charge(handlers.len())?;
        self.reserve(
            handlers
                .len()
                .saturating_mul(8 * size_of::<s::PromotedHandler>() + 192)
                .saturating_add(192),
        )?;
        let linkage = facts.crates.get_or_insert_default();
        linkage.bind_promotion(identity, location, declaration, false)?;
        for (index, (closure, signature, owner, source)) in handlers.into_iter().enumerate() {
            let field = self.rows.fields[fields.start as usize + index].ty;
            let ordinal = if let Some(prior) = linkage.promoted_handler(closure) {
                if prior.source.is_none() || prior.signature != signature {
                    return Err("source promoted closure changed its native owner or signature");
                }
                prior.identity
            } else {
                let ordinal = offset(self.cold()?.promoted_handlers.len())?;
                linkage.bind_promoted_handler(
                    identity,
                    closure,
                    ordinal,
                    signature,
                    Some((owner, source)),
                )?;
                ordinal
            };
            self.cold()?.promoted_handlers.push(s::PromotedHandler {
                identity: ordinal,
                signature: field,
            });
        }
        Ok(s::ConstantBody::PromotedReference {
            anchor: declaration,
            handlers: g::Range {
                start,
                end: offset(self.cold()?.promoted_handlers.len())?,
            },
        })
    }
}

fn offset(length: usize) -> Result<u32, &'static str> {
    u32::try_from(length).map_err(|_| "source promotion handler ordinal overflow")
}
