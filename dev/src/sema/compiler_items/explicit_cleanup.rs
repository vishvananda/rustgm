//! Explicit-call prohibition follows actual source member/override identities.
use crate::{
    resolve::model::{DefId, Target},
    sema::{associated::OwnerKind, model::Declarations},
    support::bindings::LangItem,
};

#[cfg(test)]
#[path = "../../../tests/invariants/prohibited_calls.rs"]
mod tests;

/// A denial rule, never an implementation or execution permission.
pub(in crate::sema) fn explicit_drop_member(
    facts: &Declarations,
    definition: DefId,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some(declaration) = catalog.language(LangItem::Drop) else {
        return Ok(false);
    };
    let Some(owner) = facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
    else {
        return Ok(false);
    };
    let implementation = match owner.kind {
        OwnerKind::Trait {
            definition: parent, ..
        } => {
            if parent != declaration.definition {
                return Ok(false);
            }
            None
        }
        OwnerKind::Impl {
            trait_type: Some(path),
            ..
        } => {
            let header = facts
                .generics
                .implementations
                .as_deref()
                .and_then(|registry| registry.header(owner.self_definition));
            let Some(header) = header else {
                // Source discovery can identify an unpaid match, but never replace
                // its canonical header or cache its absence as an unrelated owner.
                return match facts.node(path).and_then(|f| f.reference) {
                    Some(Target::Definition(other)) if other != declaration.definition => Ok(false),
                    _ => Err("explicit source Drop use requires a completed implementation header"),
                };
            };
            if header.owner != owner.self_definition {
                return Err("explicit source Drop use changed implementation ownership");
            }
            let Some((application, _)) = header.predicate.bound.trait_parts() else {
                return Ok(false);
            };
            let application = facts
                .types
                .trait_application(application)
                .ok_or("explicit source Drop use has an invalid trait application")?;
            if application.item.0 != declaration.definition.0 {
                return Ok(false);
            }
            if facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
            {
                return Err("explicit source Drop use has unexpected trait arguments");
            }
            Some(header)
        }
        OwnerKind::Impl {
            trait_type: None, ..
        } => return Ok(false),
        OwnerKind::ImportedImpl(_) => {
            facts.associated.require_imported(*owner)?;
            return Err("imported explicit Drop rules require their wider supported role profile");
        }
        // An imported inherent method is never the Drop override.
        OwnerKind::ImportedInherent(_) => {
            facts.associated.require_imported(*owner)?;
            return Ok(false);
        }
    };
    catalog.validated_language(LangItem::Drop)?;
    let Some(header) = implementation else {
        return Ok(true);
    };
    if !header
        .methods
        .iter()
        .any(|(_, actual)| *actual == Some(definition))
    {
        return Err("explicit source Drop use requires its exact declaration-to-override mapping");
    }
    Ok(true)
}

/// The exemption belongs to the genuine provided body, not an enclosing method
/// environment inherited by a closure, inline constant or subtrait body.
pub(in crate::sema) fn source_drop_body(
    facts: &Declarations,
    owner: Option<DefId>,
    body: crate::resolve::unit::NodeKey,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some(drop) = catalog.validated_language(LangItem::Drop)? else {
        return Ok(false);
    };
    let Some(owner) = owner else {
        return Ok(false);
    };
    if facts.associated.trait_owner(owner) != Some(drop.definition) {
        return Ok(false);
    }
    let function = facts
        .function_indices
        .get(&owner)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("provided Drop body lost its actual source function")?;
    Ok(function.definition == owner && function.body == Some(body))
}
