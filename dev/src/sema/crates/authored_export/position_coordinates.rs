//! Version-fourteen deduplicated caller-site storage for non-tracked originals.
use super::{Stage, s};
use crate::support::{crates::owner_catalog::BodyRef, hash::HashMap};

impl Stage {
    pub(crate) fn record_caller_site(
        &mut self,
        body: BodyRef,
        value: u32,
        filename: &str,
        line: u32,
        column: u32,
    ) -> Result<(), &'static str> {
        if filename.is_empty()
            || filename.len() > 4096
            || filename.as_bytes().contains(&0)
            || line == 0
            || column == 0
        {
            return Err("source caller site has invalid filename, line or column");
        }
        self.charge(filename.len().saturating_add(1))?;
        if self.caller_site_files.is_none() {
            self.reserve(size_of::<HashMap<Box<str>, u32>>())?;
            self.caller_site_files = Some(Box::default());
        }
        if self.rows.cold().caller_sites.is_none() {
            self.reserve(size_of::<s::CallerSiteColumns>())?;
            self.cold()?.caller_sites = Some(Box::default());
        }
        let existing = self
            .caller_site_files
            .as_deref()
            .and_then(|files| files.get(filename).copied());
        let file = if let Some(index) = existing {
            index
        } else {
            let next = self
                .rows
                .cold()
                .caller_sites
                .as_deref()
                .map_or(0, |sites| sites.files.len());
            let index =
                u32::try_from(next).map_err(|_| "source caller site file ordinal overflow")?;
            self.reserve(
                filename
                    .len()
                    .saturating_mul(2)
                    .saturating_add(4 * size_of::<Box<str>>())
                    .saturating_add(4 * size_of::<(Box<str>, u32)>()),
            )?;
            let owned: Box<str> = Box::from(filename);
            self.cold()?
                .caller_sites
                .as_deref_mut()
                .ok_or("source caller site table disappeared")?
                .files
                .push(owned.clone());
            self.caller_site_files
                .as_deref_mut()
                .ok_or("source caller site file map disappeared")?
                .insert(owned, index);
            index
        };
        self.reserve(4 * size_of::<s::CallerSiteRow>())?;
        self.cold()?
            .caller_sites
            .as_deref_mut()
            .ok_or("source caller site table disappeared")?
            .rows
            .push(s::CallerSiteRow {
                body,
                value,
                file,
                line,
                column,
            });
        Ok(())
    }

    pub(super) fn sort_caller_sites(&mut self) -> Result<(), &'static str> {
        let count = self
            .rows
            .cold()
            .caller_sites
            .as_deref()
            .map_or(0, |sites| sites.rows.len());
        if count == 0 {
            return Ok(());
        }
        self.charge(count.saturating_mul(count.ilog2() as usize + 1))?;
        if let Some(sites) = self
            .rows
            .cold
            .as_deref_mut()
            .and_then(|cold| cold.caller_sites.as_deref_mut())
        {
            sites
                .rows
                .sort_unstable_by_key(|row| (row.body.0, row.value));
        }
        Ok(())
    }
}
