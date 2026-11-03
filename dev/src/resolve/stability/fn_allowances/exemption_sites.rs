//! A checked lexical exemption never becomes a crate feature or callee permission.
use super::{Error, FeatureName, Grants, Package};
use crate::resolve::{
    expansion::Report,
    model::DefId,
    stability::{Inventory, Library},
};

impl Inventory {
    pub(crate) fn implicit_const_functions(&self) -> &[DefId] {
        self.consts.as_deref().map_or(&[], |rows| &rows.implicit)
    }
    pub(crate) fn const_body_features(&self, owner: DefId) -> Option<&[Box<str>]> {
        let grants = self.consts.as_deref()?.grants.as_deref()?;
        let index = grants
            .rows
            .binary_search_by_key(&owner.0, |row| row.definition.0)
            .ok()?;
        Some(&grants.rows[index].features)
    }
    pub(crate) fn has_const_body_contracts(&self) -> bool {
        self.consts.as_deref().is_some_and(|rows| {
            !rows.implicit.is_empty()
                || rows
                    .grants
                    .as_deref()
                    .is_some_and(|grants| grants.rows.iter().any(|row| !row.features.is_empty()))
        })
    }
    pub(crate) fn const_body_allows(
        &self,
        owner: DefId,
        feature: &FeatureName<'_>,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        *steps = steps.saturating_add(1);
        if *steps > crate::support::crates::MAX_COMPONENTS {
            return Err("const allowance query exceeds work budget");
        }
        let Some(grants) = self
            .consts
            .as_deref()
            .and_then(|rows| rows.grants.as_deref())
        else {
            return Ok(false);
        };
        let depth =
            usize::try_from(usize::BITS - grants.rows.len().leading_zeros()).unwrap_or(usize::MAX);
        *steps = steps.saturating_add(depth + 1);
        if *steps > crate::support::crates::MAX_COMPONENTS {
            return Err("const allowance query exceeds work budget");
        }
        let Ok(index) = grants
            .rows
            .binary_search_by_key(&owner.0, |row| row.definition.0)
        else {
            return Ok(false);
        };
        let names = &grants.rows[index].features;
        let mut left = 0;
        let mut right = names.len();
        while left < right {
            let middle = left + (right - left) / 2;
            *steps = steps.saturating_add(
                names[middle]
                    .len()
                    .saturating_add(feature.spelling().len())
                    .saturating_add(1),
            );
            if *steps > crate::support::crates::MAX_COMPONENTS {
                return Err("const allowance query exceeds work budget");
            }
            match names[middle].as_ref().cmp(feature.spelling()) {
                std::cmp::Ordering::Equal => return Ok(true),
                std::cmp::Ordering::Less => left = middle + 1,
                std::cmp::Ordering::Greater => right = middle,
            }
        }
        Ok(false)
    }
}
fn grants(package: &Package) -> Option<&Grants> {
    package
        .library_policy()
        .and_then(Library::inventory)?
        .consts
        .as_deref()?
        .grants
        .as_deref()
}
pub(crate) fn validate(report: &Report) -> Result<(), Error> {
    let Some(grants) = grants(&report.package) else {
        return Ok(());
    };
    let requested = report
        .package
        .requested_rustc_attributes()
        .map_err(Error::unlocated)?;
    let feature = FeatureName::new("rustc_attrs").map_err(Error::unlocated)?;
    let mut session = report.macro_allowances();
    for row in &grants.rows {
        for site in &row.sites {
            site.validate(&report.package)?;
            if !requested && !session.allows_at(site.owner.file, site.attribute.span, &feature)? {
                return Err(Error::at(
                    site.owner.file,
                    site.attribute.span.start,
                    "const function allowance requires rustc_attrs or its actual attribute allowance",
                ));
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_plain(package: &Package) -> Result<(), Error> {
    let Some(grants) = grants(package) else {
        return Ok(());
    };
    let requested = package
        .requested_rustc_attributes()
        .map_err(Error::unlocated)?;
    for row in &grants.rows {
        for site in &row.sites {
            site.validate(package)?;
            if !requested {
                return Err(Error::at(
                    site.owner.file,
                    site.attribute.span.start,
                    "const function allowance requires rustc_attrs",
                ));
            }
        }
    }
    Ok(())
}
impl super::Site {
    fn validate(&self, package: &Package) -> Result<(), Error> {
        let error = || {
            Error::at(
                self.owner.file,
                self.attribute.span.start,
                "const allowance lost its configured attribute owner",
            )
        };
        let unit = package.unit(self.owner.file);
        let owner = unit
            .owner_indices
            .get(self.owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
            .ok_or_else(error)?;
        if !owner.enabled
            || owner.node != self.owner.node
            || !owner.attributes.contains(&self.effect)
            || unit
                .configured
                .effects
                .get(self.effect)
                .is_none_or(|effect| {
                    effect.attribute.attribute != self.attribute
                        || effect.attribute.effect != crate::expand::configured::Effect::Keep
                })
        {
            return Err(error());
        }
        Ok(())
    }
}
