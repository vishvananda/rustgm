//! Inline attributes are checked advisory data, independent of execution permission.
use crate::support::hash::HashMap;
use crate::{
    resolve::{
        expansion::Report,
        model::DefId,
        package::Error,
        unit::{FileId, NodeKey},
    },
    sema::{model::Declarations, must_use::WarningCode},
    support::{crates::generics::source::InlineApplicability, source::OriginalImage},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::{Delimiter, Forest, TreeKind},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hint {
    Hint,
    Always,
    Never,
}
#[derive(Clone, Copy)]
pub struct View<'a> {
    pub origin: super::header_attributes::ContractOrigin<'a>,
    pub hint: Hint,
    pub applicability: InlineApplicability,
}
struct Entry {
    source: super::header_attributes::SourceContractSite,
    hint: Hint,
    applicability: InlineApplicability,
    effect: usize,
    image: std::sync::Arc<OriginalImage>,
}
#[derive(Default)]
pub(in crate::sema) struct Contracts {
    entries: HashMap<NodeKey, Entry>,
    bytes: usize,
}
impl Declarations {
    /// Read a validated advisory hint without authorizing a body or invocation.
    /// # Errors
    /// Rejects an unknown source item or changed configured owner.
    pub fn inline_hint(
        &self,
        report: &Report,
        declaration: NodeKey,
    ) -> Result<Option<View<'_>>, Error> {
        let Some(entry) = self
            .must_use_contracts
            .as_deref()
            .and_then(|contracts| contracts.inline.as_deref())
            .and_then(|contracts| contracts.entries.get(&declaration))
        else {
            return Ok(None);
        };
        let view = entry.source;
        let unit = report.package.unit(view.owner.file);
        let owner = unit
            .owner_indices
            .get(view.owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1));
        if !std::sync::Arc::ptr_eq(&entry.image, unit.source.original_image())
            || owner.is_none_or(|owner| {
                !owner.enabled
                    || owner.node != view.owner.node
                    || !owner.attributes.contains(&entry.effect)
                    || !unit
                        .retained_attributes(entry.effect..entry.effect + 1)
                        .any(|(attribute, forest)| {
                            attribute.span == view.span
                                && attribute.name(forest, unit.source.text()) == Some("inline")
                        })
            })
        {
            return Err(Error::at(
                view.owner.file,
                view.span.start,
                "inline hint actual owner changed",
            ));
        }
        Ok(Some(View {
            origin: super::header_attributes::ContractOrigin::source_site(
                view.owner,
                view.declaration,
                view.span,
            ),
            hint: entry.hint,
            applicability: entry.applicability,
        }))
    }
    /// # Errors
    /// Rejects unknown identities and untransported imported hints.
    pub fn inline_hint_for_definition(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<Option<View<'_>>, Error> {
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| Error::at(FileId(0), 0, "unknown inline declaration"))?;
        if actual.node.is_none() {
            use super::header_attributes::ContractKnowledge;
            if super::header_attributes::local_crate_owner(report, definition)?
                || super::header_attributes::bootstrap_owner(report, definition)?
            {
                return Ok(None);
            }
            if super::header_attributes::has_imported(self, definition) {
                return match self.header_attribute_contracts(report, definition)?.inline {
                    ContractKnowledge::KnownAbsent => Ok(None),
                    ContractKnowledge::KnownPresent(view) => Ok(Some(view)),
                    ContractKnowledge::Unknown => Err(Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "imported inline hint transport is not implemented",
                    )),
                };
            }
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "imported inline hint transport is not implemented",
            ));
        }
        if self
            .must_use_contracts
            .as_deref()
            .is_some_and(|contracts| contracts.unknown.contains(&definition))
        {
            return Err(Error::at(
                FileId(0),
                0,
                "imported inline hint transport is not implemented",
            ));
        }
        actual
            .node
            .map_or(Ok(None), |node| self.inline_hint(report, node))
    }
}
/// The cold word remains independent advisory data; no optimization or effect
/// promise is inferred. Wider Rust targets require their own retained producer.
pub(in crate::sema) fn consume_cold(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error("cold requires a plain word attribute"));
    }
    let ast = &report.package.unit(declaration.file).configured.ast;
    if !matches!(ast.node(declaration.node).kind,Kind::Item(index)
        if matches!(ast.items[index as usize].kind,ItemKind::Function{body:Some(_),..}))
    {
        return Err(error(
            "cold function-body profile is not implemented for this wider Rust target",
        ));
    }
    Ok(())
}
#[cfg(test)]
#[path = "../../tests/invariants/cold_annotations.rs"]
mod cold_tests;

