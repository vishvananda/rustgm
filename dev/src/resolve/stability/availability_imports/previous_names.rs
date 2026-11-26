//! Former names diagnose requests in dependency and canonical feature order.
use super::{Error, HashMap, Imports, retained};

struct Rename {
    artifact: usize,
    canonical: String,
}

#[derive(Default)]
pub(in crate::resolve::stability) struct Index {
    entries: HashMap<String, Rename>,
    spelling_bytes: usize,
}

impl Imports {
    pub(super) fn collect_previous_name(
        &mut self,
        artifact: usize,
        previous: &str,
        canonical: &str,
    ) -> Result<(), Error> {
        self.charge(previous.len().saturating_add(1), 0)?;
        if !super::super::attributes::identifier(previous) {
            return Ok(());
        }
        let canonical = self.spelling(canonical)?;
        if let Some(index) = self.previous_names()
            && index.entries.get(previous).is_some_and(|old| {
                (old.artifact, old.canonical.as_str()) <= (artifact, canonical.as_ref())
            })
        {
            return Ok(());
        }
        if self.previous_names().is_none() {
            self.charge(1, size_of::<Index>())?;
            self.reserve_request_notices()?;
            self.retirements
                .as_mut()
                .ok_or_else(|| Error::unlocated("missing imported feature request notices"))?
                .previous_names = Some(Box::default());
        }
        let index = self
            .previous_names()
            .ok_or_else(|| Error::unlocated("missing imported feature rename inventory"))?;
        let growth = retained::Growth::map::<String, Rename>(
            index.entries.len(),
            index.entries.capacity(),
            1,
        );
        let names = previous.len().saturating_add(canonical.len());
        let rehash = if growth.bytes == 0 {
            0
        } else {
            index.spelling_bytes
        };
        self.charge(
            growth.work.saturating_add(rehash).saturating_add(names),
            growth.bytes.saturating_add(names),
        )?;
        let index = self
            .retirements
            .as_deref_mut()
            .and_then(|notices| notices.previous_names.as_deref_mut())
            .ok_or_else(|| Error::unlocated("missing imported feature rename inventory"))?;
        index
            .entries
            .try_reserve(1)
            .map_err(|_| Error::unlocated("imported feature rename index allocation failed"))?;
        index.spelling_bytes = index.spelling_bytes.saturating_add(previous.len());
        index.entries.insert(
            previous.to_owned(),
            Rename {
                artifact,
                canonical: canonical.into_owned(),
            },
        );
        Ok(())
    }

    pub(in crate::resolve::stability) fn renamed_message(
        &self,
        requested: &str,
    ) -> Result<Option<String>, Error> {
        let Some(index) = self.previous_names() else {
            return Ok(None);
        };
        self.charge(requested.len().saturating_add(1), 0)?;
        let Some(rename) = index.entries.get(requested) else {
            return Ok(None);
        };
        if self.features.get(requested).is_some_and(|&artifact| {
            (artifact, requested) <= (rename.artifact, rename.canonical.as_str())
        }) {
            return Ok(None);
        }
        let bytes = requested
            .len()
            .saturating_add(rename.canonical.len())
            .saturating_add("library feature `` has been renamed to ``".len());
        self.charge(bytes, bytes)?;
        Ok(Some(format!(
            "library feature `{requested}` has been renamed to `{}`",
            rename.canonical,
        )))
    }

    fn previous_names(&self) -> Option<&Index> {
        self.retirements.as_deref()?.previous_names.as_deref()
    }
}
