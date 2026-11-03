//! Const body policy is checked in its own canonical provider, never its client.
use super::{Original, charge, s};
use crate::support::crates::owner_catalog::DeclarationRef;
impl Original<'_> {
    pub(crate) fn check_const_policies(&self, steps: &mut usize) -> Result<(), &'static str> {
        charge(steps)?;
        if !self.const_context() {
            return Ok(());
        }
        if !self.rows.descriptor().profile().unconditional_callables() {
            return Err("source const body policy requires its explicit callable profile");
        }
        let rows = self.rows.availability()?;
        if !safe(rows, self.raw().declaration, steps)? {
            return Ok(());
        }
        for request in &self.raw().calls {
            charge(steps)?;
            let application = self
                .rows
                .function_applications
                .get(request.target.0 as usize)
                .ok_or("const body policy lost its actual call application")?;
            if application.declaration.artifact != s::ArtifactRef(0) {
                return Err(
                    "foreign source const body policy requires its complete defining origin",
                );
            }
            if !safe(rows, application.declaration.declaration, steps)?
                && !exempt(
                    rows,
                    self.raw().declaration,
                    application.declaration.declaration,
                    steps,
                )?
            {
                return Err(
                    "source const-stable body requires its actual const-safe callee policy or wider allowance transport",
                );
            }
        }
        Ok(())
    }
}
fn safe(
    rows: &s::AvailabilityRows,
    declaration: DeclarationRef,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    charge(steps)?;
    let row = rows
        .declarations
        .get(declaration.0 as usize)
        .filter(|row| row.declaration == declaration)
        .ok_or("const body policy lost its canonical declaration coverage")?;
    let Some(reference) = row.constant else {
        let Some(contracts) = &rows.const_bodies else {
            return Ok(false);
        };
        let mut left = 0;
        let mut right = contracts.implicit.len();
        while left < right {
            charge(steps)?;
            let middle = left + (right - left) / 2;
            match contracts.implicit[middle].0.cmp(&declaration.0) {
                std::cmp::Ordering::Equal => return Ok(true),
                std::cmp::Ordering::Less => left = middle + 1,
                std::cmp::Ordering::Greater => right = middle,
            }
        }
        return Ok(false);
    };
    charge(steps)?;
    let policy = rows
        .policies
        .get(reference.0 as usize)
        .ok_or("const body policy has an absent actual policy")?;
    if policy.kind != s::AvailabilityKind::Const {
        // Inherited regular policy is an independently validated const-safe case.
        return if row.regular == row.constant {
            Ok(true)
        } else {
            Err("const body policy changed its actual policy kind")
        };
    }
    Ok(matches!(policy.level, s::AvailabilityLevel::Stable(_)) || row.regular == row.constant)
}
fn exempt(
    rows: &s::AvailabilityRows,
    owner: DeclarationRef,
    target: DeclarationRef,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    charge(steps)?;
    let Some(contracts) = &rows.const_bodies else {
        return Ok(false);
    };
    let policy = rows
        .declarations
        .get(target.0 as usize)
        .and_then(|row| row.constant)
        .and_then(|reference| rows.policies.get(reference.0 as usize));
    let Some(policy) = policy.filter(|policy| policy.kind == s::AvailabilityKind::Const) else {
        return Ok(false);
    };
    let mut left = 0;
    let mut right = contracts.grants.len();
    while left < right {
        let middle = left + (right - left) / 2;
        let grant = &contracts.grants[middle];
        *steps = steps.saturating_add(grant.feature.len().saturating_add(policy.feature.len()));
        charge(steps)?;
        match (grant.owner.0, grant.feature.as_str()).cmp(&(owner.0, policy.feature.as_str())) {
            std::cmp::Ordering::Equal => return Ok(true),
            std::cmp::Ordering::Less => left = middle + 1,
            std::cmp::Ordering::Greater => right = middle,
        }
    }
    Ok(false)
}
