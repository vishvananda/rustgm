//! A configured struct owns its limit; instantiated array fields enforce it later.
use crate::{
    literal::{self, Value},
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    sema::{header_attributes::reserve_source_contracts, model::Declarations},
    support::{hash::HashMap, retained::Growth},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};
pub(in crate::sema) type Sources = Option<Box<HashMap<NodeKey, u64>>>;
pub(super) fn parse(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<u64, Error> {
    facts.reserve_must_use_source(key, attribute.span)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let unit = report.package.unit(key.file);
    if !matches!(unit.configured.ast.node(key.node).kind, Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Struct { .. }))
    {
        return Err(error("SIMD lane limit requires an actual struct"));
    }
    if !report.package.requested_internal_attributes()
        && !allowances.allows_at(
            key.file,
            attribute.span,
            &FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error("SIMD lane limit requires rustc_attrs"));
    }
    let Input::Equals(range) = attribute.input else {
        return Err(error("SIMD lane limit requires a name-value string"));
    };
    if attribute.unsafe_wrapper || range.end != range.start + 1 {
        return Err(error("SIMD lane limit requires an ordinary string value"));
    }
    let token = forest
        .token(range.start)
        .ok_or_else(|| error("SIMD lane limit requires a literal"))?;
    let Some(Value::String(bytes)) =
        literal::interpret(unit.source.text(), token).map_err(|failure| error(failure.message))?
    else {
        return Err(error("SIMD lane limit requires a string literal"));
    };
    let text =
        std::str::from_utf8(bytes.as_ref()).map_err(|_| error("invalid SIMD limit string"))?;
    text.parse::<u64>()
        .map_err(|_| error("SIMD lane limit must be a target-width non-negative decimal integer"))
}
pub(super) fn retain(
    facts: &mut Declarations,
    sources: &mut Sources,
    key: NodeKey,
    value: u64,
) -> Result<(), Error> {
    let error = |message| Error::at(key.file, 0, message);
    if sources
        .as_deref()
        .is_some_and(|rows| rows.contains_key(&key))
    {
        return Err(error("duplicate SIMD lane limit on configured struct"));
    }
    let (len, capacity) = sources
        .as_deref()
        .map_or((0, 0), |rows| (rows.len(), rows.capacity()));
    let growth = Growth::map::<NodeKey, u64>(len, capacity, 1);
    let owner = if sources.is_none() {
        size_of::<HashMap<NodeKey, u64>>()
    } else {
        0
    };
    reserve_source_contracts(facts, 1 + growth.work, owner.saturating_add(growth.bytes))?;
    let rows = sources.get_or_insert_default();
    rows.try_reserve(1)
        .map_err(|_| error("SIMD source limit allocation failed"))?;
    rows.insert(key, value);
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/simd_limits.rs"]
mod tests;
