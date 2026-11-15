//! `#[may_dangle]` marks a parameter that an `unsafe impl Drop` promises not
//! to read, which lets rustc's drop check skip that argument. Drop checking
//! here has no such opt-out: it still requires every argument of a marked
//! impl, which rejects more programs than rustc and accepts none that rustc
//! rejects. The attribute is therefore admitted only as a bring-up waiver.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::{GenericFlavor, ItemKind},
        token_tree::Forest,
    },
};

/// rustc's placement rule for the attribute.
pub(super) const MISPLACED: &str =
    "`#[may_dangle]` must be applied to a lifetime or type generic parameter in `Drop` impl";

/// Validate one `#[may_dangle]`: its word form, its feature, and that its
/// owner is a lifetime or type parameter of a trait impl. The `Drop` impl
/// headers check the trait and the impl's safety.
/// # Errors
/// Rejects any other form, an ungated use and any other owner.
pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    (owner, declaration): (NodeKey, NodeKey),
    attribute: Attribute,
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.style != Style::Outer
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
    {
        return Err(error("may_dangle requires an outer word attribute"));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("may_dangle attribute has no actual name token"))?
        .span;
    if !report.package.requested_dropck_eyepatch().map_err(error)?
        && !allowances.allows_at(
            declaration.file,
            name,
            &FeatureName::new("dropck_eyepatch").map_err(error)?,
        )?
    {
        return Err(error(
            "`#[may_dangle]` requires dropck_eyepatch or exact token allowance",
        ));
    }
    let ast = &report.package.unit(declaration.file).configured.ast;
    let parameter = matches!(ast.node(declaration.node).kind, Kind::Generic(index)
        if ast.generics.get(index as usize).is_some_and(|parameter| parameter.flavor != GenericFlavor::Const));
    if !parameter {
        return Err(error(MISPLACED));
    }
    // The parser keeps a parameter's attributed node in its owner's list.
    crate::sema::header_attributes::reserve_source_contracts(facts, ast.items.len(), 0)?;
    let in_trait_impl = ast.items.iter().any(|item| {
        matches!(
            item.kind,
            ItemKind::Impl {
                trait_path: Some(_),
                ..
            }
        ) && ast.list(item.generics).contains(&owner.node)
    });
    if !in_trait_impl {
        return Err(error(MISPLACED));
    }
    Ok(())
}

/// Whether the trait impl at `source` marks any configured parameter.
/// # Errors
/// Rejects an attribute with arguments.
pub(super) fn marked(report: &Report, source: NodeKey) -> Result<bool, Error> {
    let unit = report.package.unit(source.file);
    let ast = &unit.configured.ast;
    let Kind::Item(index) = ast.node(source.node).kind else {
        return Err(Declarations::error(
            report,
            source,
            "missing impl item syntax",
        ));
    };
    for &node in ast.list(ast.items[index as usize].generics) {
        if unit.configured.active.get(node.get() as usize) == Some(&true)
            && unit
                .word_attribute(Some(node), "may_dangle")
                .map_err(|error| Error::at(source.file, error.offset, error.message))?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Admit a marked `Drop` impl: a recorded waiver in bring-up, a rejection
/// in every strict compilation.
/// # Errors
/// Outside the bring-up profile.
pub(super) fn admit(report: &Report, source: NodeKey) -> Result<(), Error> {
    if crate::support::bringup::waive(
        "`#[may_dangle]` is admitted without its drop-check opt-out: drop checking still requires every argument of a marked Drop impl",
    ) {
        return Ok(());
    }
    Err(Declarations::error(
        report,
        source,
        "`#[may_dangle]` drop-check opt-out is not implemented",
    ))
}
