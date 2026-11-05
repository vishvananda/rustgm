//! Defining contexts retain hidden types separately from published result identities.
mod applications;
mod async_output;
mod asyncshape;
pub(crate) use applications::StorageFailure;
mod captures;
mod future_contract;
mod futureform;
pub(crate) use future_contract::check as check_future_contract;
mod closing;
pub(in crate::sema) mod overrides;
pub(in crate::sema) mod refinement;
mod reveal;
#[cfg(test)]
#[path = "../../tests/invariants/opclaims.rs"]
mod symbolic_tests;
use crate::support::hash::HashMap;
use crate::{
    resolve::{body_origin::DeclarationOrigin, model::DefId},
    types::{TypeId, opaque::OpaqueId, parameters::clauses::Clause},
};

pub struct Declaration {
    pub owner: DefId,
    pub(crate) origin: DeclarationOrigin,
    pub application: TypeId,
    pub bounds: Vec<Clause>,
    pub hidden: Option<TypeId>,
    pub(crate) inferred: Option<TypeId>,
    pub(crate) free_regions: Vec<crate::types::RegionId>,
}

impl Declaration {
    pub(crate) fn require_current(
        &self,
        facts: &super::model::Declarations,
        id: OpaqueId,
    ) -> Result<(), &'static str> {
        self.site(facts)?;
        if !matches!(facts.types.kind(self.application),Some(crate::types::Kind::Opaque(actual,_)) if actual==id)
        {
            return Err("opaque result key changed its actual canonical identity");
        }
        match self.origin {
            DeclarationOrigin::Source(_) => {
                if facts
                    .crates
                    .as_deref()
                    .and_then(super::crates::Linkage::origin_catalog)
                    .map(|catalog| catalog.declaration_if_imported(self.owner))
                    .transpose()?
                    .flatten()
                    .is_some()
                {
                    return Err("imported opaque result cannot acquire source provenance");
                }
                Ok(())
            }
            DeclarationOrigin::Imported(_) => facts
                .associated
                .imported_owner()
                .ok_or("imported opaque lost its actual canonical owner graph")?
                .require_opaque(facts, id, self),
        }
    }
    pub(crate) fn source(&self) -> Result<crate::resolve::unit::NodeKey, &'static str> {
        self.origin
            .source()
            .ok_or("imported opaque requires its authenticated defining operation")
    }

    pub(crate) fn site(
        &self,
        facts: &super::model::Declarations,
    ) -> Result<crate::resolve::body_origin::ProofSite, &'static str> {
        match self.origin {
            DeclarationOrigin::Source(source) => Ok(source.into()),
            DeclarationOrigin::Imported(origin) => {
                let linkage = facts
                    .crates
                    .as_deref()
                    .ok_or("imported opaque lost its actual linkage")?;
                linkage.require_original_store(&facts.types)?;
                let catalog = linkage
                    .origin_catalog()
                    .ok_or("imported opaque lost its actual catalog")?;
                catalog.validate_declaration(
                    origin,
                    self.owner,
                    crate::resolve::model::DefKind::Function,
                )?;
                let site = catalog.declaration_site(origin, 0)?;
                catalog.declaration_proof_site(origin, site)
            }
        }
    }
}

/// A proved override result in the impl parent and trait method parameter context.
pub(crate) struct Implementation {
    pub ty: TypeId,
    pub parent_regions: u32,
    /// Trait method source slots mapped to this template's lifetime coordinates.
    pub regions: std::sync::Arc<[crate::types::RegionId]>,
}

#[derive(Default)]
pub struct Opaques {
    pub declarations: HashMap<OpaqueId, Declaration>,
    pub owners: HashMap<DefId, Vec<OpaqueId>>,
    pub body_results: HashMap<DefId, TypeId>,
    pub(crate) implementations: HashMap<(DefId, OpaqueId), Implementation>,
    storage_cursor: usize,
    storage_preparation: Option<Box<applications::Preparation>>,
}
