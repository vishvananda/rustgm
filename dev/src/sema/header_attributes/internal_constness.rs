//! Real header words describe API/defining policy, never const-call permission.
use super::{Declarations, DefId, Error, NodeKey, Report};
use crate::syntax::{
    ast::Kind,
    attribute::{Attribute, Input},
    item::ItemKind,
};
#[cfg(test)]
#[path = "../../../tests/invariants/indirect_const_api.rs"]
mod tests;

pub(super) fn validate_words(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    declaration: NodeKey,
    words: (bool, bool),
) -> Result<(), Error> {
    if words == (false, false) {
        return Ok(());
    }
    let error = |message| {
        Error::at(
            declaration.file,
            report
                .package
                .unit(declaration.file)
                .configured
                .ast
                .node(declaration.node)
                .span
                .start,
            message,
        )
    };
    if words.1 {
        if facts
            .associated
            .owner(definition)
            .and_then(|id| facts.associated.get(id))
            .is_some_and(|owner| {
                !matches!(
                    owner.kind,
                    crate::sema::associated::OwnerKind::Impl {
                        trait_type: None,
                        ..
                    }
                )
            })
        {
            return Err(error(
                "rustc_const_stable_indirect requires a free or inherent function",
            ));
        }
        if report
            .package
            .library_policy()
            .and_then(crate::resolve::stability::Library::inventory)
            .and_then(|inventory| inventory.const_declaration(definition))
            .is_some_and(|policy| {
                matches!(policy.level, crate::resolve::stability::Level::Stable(_))
            })
        {
            return Err(error(
                "const-stable functions cannot also be const-stable-indirect",
            ));
        }
    }
    if words.0 {
        let role = report
            .resolution
            .source_bindings
            .as_deref()
            .is_some_and(|bindings| {
                [
                    crate::support::bindings::LangItem::PanicFmt,
                    crate::support::bindings::LangItem::PanicDisplay,
                ]
                .into_iter()
                .any(|role| {
                    bindings.language(role).is_some_and(|actual| {
                        actual.definition == definition && actual.origin == declaration
                    })
                })
            });
        if !role || !words.1 {
            return Err(error(
                "source do-not-const-check requires its actual supported panic role and indirect marker",
            ));
        }
    }
    Ok(())
}

pub(in crate::sema) fn consume_const_panic(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error("const panic policy requires a word attribute"));
    }
    if report.package.source_core().is_none() || !report.package.requested_internal_attributes() {
        return Err(error(
            "source const panic policy requires its actual rustc_attrs provider",
        ));
    }
    let unit = report.package.unit(declaration.file);
    if !matches!(unit.configured.ast.node(declaration.node).kind,
        Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Function { .. }))
    {
        return Err(error("const panic policy requires an actual function"));
    }
    Ok(())
}

/// This driver's fixed checked profile already emits every arithmetic check.
/// The word grants no caller context, const eligibility or imported permission.
pub(in crate::sema) fn consume_inherited_overflow(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    forest: &crate::syntax::token_tree::Forest,
    allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error("inherited overflow policy requires a word attribute"));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("overflow policy lost its actual name"))?
        .span;
    if report.package.source_core().is_none()
        || (!report.package.requested_internal_attributes()
            && !allowances.allows_at(
                declaration.file,
                name,
                &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
            )?)
    {
        return Err(error(
            "inherited overflow policy requires its rustc_attrs source profile",
        ));
    }
    let ast = &report.package.unit(declaration.file).configured.ast;
    let valid = match ast.node(declaration.node).kind {
        Kind::Closure { .. } => true,
        Kind::Item(index) => matches!(
            ast.items[index as usize].kind,
            ItemKind::Function { body: Some(_), .. }
        ),
        _ => false,
    };
    if !valid {
        return Err(error(
            "inherited overflow policy requires a function body or closure",
        ));
    }
    Ok(())
}

impl Declarations {
    /// Each query authenticates the completed physical header before reading
    /// the retained words. No marker result is a role, instance or call proof.
    pub(in crate::sema) fn const_panic_markers(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<(bool, bool), Error> {
        let key = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .filter(|definition| definition.kind == crate::resolve::model::DefKind::Function)
            .and_then(|definition| definition.node)
            .ok_or_else(|| Error::unlocated("const panic policy requires a local function"))?;
        let mut markers = (false, false);
        if !super::implicit_receivers::visit_words(self, report, definition, |name| match name {
            Some("rustc_do_not_const_check") => markers.0 = true,
            Some("rustc_const_stable_indirect") => markers.1 = true,
            _ => {}
        })? {
            return Err(Error::at(
                key.file,
                0,
                "const panic policy requires completed source headers",
            ));
        }
        Ok(markers)
    }
}
