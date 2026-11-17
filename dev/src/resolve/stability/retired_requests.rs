//! Imported removed data is a fallback index, disjoint from active feature definitions.
use super::availability_imports::Imports;
use crate::{
    resolve::package::Error,
    support::{crates::RemovedFeatures, hash::HashMap, retained::Growth},
};
#[derive(Clone, Copy)]
struct Entry {
    artifact: usize,
    row: usize,
}
#[derive(Default)]
pub(super) struct Index {
    entries: HashMap<String, Entry>,
    spelling_bytes: usize,
}
impl Imports {
    pub(super) fn collect_removed(
        &mut self,
        artifact: usize,
        table: &RemovedFeatures,
    ) -> Result<(), Error> {
        for (row, declaration) in table.entries().iter().enumerate() {
            self.charge(declaration.feature.len().saturating_add(1), 0)?;
            if self
                .retirements
                .as_ref()
                .is_some_and(|index| index.entries.contains_key(&declaration.feature))
            {
                continue;
            }
            if self.retirements.is_none() {
                self.charge(1, size_of::<Index>())?;
                self.retirements = Some(Box::default());
            }
            let index = self
                .retirements
                .as_ref()
                .ok_or_else(|| Error::unlocated("removed request index is missing"))?;
            let growth =
                Growth::map::<String, Entry>(index.entries.len(), index.entries.capacity(), 1);
            let rehash = if growth.bytes == 0 {
                0
            } else {
                index.spelling_bytes
            };
            self.charge(
                growth
                    .work
                    .saturating_add(rehash)
                    .saturating_add(declaration.feature.len()),
                growth.bytes.saturating_add(declaration.feature.len()),
            )?;
            let index = self
                .retirements
                .as_mut()
                .ok_or_else(|| Error::unlocated("removed request index changed"))?;
            index
                .entries
                .try_reserve(1)
                .map_err(|_| Error::unlocated("removed request index allocation failed"))?;
            index.spelling_bytes = index
                .spelling_bytes
                .saturating_add(declaration.feature.len());
            index
                .entries
                .insert(declaration.feature.clone(), Entry { artifact, row });
        }
        Ok(())
    }
    pub(in crate::resolve::stability) fn removed_message(
        &self,
        name: &str,
    ) -> Result<Option<String>, Error> {
        self.charge(name.len().saturating_add(1), 0)?;
        let Some(entry) = self
            .retirements
            .as_ref()
            .and_then(|index| index.entries.get(name))
        else {
            return Ok(None);
        };
        let artifact = self
            .inventory
            .artifacts()
            .get(entry.artifact)
            .ok_or_else(|| Error::unlocated("removed request artifact is foreign"))?;
        let table = artifact
            .removed_features()
            .ok_or_else(|| Error::unlocated("removed request attachment is missing"))?;
        if table.owner() != artifact.identity() {
            return Err(Error::unlocated(
                "removed request table has a foreign owner",
            ));
        }
        let row = table
            .entries()
            .get(entry.row)
            .ok_or_else(|| Error::unlocated("removed request row is foreign"))?;
        let fixed = "library feature `` has been removed since ..: ; see ".len();
        let digits = "65535".len() * 3;
        let bytes = name
            .len()
            .saturating_add(row.reason.len())
            .saturating_add(row.link.len())
            .saturating_add(fixed)
            .saturating_add(digits);
        let growth = Growth::vector::<u8>(0, 0, bytes);
        self.charge(bytes, growth.bytes)?;
        Ok(Some(format!(
            "library feature `{name}` has been removed since {}.{}.{}: {}; see {}",
            row.since.major, row.since.minor, row.since.patch, row.reason, row.link
        )))
    }
}
