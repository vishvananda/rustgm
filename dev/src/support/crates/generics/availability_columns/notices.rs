//! Sparse deprecation notices with shared declaration origins.
use super::{AvailabilityAncestry, AvailabilitySince, DeclarationRef, Work};
use crate::support::{
    crates::{MAX_BYTES, MAX_COMPONENTS, generics::source::MAX_REASON_BYTES},
    retained,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeprecationSince {
    Unspecified,
    Future,
    Version(AvailabilitySince),
    Nonstandard(Box<str>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeprecationNotice {
    pub owner: DeclarationRef,
    pub site: u32,
    pub since: DeprecationSince,
    pub note: Option<Box<str>>,
    pub suggestion: Option<Box<str>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeprecatedDeclaration {
    pub declaration: DeclarationRef,
    pub owner: DeclarationRef,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeprecationColumns {
    pub notices: Vec<DeprecationNotice>,
    pub declarations: Vec<DeprecatedDeclaration>,
}

impl DeprecationColumns {
    pub(crate) fn storage_bytes(&self, work: &mut Work) -> Result<usize, &'static str> {
        if self.notices.is_empty()
            || self.notices.len().max(self.declarations.len()) > MAX_COMPONENTS
        {
            return Err("invalid deprecation column size");
        }
        work.charge(self.notices.len().saturating_add(self.declarations.len()))?;
        let mut bytes = retained::sum([
            size_of::<Self>(),
            retained::vector(&self.notices),
            retained::vector(&self.declarations),
        ]);
        for notice in &self.notices {
            let extra = match &notice.since {
                DeprecationSince::Nonstandard(text) => Some(text.as_ref()),
                _ => None,
            };
            for text in [notice.note.as_deref(), notice.suggestion.as_deref(), extra]
                .into_iter()
                .flatten()
            {
                if text.len() > MAX_REASON_BYTES {
                    return Err("deprecation text exceeds transport limit");
                }
                work.charge(text.len())?;
                bytes = bytes.saturating_add(text.len());
            }
        }
        if bytes > MAX_BYTES {
            return Err("deprecation columns exceed retained storage budget");
        }
        Ok(bytes)
    }

    pub(crate) fn validate(
        &self,
        declarations: &[super::super::Declaration],
        ancestry: &AvailabilityAncestry,
        staged: bool,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let mut previous = None;
        for notice in &self.notices {
            work.charge(1)?;
            let actual = declarations
                .get(notice.owner.0 as usize)
                .ok_or("deprecation notice has no actual declaration")?;
            if previous.is_some_and(|owner| owner >= notice.owner.0)
                || actual.local.is_some()
                || notice.site >= actual.sites
            {
                return Err("deprecation notice changed its actual declaration or site");
            }
            if staged
                && (notice.note.is_none()
                    || matches!(
                        notice.since,
                        DeprecationSince::Unspecified | DeprecationSince::Nonstandard(_)
                    ))
            {
                return Err("staged deprecation requires its release and note");
            }
            previous = Some(notice.owner.0);
        }
        let mut previous = None;
        let lookup = self.notices.len().checked_ilog2().unwrap_or(0) as usize + 1;
        for row in &self.declarations {
            work.charge(lookup)?;
            if previous.is_some_and(|owner| owner >= row.declaration.0)
                || !ancestry.contains(row.owner, row.declaration)
                || self
                    .notices
                    .binary_search_by_key(&row.owner.0, |notice| notice.owner.0)
                    .is_err()
            {
                return Err("deprecation declaration changed its actual origin");
            }
            previous = Some(row.declaration.0);
        }
        Ok(())
    }
}
