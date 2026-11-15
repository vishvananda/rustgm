//! The allocator attributes (`rustc_allocator`, `rustc_deallocator`,
//! `rustc_reallocator`, `rustc_allocator_zeroed` and
//! `rustc_allocator_zeroed_variant = "..."`) tell code generation which
//! allocator entry a function is, and `rustc_std_internal_symbol` gives a
//! function the session-internal symbol that the allocator shim defines or
//! calls. Neither meaning exists here: there is no allocator shim and no
//! internal symbol mangling. A bring-up session admits them with a recorded
//! waiver and every strict session rejects them. `rustc_conversion_suggestion`
//! only marks a method that rustc's type-mismatch diagnostics may suggest, so
//! it is admitted in every profile.
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

/// The waiver that admits the allocator and internal-symbol attributes.
const WAIVER: &str = "allocator and internal-symbol attributes are admitted without their code generation: no allocator shim or internal symbol mangling is generated";

/// Validate one allocator, internal-symbol or conversion-suggestion attribute
/// on its function and admit it.
/// # Errors
/// Rejects another form or owner, an ungated use, and the allocator and
/// internal-symbol attributes outside the bring-up profile.
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
    if name == "rustc_conversion_suggestion" || crate::support::bringup::waive(WAIVER) {
        return Ok(());
    }
    Err(error(format!(
        "{name} requires allocator shim code generation, which is not implemented"
    )))
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
