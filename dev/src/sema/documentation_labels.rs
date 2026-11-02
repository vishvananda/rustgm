//! Primitive documentation labels never create primitive types or language authority.
use crate::{
    literal::{self, Value},
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::{FileId, NodeKey},
    },
    sema::model::Declarations,
    support::{hash::HashSet, retained::Growth},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};
/// Sparse state lasts only for the existing owner walk. Inner and outer
/// captures of one actual module share its parsed Item identity.
#[derive(Default)]
pub(super) struct Draft {
    modules: HashSet<(FileId, u32)>,
}
pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
    draft: &mut Draft,
) -> Result<(), Error> {
    facts.reserve_must_use_source(key, attribute.span)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let unit = report.package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("primitive documentation requires an actual module"));
    };
    if !matches!(
        unit.configured.ast.items[index as usize].kind,
        ItemKind::Module { .. }
    ) {
        return Err(error("primitive documentation requires an actual module"));
    }
    if !report.package.requested_internal_attributes()
        && !allowances.allows_at(
            key.file,
            attribute.span,
            &FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error("primitive documentation requires rustc_attrs"));
    }
    let module = (key.file, index);
    if draft.modules.contains(&module) {
        return Err(error("duplicate rustc_doc_primitive attribute"));
    }
    let Input::Equals(range) = attribute.input else {
        return Err(error(
            "primitive documentation requires a name-value string",
        ));
    };
    if attribute.unsafe_wrapper || range.end != range.start + 1 {
        return Err(error(
            "primitive documentation requires an ordinary string value",
        ));
    }
    let token = forest
        .token(range.start)
        .ok_or_else(|| error("primitive documentation requires a literal token"))?;
    let Some(Value::String(_)) =
        literal::interpret(unit.source.text(), token).map_err(|failure| error(failure.message))?
    else {
        return Err(error("primitive documentation requires a string literal"));
    };
    let growth = Growth::map::<(FileId, u32), ()>(draft.modules.len(), draft.modules.capacity(), 1);
    crate::sema::header_attributes::reserve_source_contracts(facts, 1 + growth.work, growth.bytes)?;
    draft
        .modules
        .try_reserve(1)
        .map_err(|_| error("documentation module draft allocation failed"))?;
    draft.modules.insert(module);
    // The configured AST already owns the exact string and its source origin.
    // No compiler/doc generator consumes a copied name, and spelling grants no role.
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/invariants/documentation_labels.rs"]
mod tests;
