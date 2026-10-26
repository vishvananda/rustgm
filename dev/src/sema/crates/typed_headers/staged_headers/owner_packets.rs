//! The actual scope order supplies imported trait/impl owners and complete members.
use super::{Error, Headers, Scope, Stage, error, mapped_source, s};
use crate::sema::associated::{ImportedOwner, Owner, OwnerKind, Schema};

impl Stage {
    pub(super) fn associated_rows(
        &mut self,
        context: &mapped_source::Context<'_>,
        store: &crate::types::Store,
        budget: &mut Headers,
    ) -> Result<Vec<ImportedOwner>, Error> {
        self.graph.require(
            context.imported,
            context.catalog,
            context.descriptor,
            budget,
        )?;
        let rows = context.descriptor.rows();
        let count = rows.traits.len().saturating_add(rows.implementations.len());
        budget.reserve(
            count
                .saturating_mul(size_of::<ImportedOwner>())
                .saturating_add(
                    rows.members
                        .len()
                        .saturating_mul(size_of::<crate::resolve::model::DefId>()),
                )
                .saturating_add(
                    rows.region_bounds
                        .len()
                        .saturating_mul(size_of::<crate::types::outlives::Pair>()),
                ),
        )?;
        let mut result = Vec::with_capacity(count);
        for (raw, scope) in rows.scopes.iter().zip(&mut self.scopes) {
            budget.charge(1)?;
            match rows.header(raw.owner).map_err(error)? {
                Some(s::HeaderRef::Implementation(index)) => result.push(receiver_owner(
                    &self.graph,
                    store,
                    scope,
                    &rows.implementations[index as usize],
                )?),
                Some(s::HeaderRef::Trait(index)) => result.push(trait_owner(
                    context,
                    &self.graph,
                    &rows.traits[index as usize],
                    budget,
                )?),
                _ => {}
            }
        }
        if result.len() != count {
            return Err(error(
                "source associated scopes omitted complete trait/impl owner coverage",
            ));
        }
        // The outer vector is scratch. Inner schema/member allocations move into
        // the real inventory; they were counted by their actual producers.
        budget.charge(1)?;
        Ok(result)
    }
}

fn receiver_owner(
    graph: &mapped_source::Graph,
    store: &crate::types::Store,
    scope: &mut Scope,
    raw: &s::ImplHeader,
) -> Result<ImportedOwner, Error> {
    if raw.members.start != raw.members.end {
        return Err(error(
            "imported impl members require their wider exact conformance profile",
        ));
    }
    let prepared = scope.implementation.take().ok_or_else(|| {
        error("source implementation omitted its already prepared actual owner/schema")
    })?;
    let owner = Owner {
        source: None,
        self_definition: scope.definition,
        kind: OwnerKind::ImportedImpl(prepared.owner),
    };
    graph
        .implementation(store, owner, prepared.predicate)
        .map_err(error)?;
    Ok(ImportedOwner {
        owner,
        schema: Schema {
            self_type: prepared.self_type,
            captures: prepared.captures,
            bounds: prepared.bounds,
        },
        members: Vec::new(),
    })
}

fn trait_owner(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    raw: &s::TraitHeader,
    budget: &mut Headers,
) -> Result<ImportedOwner, Error> {
    let rows = context.descriptor.rows();
    let flags = raw.flags.bits();
    let definition = context.definition(raw.declaration, budget)?;
    let self_definition = context.definition(raw.self_definition, budget)?;
    let owner = Owner {
        source: None,
        self_definition,
        kind: OwnerKind::Trait {
            definition,
            auto_: flags & s::TraitFlags::AUTO != 0,
            unsafe_: flags & s::TraitFlags::UNSAFE != 0,
            constant: flags & s::TraitFlags::CONST != 0,
        },
    };
    let bounds = raw
        .region_bounds
        .slice(&rows.region_bounds)
        .map_err(error)?
        .iter()
        .map(|bound| {
            budget.charge(1)?;
            Ok((graph.region(bound.longer)?, graph.region(bound.shorter)?))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let members = raw
        .members
        .slice(&rows.members)
        .map_err(error)?
        .iter()
        .map(|&member| context.definition(member, budget))
        .collect::<Result<Vec<_>, _>>()?;
    budget.observed.vector(&bounds);
    budget.observed.vector(&members);
    Ok(ImportedOwner {
        owner,
        schema: Schema {
            self_type: graph.ty(raw.self_type)?,
            captures: raw.region_captures,
            bounds,
        },
        members,
    })
}
