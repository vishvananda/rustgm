//! `#[thread_local]` gives a static one instance per thread, as std's
//! `thread_local!` expands it (`library/std/src/sys/thread_local/`). It is an
//! outer word attribute on a static or a foreign static, gated by
//! `thread_local` or by a macro's exact allowance of it. Such a static needs
//! no `Sync`, is never read or addressed during constant evaluation, is never
//! borrowed beyond the function that borrows it, and lowers to a backend
//! thread-local global.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::hash::HashSet,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::Forest,
    },
};

/// Validate one `#[thread_local]` and remember its static for [`bind`].
/// # Errors
/// Rejects another form, an ungated use and an owner that is not a static.
pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
    statics: &mut Vec<NodeKey>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper {
        return Err(error("`thread_local` is not an unsafe attribute"));
    }
    if attribute.style != Style::Outer || attribute.input != Input::Empty {
        return Err(error(
            "malformed `thread_local` attribute input: it is an outer word attribute",
        ));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("thread_local attribute has no actual name token"))?
        .span;
    if !report.package.requested_thread_local().map_err(error)?
        && !allowances.allows_at(
            declaration.file,
            name,
            &FeatureName::new("thread_local").map_err(error)?,
        )?
    {
        return Err(error(
            "`#[thread_local]` is an experimental feature: it requires thread_local",
        ));
    }
    let ast = &report.package.unit(declaration.file).configured.ast;
    if !matches!(ast.node(declaration.node).kind, Kind::Item(index)
        if matches!(ast.items[index as usize].kind, ItemKind::Static { .. }))
    {
        return Err(error("`#[thread_local]` applies only to statics"));
    }
    statics.push(declaration);
    Ok(())
}

/// Record the definition of every static [`consume`] admitted.
/// # Errors
/// Rejects an exhausted definition identity.
pub(super) fn bind(
    report: &Report,
    facts: &mut Declarations,
    statics: &[NodeKey],
) -> Result<(), Error> {
    if statics.is_empty() {
        return Ok(());
    }
    let statics: HashSet<NodeKey> = statics.iter().copied().collect();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        let Some(key) = definition.node else {
            continue;
        };
        if definition.kind == DefKind::Static && statics.contains(&key) {
            let id = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::at(key.file, 0, "too many static declarations"))?,
            );
            facts.static_permissions_mut().thread_local.insert(id);
        }
    }
    Ok(())
}
