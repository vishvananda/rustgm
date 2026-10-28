//! Regular availability belongs to an actual indexed enum variant, never an expression.
use super::{Draft, Error, Package};
use crate::{
    resolve::{
        model::{DefKind, Definition, Resolution},
        unit::NodeKey,
    },
    support::hash::HashSet,
    syntax::ast::Kind,
};
pub(super) fn consume(
    package: &Package,
    facts: &Resolution,
    definition: &Definition,
    draft: &mut Draft,
    used: &mut HashSet<(NodeKey, bool)>,
) -> Result<bool, Error> {
    if definition.kind != DefKind::Variant {
        return Ok(false);
    }
    let key = definition
        .node
        .ok_or_else(|| Error::unlocated("variant availability has no source owner"))?;
    let regular = draft.attributes.get(&key).copied();
    if regular.is_none() && !draft.const_attributes.contains_key(&key) {
        return Ok(true);
    }
    draft.charge(12, 0)?;
    let error = |message| super::error(package, key, message);
    let unit = package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(|| Error::unlocated("variant availability has an invalid source file"))?;
    let Some(Kind::Variant(index)) = unit
        .configured
        .ast
        .nodes
        .get(key.node.get() as usize)
        .map(|n| n.kind)
    else {
        return Err(error(
            "variant availability requires its actual parsed variant",
        ));
    };
    let scope = facts
        .scopes
        .get(definition.scope.0 as usize)
        .ok_or_else(|| error("variant availability has an invalid resolved scope"))?;
    let owner = scope
        .nominal_self
        .and_then(|id| facts.definitions.get(id.0 as usize))
        .ok_or_else(|| error("variant availability has no actual enum owner"))?;
    if owner.kind != DefKind::Enum
        || owner.members != Some(definition.scope)
        || owner.node.is_none_or(|n| n.file != key.file)
        || unit.configured.active.get(key.node.get() as usize) != Some(&true)
        || unit
            .spelling(unit.configured.ast.variants[index as usize].name)
            .map_err(|_| error("variant availability has an invalid source name"))?
            != package.names.get(definition.name)
    {
        return Err(error(
            "variant availability changed its actual resolved source owner",
        ));
    }
    if draft.const_attributes.contains_key(&key) {
        return Err(error(
            "const library stability is not supported on enum variants",
        ));
    }
    if let Some(id) = regular {
        if draft
            .policies
            .get(id.get() - 1)
            .is_none_or(|p| p.origin != key)
        {
            return Err(error(
                "variant stability policy belongs to another current source owner",
            ));
        }
        used.insert((key, false));
    }
    Ok(true)
}
#[cfg(test)]
#[path = "../../../../tests/invariants/variant_availability.rs"]
mod tests;
