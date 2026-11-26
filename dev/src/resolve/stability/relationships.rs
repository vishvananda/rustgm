//! Exact policy links share cold storage; a former name never enables an API.
use super::{Draft, Inventory, Level, PolicyId};
use crate::{
    resolve::{
        package::{Error, Package},
        unit::FileId,
    },
    support::hash::{HashMap, HashSet},
};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    ImpliedBy,
    OldName,
}
pub(super) struct Relationship {
    pub policy: PolicyId,
    pub name: Box<str>,
    pub kind: Kind,
    pub canonical: bool,
}
#[derive(Default)]
pub(super) struct Pending {
    pub implied_by: Option<Box<str>>,
    pub old_name: Option<Box<str>>,
}
impl Pending {
    pub(super) fn entries(self) -> [(Kind, Option<Box<str>>); 2] {
        [
            (Kind::ImpliedBy, self.implied_by),
            (Kind::OldName, self.old_name),
        ]
    }
    pub(super) fn len(&self) -> usize {
        usize::from(self.implied_by.is_some()) + usize::from(self.old_name.is_some())
    }
}
impl Draft<'_> {
    /// Complete the declaration projection and all root requests together, before owners publish.
    pub(super) fn check_feature_requests(
        &mut self,
        package: &Package,
        imported: Option<&super::availability_imports::Imports>,
        requests: &[Box<str>],
    ) -> Result<(), Error> {
        self.charge(
            requests
                .iter()
                .map(|name| name.len().saturating_add(1))
                .sum::<usize>()
                .saturating_add(self.relationship_work()),
            0,
        )?;
        let has_renames = self
            .relationships
            .iter()
            .any(|edge| edge.kind == Kind::OldName && edge.canonical);
        if has_renames {
            self.charge(
                requests
                    .iter()
                    .map(|name| name.len().saturating_add(1))
                    .sum(),
                crate::support::retained::new_map::<&str, ()>(requests.len()),
            )?;
        }
        if let Some(id) = self.features.conflict {
            return Err(super::owners::error(
                package,
                self.policies[id.get() - 1].origin,
                "conflicting library feature definitions",
            ));
        }
        let features = &self.features.definitions;
        let requested_names: HashSet<_> = if has_renames {
            requests.iter().map(AsRef::as_ref).collect()
        } else {
            HashSet::default()
        };
        self.validate_relationships(package, features, imported, &requested_names)?;
        // These are also language gates: using them needs no local unstable API.
        // This does not add those names to `features` or validate an implied_by target.
        let aggregate_language = package
            .requested_aggregate_const_parameters()
            .map_err(|message| Error::at(FileId(0), 0, message))?;
        let async_destruction = package
            .requested_async_drop()
            .map_err(|message| Error::at(FileId(0), 0, message))?;
        for name in requests {
            let known = features.contains_key(name.as_ref())
                || (aggregate_language
                    && matches!(name.as_ref(), "min_adt_const_params" | "adt_const_params"))
                || (async_destruction && name.as_ref() == "async_drop")
                || (package.requested_named_opaque() && name.as_ref() == "type_alias_impl_trait");
            if !known {
                if let Some(imports) = imported {
                    if let Some(message) = imports.renamed_message(name)? {
                        return Err(Error::at(FileId(0), 0, message));
                    }
                    if imports.has_feature(name)? {
                        continue;
                    }
                }
                if let Some(message) = imported
                    .map(|imports| imports.removed_message(name))
                    .transpose()?
                    .flatten()
                {
                    return Err(Error::at(FileId(0), 0, message));
                }
                return Err(Error::at(
                    FileId(0),
                    0,
                    format!(
                        "unsupported active source-core feature or undefined library feature request `{name}`"
                    ),
                ));
            }
        }
        Ok(())
    }
    pub(super) fn prepay_relationships(&mut self, additional: usize) -> Result<(), Error> {
        let growth = crate::support::retained::Growth::vector::<Relationship>(
            self.relationships.len(),
            self.relationships.capacity(),
            additional,
        );
        self.charge(growth.work, growth.bytes)?;
        self.relationships
            .try_reserve(additional)
            .map_err(|_| Error::unlocated("library relationship allocation failed"))
    }
    pub(super) fn relationship_work(&self) -> usize {
        self.relationships
            .iter()
            .map(|edge| {
                edge.name
                    .len()
                    .saturating_mul(2)
                    .saturating_add(self.policies[edge.policy.get() - 1].feature.len())
                    .saturating_add(4)
            })
            .sum()
    }
    pub(super) fn validate_relationships(
        &self,
        package: &Package,
        features: &HashMap<Rc<str>, PolicyId>,
        imported: Option<&super::availability_imports::Imports>,
        requests: &HashSet<&str>,
    ) -> Result<(), Error> {
        for edge in &self.relationships {
            let policy = &self.policies[edge.policy.get() - 1];
            match edge.kind {
                Kind::ImpliedBy => {
                    if !features.contains_key(edge.name.as_ref())
                        && !imported
                            .map(|imports| imports.has_feature(&edge.name))
                            .transpose()?
                            .unwrap_or(false)
                    {
                        return Err(super::owners::error(
                            package,
                            policy.origin,
                            "implied_by names an undefined library feature",
                        ));
                    }
                }
                Kind::OldName => {
                    // Rust checks sorted canonical local features before dependencies.
                    // A canonical request consumed earlier is not reinterpreted as old.
                    if edge.canonical
                        && requests.contains(edge.name.as_ref())
                        && (!features.contains_key(edge.name.as_ref())
                            || policy.feature.as_ref() < edge.name.as_ref())
                    {
                        return Err(super::owners::error(
                            package,
                            policy.origin,
                            "requested library feature has been renamed",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
// Since::Current follows this compiler's pinned Rust 1.98.0 language profile,
// independently of the host used to build RustGM; retain its original wire spelling.
pub(super) fn release(level: &Level) -> Option<(u16, u16, u16)> {
    let since = match level {
        Level::Stable(since) | Level::StableWithCompatibility { since, .. } => since,
        Level::Unstable { .. } => return None,
    };
    Some(since.release())
}
impl Inventory {
    pub(crate) fn has_relationships(&self) -> bool {
        !self.relationships.is_empty()
    }

    pub(crate) fn policy_relationships(
        &self,
        policy: usize,
        steps: &mut usize,
    ) -> (Option<&str>, Option<&str>) {
        let start = self.relationships.partition_point(|row| {
            *steps = steps.saturating_add(1);
            row.policy.get() - 1 < policy
        });
        let mut implied_by = None;
        let mut old_name = None;
        for row in &self.relationships[start..] {
            *steps = steps.saturating_add(1);
            if row.policy.get() - 1 != policy {
                break;
            }
            match row.kind {
                Kind::ImpliedBy => implied_by = Some(row.name.as_ref()),
                Kind::OldName if row.canonical => old_name = Some(row.name.as_ref()),
                Kind::OldName => {}
            }
        }
        (implied_by, old_name)
    }
}
