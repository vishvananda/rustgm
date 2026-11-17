//! Removed declarations retain crate-root ownership separately from API policy IDs.
use super::{Draft, Inventory, Since};
use crate::{
    lex::token::Punctuation,
    resolve::{
        package::{Error, Package},
        unit::NodeKey,
    },
    support::{
        crates::{RemovedFeature, RemovedFeatures, RemovedVersion},
        retained::{self, Growth},
    },
    syntax::{
        attribute::{Attribute, Input, Style},
        token_tree::{Forest, Span},
    },
};
use std::borrow::Cow;

pub(super) struct Declaration {
    pub origin: NodeKey,
    pub span: Span,
    pub data: RemovedFeature,
}
#[derive(Default)]
pub(super) struct Declarations {
    pub entries: Vec<Declaration>,
}
impl Declarations {
    pub(super) fn allocation_bytes(&self) -> usize {
        self.entries.iter().fold(
            size_of::<Self>().saturating_add(retained::vector(&self.entries)),
            |bytes, row| bytes.saturating_add(row.data.allocation_bytes()),
        )
    }
}
pub(in crate::resolve) fn collect(
    package: &Package,
    draft: &mut Option<Draft<'_>>,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
) -> Result<bool, Error> {
    let unit = package.unit(key.file);
    if attribute.name(forest, unit.source.text()) != Some("unstable_removed") {
        return Ok(false);
    }
    // Templates are duplicate provenance. install_include copies their configured
    // owners into the caller Unit, whose actual owner is checked below; a copied
    // owner that is not the real crate root remains an explicit placement error.
    if unit.include_context.is_some() {
        return Ok(true);
    }
    let error = |message| Error::at(key.file, attribute.span.start, message);
    if package.source_core().is_none() {
        return Err(error(
            "removed library declarations require the actual source-core profile",
        ));
    }
    let host = package
        .modules
        .first()
        .is_some_and(|root| root.file == key.file && root.root == key.node);
    if !host && !package.guest_root(key) {
        return Err(error(
            "removed library declarations require the actual crate root",
        ));
    }
    if attribute.style != Style::Inner || attribute.unsafe_wrapper {
        return Err(error(
            "removed library declarations require a safe inner attribute",
        ));
    }
    if !package.staged_crate(key.file) {
        return Err(error("removed library declarations require own staged_api"));
    }
    let draft = draft
        .as_mut()
        .ok_or_else(|| error("removed library declarations lack their source inventory payer"))?;
    let data = parse(package, key, attribute, forest, draft)?;
    if draft.retirements.is_none() {
        draft.charge(1, size_of::<Declarations>())?;
        draft.retirements = Some(Box::default());
    }
    let column = draft
        .retirements
        .as_ref()
        .ok_or_else(|| error("removed library declaration owner is missing"))?;
    let growth = Growth::vector::<Declaration>(column.entries.len(), column.entries.capacity(), 1);
    draft.charge(1usize.saturating_add(growth.work), growth.bytes)?;
    let column = draft
        .retirements
        .as_mut()
        .ok_or_else(|| error("removed library declaration owner disappeared"))?;
    column
        .entries
        .try_reserve(1)
        .map_err(|_| error("removed library declaration allocation failed"))?;
    column.entries.push(Declaration {
        origin: key,
        span: attribute.span,
        data,
    });
    Ok(true)
}
fn parse(
    package: &Package,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    draft: &mut Draft<'_>,
) -> Result<RemovedFeature, Error> {
    let unit = package.unit(key.file);
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let Input::Delimited(group) = attribute.input else {
        return Err(error(
            "removed library declaration requires named string arguments",
        ));
    };
    let range = forest
        .children(group)
        .ok_or_else(|| error("missing removed library arguments"))?;
    let mut fields: [Option<Cow<'_, str>>; 4] = std::array::from_fn(|_| None);
    let mut cursor = range.start;
    while cursor < range.end {
        let name = forest
            .name(unit.source.text(), cursor)
            .ok_or_else(|| error("invalid removed library argument"))?;
        draft.charge(name.len().saturating_add(1), 0)?;
        let field = match name {
            "feature" => 0,
            "reason" => 1,
            "link" => 2,
            "since" => 3,
            _ => return Err(error("unsupported removed library argument")),
        };
        if fields[field].is_some() {
            return Err(error("duplicate removed library argument"));
        }
        if !forest.punctuation(cursor + 1, Punctuation::Equal) || cursor + 2 >= range.end {
            return Err(error("removed library argument requires a string literal"));
        }
        let token = forest
            .token(cursor + 2)
            .ok_or_else(|| error("missing removed library literal"))?;
        fields[field] = Some(super::attributes::string(
            unit,
            key,
            attribute.span.start,
            token,
            draft,
        )?);
        cursor += 3;
        if cursor < range.end {
            if !forest.punctuation(cursor, Punctuation::Comma) {
                return Err(error("removed library arguments require commas"));
            }
            cursor += 1;
        }
    }
    let [feature, reason, link, since] = fields;
    let feature = feature.ok_or_else(|| error("removed library declaration requires feature"))?;
    let reason = reason.ok_or_else(|| error("removed library declaration requires reason"))?;
    let link = link.ok_or_else(|| error("removed library declaration requires link"))?;
    let since = since.ok_or_else(|| error("removed library declaration requires since"))?;
    let Some(Since::Version {
        major,
        minor,
        patch,
    }) = super::attributes::version(&since)
    else {
        return Err(error(
            "removed library declaration requires a numeric version",
        ));
    };
    // Literal decoding has prepaid its owned output; borrowed values copy only
    // their logical length. The separate data never enters the active feature index.
    draft.charge(
        feature
            .len()
            .saturating_add(reason.len())
            .saturating_add(link.len()),
        feature
            .len()
            .saturating_add(reason.len())
            .saturating_add(link.len()),
    )?;
    Ok(RemovedFeature {
        feature: feature.into_owned(),
        reason: reason.into_owned(),
        link: link.into_owned(),
        since: RemovedVersion {
            major,
            minor,
            patch,
        },
    })
}
impl Inventory {
    pub(super) fn removed_bytes(&self) -> usize {
        self.retirements
            .as_deref()
            .map_or(0, Declarations::allocation_bytes)
    }
}
impl Package {
    /// Copy only this checked host's crate declaration data; guests retain their own rows.
    /// # Errors
    /// Rejects unfinished inventories, exhausted source payment and allocation failures.
    pub fn removed_features(&self) -> Result<Option<RemovedFeatures>, Error> {
        let Some(source) = self.source_core() else {
            return Ok(None);
        };
        let Some(library) = self.library_policy() else {
            return Ok(None);
        };
        let inventory = library.inventory().ok_or_else(|| {
            Error::unlocated("removed library declaration inventory is incomplete")
        })?;
        let Some(column) = inventory.retirements.as_deref() else {
            return Ok(None);
        };
        let root = self
            .modules
            .first()
            .ok_or_else(|| Error::unlocated("removed library publication has no root"))?;
        let key = NodeKey {
            file: root.file,
            node: root.root,
        };
        source
            .macros
            .inventory_order_charge(column.entries.len().saturating_mul(3), 0, 0)
            .map_err(Error::unlocated)?;
        let count = column
            .entries
            .iter()
            .filter(|row| row.origin == key)
            .count();
        if count == 0 {
            return Ok(None);
        }
        let growth = Growth::vector::<RemovedFeature>(0, 0, count);
        let strings = column
            .entries
            .iter()
            .filter(|row| row.origin == key)
            .fold(0usize, |bytes, row| {
                bytes.saturating_add(row.data.string_bytes())
            });
        let transient = growth
            .bytes
            .saturating_add(strings)
            .saturating_add(size_of::<RemovedFeatures>());
        source
            .macros
            .inventory_order_charge(
                column.entries.len().saturating_add(strings),
                transient,
                transient,
            )
            .map_err(Error::unlocated)?;
        let mut rows = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| Error::unlocated("removed library publication allocation failed"))?;
        for declaration in &column.entries {
            if declaration.origin != key {
                continue;
            }
            if declaration.span.start > declaration.span.end
                || declaration.span.end as usize > self.unit(key.file).source.text().len()
            {
                return Err(Error::at(
                    key.file,
                    declaration.span.start,
                    "removed library declaration span is outside its root source",
                ));
            }
            rows.push(declaration.data.clone());
        }
        Ok(Some(
            RemovedFeatures::new(source.current.clone(), rows).map_err(Error::unlocated)?,
        ))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/retired_feature_records.rs"]
mod tests;