pub(super) fn collect(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file).map_err(|_| Error::at(FileId(0), 0, "too many inline files"))?,
        );
        for owner in &unit.configured.owners {
            if !owner.enabled {
                continue;
            }
            let owner_key = NodeKey {
                file,
                node: owner.node,
            };
            let mut node = owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            let key = NodeKey { file, node };
            for (effect, attribute, forest) in
                unit.retained_indexed_attributes(owner.attributes.clone())
            {
                if attribute.name(forest, unit.source.text()) != Some("inline") {
                    continue;
                }
                let hint = parse(unit.source.text(), forest, attribute)
                    .map_err(|message| Error::at(file, attribute.span.start, message))?;
                let applicability = target(unit, node)
                    .map_err(|message| Error::at(file, attribute.span.start, message))?;
                if applicability != InlineApplicability::FunctionBody {
                    facts.must_use_diagnostic(
                        report,
                        owner_key,
                        attribute.span.start,
                        0,
                        WarningCode::UnusedAttributes,
                        "inline attribute on a function without a body has no effect".into(),
                    )?;
                }
                let repeated = facts
                    .must_use_contracts
                    .as_deref()
                    .and_then(|contracts| contracts.inline.as_deref())
                    .is_some_and(|contracts| contracts.entries.contains_key(&key));
                if repeated {
                    facts.must_use_diagnostic(
                        report,
                        owner_key,
                        attribute.span.start,
                        0,
                        WarningCode::UnusedAttributes,
                        "duplicate inline attribute".into(),
                    )?;
                    continue;
                }
                let contracts = facts
                    .must_use_contracts
                    .get_or_insert_default()
                    .inline
                    .get_or_insert_default();
                contracts.bytes = contracts
                    .bytes
                    .saturating_add(crate::support::retained::new_map::<NodeKey, Entry>(1));
                if contracts.entries.len() >= 1_000_000 || contracts.bytes > 64 * 1024 * 1024 {
                    return Err(Error::at(
                        file,
                        attribute.span.start,
                        "inline hint storage budget exceeded",
                    ));
                }
                contracts.entries.insert(
                    key,
                    Entry {
                        source: super::header_attributes::SourceContractSite {
                            owner: owner_key,
                            declaration: key,
                            span: attribute.span,
                        },
                        hint,
                        applicability,
                        effect,
                        image: std::sync::Arc::clone(unit.source.original_image()),
                    },
                );
            }
        }
    }
    Ok(())
}
fn target(
    unit: &crate::resolve::unit::Unit,
    node: crate::syntax::ast::NodeId,
) -> Result<InlineApplicability, &'static str> {
    match unit.configured.ast.node(node).kind {
        Kind::Item(index) => match unit.configured.ast.items[index as usize].kind {
            ItemKind::Function { body, .. } => Ok(if body.is_some() {
                InlineApplicability::FunctionBody
            } else {
                InlineApplicability::DiagnosedNoBody
            }),
            _ => Err("inline attribute requires a function or closure"),
        },
        Kind::Closure { .. } => Ok(InlineApplicability::FunctionBody),
        _ => Err("inline attribute requires a function or closure"),
    }
}

fn parse(source: &str, forest: &Forest, attribute: Attribute) -> Result<Hint, &'static str> {
    if attribute.unsafe_wrapper {
        return Err("inline attribute cannot use an unsafe wrapper");
    }
    match attribute.input {
        Input::Empty => Ok(Hint::Hint),
        Input::Delimited(group) => {
            if !forest.tree(group).is_some_and(|tree| {
                matches!(
                    tree.kind,
                    TreeKind::Group {
                        delimiter: Delimiter::Parenthesis,
                        ..
                    }
                )
            }) {
                return Err("inline attribute requires parenthesized arguments");
            }
            let range = forest
                .children(group)
                .ok_or("inline attribute arguments are missing")?;
            let end = range.end
                - u32::from(
                    range.end > range.start
                        && forest.punctuation(range.end - 1, crate::lex::token::Punctuation::Comma),
                );
            if end != range.start + 1 {
                return Err("inline attribute requires exactly always or never");
            }
            match forest.name(source, range.start) {
                Some("always") => Ok(Hint::Always),
                Some("never") => Ok(Hint::Never),
                _ => Err("inline attribute requires exactly always or never"),
            }
        }
        // The independent default-deny ill_formed_attribute_input contract rejects
        // this form. Its explicitly allowed warning profile remains unsupported.
        _ => Err("inline name-value attribute profile is not implemented"),
    }
}
