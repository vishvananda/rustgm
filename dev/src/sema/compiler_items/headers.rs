//! Borrow actual trait declarations; structural validation grants no role or proof.
mod authored;
mod clone_signature;
use crate::{
    resolve::{body_origin::DeclarationOrigin, model::DefId},
    sema::{declaration_inputs::Inputs, generics::Parameter},
    support::constness::Constness,
    types::{
        ItemId, Kind, Store, TypeId,
        parameters::{ParameterKind, clauses::Clause, predicates::Bound},
    },
};
pub(super) use clone_signature::{Method, clone_method};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) struct Flags {
    pub unsafe_: bool,
    pub constant: bool,
    pub auto_: bool,
}

/// Counts of written declarations, distinct from elaborated predicate counts.
pub(in crate::sema) struct Written {
    pub parameters: usize,
    pub supertraits: usize,
}

/// Both adapters borrow their producer-owned vectors. Imported rows require the
/// actual completed catalog, canonical remap and installed declaration facts.
pub(in crate::sema) struct Trait<'a> {
    pub definition: DefId,
    pub parent: Option<DefId>,
    pub origin: DeclarationOrigin,
    pub self_definition: DefId,
    pub self_type: TypeId,
    pub flags: Flags,
    pub attributes: crate::resolve::bindings::HeaderAttributes,
    pub captures: u32,
    pub parameters: &'a [Parameter],
    pub members: &'a [DefId],
    pub inputs: Inputs<'a>,
    pub written: Written,
}

impl super::Catalog {
    /// Borrow only completed role declarations. Sizing uses its existing
    /// separate validation ledger; this supplies no implementation evidence.
    pub(in crate::sema) fn header_role(
        &self,
        role: crate::support::bindings::LangItem,
    ) -> Result<Option<crate::resolve::bindings::Declaration>, &'static str> {
        use crate::support::bindings::LangItem;
        let ordinal = match role {
            LangItem::Sized => Some(0),
            LangItem::MetaSized => Some(1),
            LangItem::PointeeSized => Some(2),
            _ => None,
        };
        let Some(ordinal) = ordinal else {
            return self.validated_language(role);
        };
        let Some(declaration) = self.language(role) else {
            return Ok(None);
        };
        if self.sizing.as_ref().map(|sizing| sizing[ordinal]) != Some(declaration.definition) {
            return Err("source sizing role has no exact completed header");
        }
        Ok(Some(declaration))
    }
}
impl Trait<'_> {
    /// Role hierarchy checks use the actual canonical declaration contract.
    /// Full elaborated inputs remain borrowed separately for export/environments.
    pub(super) fn canonical_inputs<'a>(
        &self,
        facts: &'a crate::sema::model::Declarations,
    ) -> Result<Inputs<'a>, &'static str> {
        self.validate_self(&facts.types)?;
        let header = facts
            .generics
            .traits
            .as_deref()
            .and_then(|contracts| contracts.header(ItemId(self.definition.0)))
            .ok_or("source role lacks its actual canonical trait header")?;
        if header.self_type != self.self_type {
            return Err("source role canonical header has a different actual Self identity");
        }
        match self.origin {
            DeclarationOrigin::Source(_) => {
                header.inputs.source_ref()?;
            }
            DeclarationOrigin::Imported(id) => {
                let catalog = facts
                    .crates
                    .as_deref()
                    .and_then(|linkage| linkage.origin_catalog())
                    .ok_or("imported role canonical header lacks its actual catalog")?;
                catalog.validate_declaration(
                    id,
                    self.definition,
                    crate::resolve::model::DefKind::Trait,
                )?;
                header.inputs.require_catalog(catalog, self.definition)?;
            }
        }
        Ok(header.inputs.view())
    }

    pub(super) fn profile(
        &self,
        store: &Store,
        flags: Flags,
        parameters: usize,
    ) -> Result<(), &'static str> {
        if self.parent.is_some() || self.flags != flags || self.captures != 0 {
            return Err("source trait header is outside the pinned provider profile");
        }
        if self.parameters.len() != parameters {
            return Err("source trait role has unsupported generic parameters");
        }
        self.validate_self(store)
    }

    pub(in crate::sema) fn validate_self(&self, store: &Store) -> Result<(), &'static str> {
        require_self(store, self.self_type, self.self_definition)
    }

    pub(super) fn requires(
        &self,
        store: &Store,
        expected: DefId,
        sized: Option<DefId>,
        constness: Constness,
    ) -> bool {
        self.inputs.iter().any(|input| {
            let Clause::Predicate(predicate) = input.value else {
                return false;
            };
            if predicate.ty != self.self_type {
                return false;
            }
            if sized == Some(expected)
                && constness == Constness::Ordinary
                && predicate.bound == Bound::Sized
            {
                return true;
            }
            predicate.bound.trait_parts().is_some_and(|(id, effect)| {
                effect == constness
                    && store.trait_application(id).is_some_and(|application| {
                        application.item.0 == expected.0
                            && store
                                .arguments(application.arguments)
                                .is_some_and(|arguments| arguments.len() == 0)
                    })
            })
        })
    }
}

/// Authenticate the actual rigid Self declaration; this establishes no trait proof.
pub(in crate::sema) fn require_self(
    store: &Store,
    self_type: TypeId,
    self_definition: DefId,
) -> Result<(), &'static str> {
    let Some(Kind::Parameter(id)) = store.kind(self_type) else {
        return Err("trait header has no exact Self parameter");
    };
    if store.parameter(id)
        != Some(crate::types::parameters::Parameter {
            owner: ItemId(self_definition.0),
            index: 0,
            kind: ParameterKind::Type { sized: false },
        })
    {
        return Err("trait header Self parameter belongs to another owner or slot");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/role_header_views.rs"]
mod tests;
