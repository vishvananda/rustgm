//! Const-function feature exemptions belong to exact bodies, not crate requests.
mod allowed_targets;
mod exemption_sites;
use super::{Draft, Error, Library, constness::ConstPolicies};
use crate::{
    lex::token::Punctuation,
    resolve::{
        bindings::FeatureName,
        model::{DefId, Resolution},
        package::Package,
        unit::NodeKey,
    },
    support::hash::HashMap,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};

struct Site {
    owner: NodeKey,
    effect: usize,
    attribute: Attribute,
}
struct Entry {
    site: Site,
    features: Vec<Box<str>>,
}
#[derive(Default)]
pub(super) struct Pending {
    entries: HashMap<NodeKey, Vec<Entry>>,
}
struct Row {
    definition: DefId,
    features: Vec<Box<str>>,
    sites: Vec<Site>,
}
#[derive(Default)]
pub(super) struct Grants {
    rows: Vec<Row>,
}

pub(in crate::resolve) fn collect(
    package: &Package,
    draft: &mut Option<Draft<'_>>,
    owner: NodeKey,
    effect: usize,
    attribute: Attribute,
    forest: &Forest,
) -> Result<bool, Error> {
    let unit = package.unit(owner.file);
    if attribute.name(forest, unit.source.text()) != Some("rustc_allow_const_fn_unstable") {
        return Ok(false);
    }
    if unit.include_context.is_some() {
        return Ok(true);
    }
    let error = |message| Error::at(owner.file, attribute.span.start, message);
    let draft = draft.get_or_insert_default();
    let width = attribute.span.end.saturating_sub(attribute.span.start) as usize;
    draft.charge(
        width.saturating_mul(8).saturating_add(1),
        width
            .saturating_mul(64)
            .saturating_add(4 * size_of::<Entry>() + size_of::<Pending>()),
    )?;
    if attribute.unsafe_wrapper {
        return Err(error(
            "const function allowance cannot use an unsafe wrapper",
        ));
    }
    let Input::Delimited(group) = attribute.input else {
        return Err(error("const function allowance requires a feature list"));
    };
    let range = forest
        .children(group)
        .ok_or_else(|| error("missing const function feature list"))?;
    let mut features = Vec::new();
    let mut cursor = range.start;
    while cursor < range.end {
        let name = forest
            .name(unit.source.text(), cursor)
            .filter(|name| {
                !matches!(*name, "true" | "false")
                    || forest
                        .token(cursor)
                        .is_some_and(|token| token.kind == crate::lex::token::Kind::RawIdentifier)
            })
            .ok_or_else(|| error("const function allowance requires feature identifiers"))?;
        let feature = FeatureName::new(name).map_err(error)?;
        features.push(feature.spelling().into());
        cursor += 1;
        if cursor < range.end {
            if !forest.punctuation(cursor, Punctuation::Comma) {
                return Err(error(
                    "const function allowance requires comma-separated feature identifiers",
                ));
            }
            cursor += 1;
        }
    }
    let mut item = owner;
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(item.node).kind {
        item.node = value;
    }
    if !matches!(unit.configured.ast.node(item.node).kind, Kind::Item(index)
        if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Function { body: Some(_), .. }))
    {
        return Err(error(
            "const function allowance requires an actual const function body",
        ));
    }
    draft
        .fn_allowances
        .get_or_insert_default()
        .entries
        .entry(item)
        .or_default()
        .push(Entry {
            site: Site {
                owner,
                effect,
                attribute,
            },
            features,
        });
    Ok(true)
}

pub(in crate::resolve) fn prepare_library(
    package: &Package,
    draft: &mut Option<Draft<'_>>,
) -> Result<Option<Box<Library>>, Error> {
    if package.library.is_none()
        && let Some(draft) = draft.as_mut().filter(|draft| draft.fn_allowances.is_some())
    {
        draft.charge(1, size_of::<Library>())?;
        return Ok(Some(Box::new(Library::default())));
    }
    Ok(None)
}

pub(super) fn finish(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft<'_>,
    consts: &mut Option<Box<ConstPolicies>>,
) -> Result<(), Error> {
    let Some(mut pending) = draft.fn_allowances.take() else {
        return Ok(());
    };
    if !package
        .library
        .as_deref()
        .is_some_and(|library| library.staged)
    {
        draft.finish_comptime()?;
    }
    let targets = allowed_targets::collect(package, facts, draft)?;
    let mut grants = Grants::default();
    for (index, definition) in facts.definitions.iter().enumerate() {
        draft.charge(1, 0)?;
        if definition.kind != crate::resolve::model::DefKind::Function {
            continue;
        }
        let Some(key) = definition.node else {
            continue;
        };
        let Some(entries) = pending.entries.remove(&key) else {
            continue;
        };
        let id = DefId(
            u32::try_from(index)
                .map_err(|_| Error::unlocated("const allowance declaration overflow"))?,
        );
        if !targets.contains(&id) {
            return Err(Error::at(
                key.file,
                entries[0].site.attribute.span.start,
                "const function allowance requires an actual const function",
            ));
        }
        let mut features = Vec::new();
        let mut sites = Vec::new();
        for entry in entries {
            features.extend(entry.features);
            sites.push(entry.site);
        }
        let bytes = features.iter().map(|name| name.len()).sum::<usize>();
        let depth =
            usize::try_from(usize::BITS - features.len().leading_zeros()).unwrap_or(usize::MAX);
        draft.charge(
            bytes
                .saturating_add(features.len())
                .saturating_mul(depth + 1),
            4 * size_of::<Row>(),
        )?;
        features.sort_unstable();
        features.dedup();
        grants.rows.push(Row {
            definition: id,
            features,
            sites,
        });
    }
    if !pending.entries.is_empty() {
        return Err(Error::unlocated(
            "const allowance has no actual function declaration",
        ));
    }
    if consts.is_none() {
        draft.charge(
            facts.definitions.len(),
            facts
                .definitions
                .len()
                .saturating_mul(size_of::<Option<super::PolicyId>>())
                .saturating_add(size_of::<ConstPolicies>()),
        )?;
        *consts = Some(Box::new(ConstPolicies {
            declarations: vec![None; facts.definitions.len()],
            implicit: Vec::new(),
            grants: None,
        }));
    }
    consts
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("missing const allowance inventory"))?
        .grants = Some(Box::new(grants));
    Ok(())
}

pub(crate) use exemption_sites::{validate, validate_plain};
impl Grants {
    pub(super) fn storage(&self) -> usize {
        size_of::<Self>()
            + self.rows.capacity() * size_of::<Row>()
            + self
                .rows
                .iter()
                .map(|row| {
                    row.features.capacity() * size_of::<Box<str>>()
                        + row.sites.capacity() * size_of::<Site>()
                        + row.features.iter().map(|name| name.len()).sum::<usize>()
                })
                .sum::<usize>()
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/fn_allowance_sites.rs"]
mod tests;
