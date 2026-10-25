//! Borrow real source functions or authenticate an installed source-free method.
use super::{Declarations, DefId, Function};
use crate::{
    resolve::body_origin::DeclarationOrigin,
    types::{Kind, TypeId},
};

/// No syntax or body classification is inferred from a signature.
#[derive(Clone, Copy)]
pub(in crate::sema) struct Method {
    pub definition: DefId,
    pub signature: TypeId,
    pub written_signature: TypeId,
}
impl Method {
    pub(in crate::sema) fn source(facts: &Declarations, function: Function) -> Self {
        let written_signature = if facts.associated.families.is_empty() {
            function.signature
        } else {
            match facts
                .unreduced
                .node(function.node)
                .and_then(|ty| facts.types.kind(ty))
            {
                Some(Kind::FunctionItem { signature, .. }) => signature,
                _ => function.signature,
            }
        };
        Self {
            definition: function.definition,
            signature: function.signature,
            written_signature,
        }
    }

    pub(in crate::sema) fn imported(
        facts: &mut Declarations,
        definition: DefId,
    ) -> Result<Self, &'static str> {
        let mut steps = 1_usize;
        let result = Self::imported_header(facts, definition, &mut steps);
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
        result
    }
    fn imported_header(
        facts: &Declarations,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<Self, &'static str> {
        if facts.function_indices.contains_key(&definition) {
            return Err("imported conformance cannot replace an actual source method");
        }
        let owner = facts
            .associated
            .owner(definition)
            .and_then(|id| facts.associated.get(id))
            .copied()
            .ok_or("imported method lacks its actual associated owner")?;
        facts.associated.require_imported(owner)?;
        let header = facts
            .function_header(definition)?
            .ok_or("imported method lacks its authenticated completed header")?;
        if !matches!(header.declaration, DeclarationOrigin::Imported(_)) {
            return Err("imported method changed its actual declaration provenance");
        }
        let scheme = facts
            .lifetimes
            .functions
            .get(&definition)
            .ok_or("imported method lacks its installed lifetime scheme")?;
        let environment = facts
            .generics
            .environments
            .get(&definition)
            .ok_or("imported method lacks its installed declaration environment")?;
        if scheme.total != header.universals
            || header
                .environment
                .is_none_or(|actual| !std::sync::Arc::ptr_eq(actual, environment))
            || !facts.generics.parameters.contains_key(&definition)
            || facts.generics.predicates.contains_key(&definition)
        {
            return Err("imported method changed its installed parameter/input/lifetime facts");
        }
        let linkage = facts
            .crates
            .as_ref()
            .ok_or("imported method lacks its actual linkage")?;
        linkage.require_source_method(definition, scheme, steps)?;
        linkage.original_inputs(definition, false)?;
        Ok(Self {
            definition,
            signature: header.signature,
            written_signature: header.signature,
        })
    }
}
