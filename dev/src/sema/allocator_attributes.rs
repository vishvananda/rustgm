//! The allocator attributes (`rustc_allocator`, `rustc_deallocator`,
//! `rustc_reallocator`, `rustc_allocator_zeroed` and
//! `rustc_allocator_zeroed_variant = "..."`) tell rustc's LLVM backend which
//! allocator entry a function is, so that it may assume a fresh, unaliased
//! result and pair allocations with their frees. `LowIR` has no allocator
//! annotations and this compiler's lowering draws no conclusion from them,
//! so after validation they have no further effect.
//! `rustc_std_internal_symbol` gives a function rustc's version-mangled
//! internal symbol, which declaration linkage reads
//! (`crate::support::linkage::internal_symbol`); the allocator shim and a
//! global allocator's functions meet alloc's declarations through it.
//! `rustc_conversion_suggestion` only marks a method that rustc's
//! type-mismatch diagnostics may suggest. All are admitted in every profile.
use crate::{
    resolve::{
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::Forest,
    },
};

/// Validate one allocator, internal-symbol or conversion-suggestion attribute
/// on its function and admit it.
/// # Errors
/// Rejects another form or owner and an ungated use.
pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let unit = report.package.unit(declaration.file);
    let error = |message: String| Error::at(declaration.file, attribute.span.start, message);
    let name = attribute
        .name(forest, unit.source.text())
        .ok_or_else(|| error("allocator attribute has no actual name".into()))?;
    let form = if name == "rustc_allocator_zeroed_variant" {
        string_value(unit, attribute, forest).map_err(|offset| {
            Error::at(
                declaration.file,
                offset,
                format!("{name} requires one string literal"),
            )
        })?
    } else {
        attribute.input == Input::Empty
    };
    if attribute.style != Style::Outer || attribute.unsafe_wrapper || !form {
        return Err(error(if name == "rustc_allocator_zeroed_variant" {
            format!("{name} requires an outer `{name} = \"...\"` attribute")
        } else {
            format!("{name} requires an outer word attribute")
        }));
    }
    crate::resolve::bindings::root_options::require_internal(
        report,
        allowances,
        declaration.file,
        attribute,
        forest,
    )?;
    if !matches!(unit.configured.ast.node(declaration.node).kind, Kind::Item(index)
        if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Function { .. }))
    {
        return Err(error(format!("{name} requires a function")));
    }
    Ok(())
}

/// Whether the attribute's input is `= "..."`, one string literal; the
/// offset of an invalid literal otherwise.
fn string_value(
    unit: &crate::resolve::unit::Unit,
    attribute: Attribute,
    forest: &Forest,
) -> Result<bool, u32> {
    let Input::Equals(range) = attribute.input else {
        return Ok(false);
    };
    if range.end != range.start + 1 {
        return Ok(false);
    }
    let Some(token) = forest.token(range.start) else {
        return Ok(false);
    };
    crate::literal::interpret(unit.source.text(), token)
        .map(|value| matches!(value, Some(crate::literal::Value::String(_))))
        .map_err(|error| error.offset)
}
