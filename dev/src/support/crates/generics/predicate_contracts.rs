//! Feature requirements are complete declaration data, never API definitions.
use crate::support::crates::generics::{
    Work,
    source::{
        AvailabilityKind, AvailabilityLevel, AvailabilityOrigin, AvailabilityRows, Columns,
        HeaderRef, Requirement, SourceRows,
    },
};

pub(super) fn spelling(name: &str, work: &mut Work) -> Result<(), &'static str> {
    work.charge(name.len().saturating_add(1))?;
    if name.is_empty()
        || name.len() > 256
        || name.as_bytes()[0].is_ascii_digit()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err("feature predicate requires a canonical ASCII identifier within256 bytes");
    }
    Ok(())
}
pub(super) fn legacy(rows: &SourceRows, work: &mut Work) -> Result<(), &'static str> {
    for clause in &rows.clauses {
        work.charge(1)?;
        if matches!(clause.requirement, Requirement::UnstableFeature(_)) {
            return Err("feature predicates require source metadata version four");
        }
    }
    Ok(())
}
pub(super) fn complete(
    columns: &Columns,
    headers: &[Option<HeaderRef>],
    availability: &AvailabilityRows,
    work: &mut Work,
) -> Result<(), &'static str> {
    for clause in &columns.clauses {
        work.charge(1)?;
        let Requirement::UnstableFeature(name) = &clause.requirement else {
            continue;
        };
        spelling(name, work)?;
        let owner = clause.owner.0 as usize;
        if clause.binder.is_some()
            || clause.self_subject.is_some()
            || !matches!(
                headers.get(owner),
                Some(Some(
                    HeaderRef::Trait(_) | HeaderRef::Function(_) | HeaderRef::Implementation(_)
                ))
            )
        {
            return Err("feature predicate has a foreign owner, binder or Self subject");
        }
        let regular = availability
            .declarations
            .get(owner)
            .filter(|row| row.declaration == clause.owner)
            .and_then(|row| row.regular)
            .and_then(|index| availability.policies.get(index.0 as usize))
            .ok_or("feature predicate requires its actual regular unstable owner")?;
        if regular.kind != AvailabilityKind::Regular
            || !matches!(regular.level, AvailabilityLevel::Unstable { .. })
            || !matches!(regular.origin, AvailabilityOrigin::Declaration {owner,..} if owner == clause.owner)
        {
            return Err("feature predicate requires its directly attributed unstable owner");
        }
    }
    Ok(())
}

pub(super) fn legacy_calls(
    rows: &SourceRows,
    headers: &super::HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    for original in &rows.originals {
        for call in &original.calls {
            work.charge(1)?;
            let target = rows.function_applications[call.target.0 as usize]
                .declaration
                .declaration;
            let Some(HeaderRef::Function(index)) = headers.header(target, work)? else {
                return Err("legacy selected call lost its function header");
            };
            if let Some(parent) = rows.functions[index as usize].parent
                && matches!(
                    headers.header(parent, work)?,
                    Some(HeaderRef::Implementation(_))
                )
            {
                return Err(
                    "source selected impl/inherent requests require complete wider transport",
                );
            }
        }
    }
    Ok(())
}
