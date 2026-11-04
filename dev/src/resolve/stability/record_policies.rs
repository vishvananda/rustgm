//! Field availability follows one actual configured aggregate/variant owner.
use super::{Draft, Inventory, Policy, PolicyId, PolicyKind, owners::Owners};
use crate::{
    resolve::{
        model::{DefKind, Resolution},
        package::{Error, Package},
        unit::NodeKey,
    },
    support::{
        hash::{HashMap, HashSet},
        retained,
    },
    syntax::{ast::Kind, item::ItemKind},
};

#[derive(Clone, Copy)]
struct Row {
    owner: NodeKey,
    regular: Option<PolicyId>,
}
#[derive(Default)]
pub(super) struct Fields {
    rows: HashMap<NodeKey, Row>,
}
impl Fields {
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>().saturating_add(retained::map(&self.rows))
    }
    fn insert(&mut self, source: NodeKey, row: Row, draft: &mut Draft<'_>) -> Result<(), Error> {
        draft.charge(1, 0)?;
        if let Some(previous) = self.rows.get(&source) {
            if previous.owner != row.owner || previous.regular != row.regular {
                return Err(Error::unlocated(
                    "field policy changed its actual aggregate owner",
                ));
            }
            return Ok(());
        }
        let growth =
            retained::Growth::map::<NodeKey, Row>(self.rows.len(), self.rows.capacity(), 1);
        draft.charge(growth.work, growth.bytes)?;
        self.rows
            .try_reserve(1)
            .map_err(|_| Error::unlocated("field policy allocation failed"))?;
        self.rows.insert(source, row);
        Ok(())
    }
}

pub(super) fn collect(
    package: &Package,
    facts: &Resolution,
    owners: &Owners,
    draft: &mut Draft<'_>,
    used: &mut HashSet<(NodeKey, bool)>,
) -> Result<Option<Box<Fields>>, Error> {
    let mut result: Option<Box<Fields>> = None;
    for (index, definition) in facts.definitions.iter().enumerate() {
        draft.charge(1, 0)?;
        if !matches!(definition.kind, DefKind::Struct | DefKind::Variant) {
            continue;
        }
        let Some(owner) = definition.node else {
            continue;
        };
        let unit = package.unit(owner.file);
        let fields = match unit.configured.ast.node(owner.node).kind {
            Kind::Item(item) => match unit.configured.ast.items[item as usize].kind {
                ItemKind::Struct { fields, .. } | ItemKind::Union { fields } => fields,
                _ => continue,
            },
            Kind::Variant(variant) => unit.configured.ast.variants[variant as usize].fields,
            _ => continue,
        };
        for &node in unit.configured.ast.list(fields) {
            draft.charge(1, 0)?;
            if !unit.configured.active[node.get() as usize] {
                continue;
            }
            let mut key = NodeKey { node, ..owner };
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(key.node).kind {
                draft.charge(1, 0)?;
                key.node = value;
            }
            if !matches!(
                unit.configured.ast.node(key.node).kind,
                Kind::FieldDefinition { .. }
            ) {
                return Err(super::owners::error(
                    package,
                    key,
                    "field policy requires an actual field definition",
                ));
            }
            if draft.attribute(key, PolicyKind::Const).is_some() {
                return Err(super::owners::error(
                    package,
                    key,
                    "const library stability is not supported on fields",
                ));
            }
            let direct = draft.attribute(key, PolicyKind::Regular);
            if let Some(id) = direct {
                if draft
                    .policies
                    .get(id.get() - 1)
                    .is_none_or(|policy| policy.origin != key)
                {
                    return Err(super::owners::error(
                        package,
                        key,
                        "field policy changed its source owner",
                    ));
                }
                used.insert((key, false));
            }
            if result.is_none() {
                draft.charge(1, size_of::<Fields>())?;
                result = Some(Box::default());
            }
            let rows = result
                .as_deref_mut()
                .ok_or_else(|| Error::unlocated("field policy owner disappeared"))?;
            rows.insert(
                key,
                Row {
                    owner,
                    regular: direct.or(owners.declarations[index]),
                },
                draft,
            )?;
        }
    }
    Ok(result)
}

impl Inventory {
    /// Completed local field data; another parent or unknown field is not absence.
    pub(crate) fn field_declaration(
        &self,
        source: NodeKey,
        owner: NodeKey,
    ) -> Result<Option<&Policy>, &'static str> {
        let row = self
            .fields
            .as_deref()
            .and_then(|fields| fields.rows.get(&source))
            .ok_or("field policy is outside its completed source inventory")?;
        if row.owner != owner {
            return Err("field policy belongs to another actual aggregate");
        }
        row.regular
            .map(|id| {
                self.policies
                    .get(id.get() - 1)
                    .ok_or("field policy index is invalid")
            })
            .transpose()
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/record_policies.rs"]
mod tests;
