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
pub(super) struct Flags {
    pub unsafe_: bool,
    pub constant: bool,
    pub auto_: bool,
}

/// Counts of written declarations, distinct from elaborated predicate counts.
pub(super) struct Written {
    pub parameters: usize,
    pub supertraits: usize,
}

/// Source vectors remain owned by their producer. A future imported adapter must
/// authenticate its complete declaration remap before borrowing equivalent rows.
pub(super) struct Trait<'a> {
    pub definition: DefId,
    pub parent: Option<DefId>,
    pub origin: DeclarationOrigin,
    pub self_definition: DefId,
    pub self_type: TypeId,
    pub flags: Flags,
    pub captures: u32,
    pub parameters: &'a [Parameter],
    pub members: &'a [DefId],
    pub inputs: Inputs<'a>,
    pub written: Written,
}
impl Trait<'_> {
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

    pub(super) fn validate_self(&self, store: &Store) -> Result<(), &'static str> {
        let Some(Kind::Parameter(id)) = store.kind(self.self_type) else {
            return Err("trait header has no exact Self parameter");
        };
        if store.parameter(id)
            != Some(crate::types::parameters::Parameter {
                owner: ItemId(self.self_definition.0),
                index: 0,
                kind: ParameterKind::Type { sized: false },
            })
        {
            return Err("trait header Self parameter belongs to another owner or slot");
        }
        Ok(())
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

#[cfg(test)]
#[path = "../../../tests/invariants/role_header_views.rs"]
mod tests;
