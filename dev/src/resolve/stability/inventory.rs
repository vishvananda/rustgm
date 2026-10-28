//! Final feature names are resolved only after enabled expansion and name resolution.
use super::{Draft, Inventory, owners};
use crate::{
    resolve::{
        model::{DefKind, Resolution, Target},
        package::{Error, Package},
        unit::FileId,
    },
    support::hash::HashSet,
};

pub(in crate::resolve) fn complete(
    package: &Package,
    facts: &Resolution,
    draft: Option<Draft>,
) -> Result<Option<Inventory>, Error> {
    let Some(library) = package.library.as_deref() else {
        return Ok(None);
    };
    if library.inventory.is_some() {
        return Err(Error::at(
            FileId(0),
            0,
            "library stability inventory was already completed",
        ));
    }
    let mut draft = draft.unwrap_or_default();
    draft.charge(library.requests.len(), library.request_bytes)?;
    let imported = facts
        .crates
        .as_deref()
        .and_then(|imports| imports.availability.as_deref());
    if facts.crates.is_some() && imported.is_none() {
        return Err(Error::at(
            FileId(0),
            0,
            "dependency library stability contracts are not implemented",
        ));
    }
    let storage = crate::support::retained::new_map::<&str, ()>(draft.policies.len());
    draft.charge(
        draft
            .policies
            .iter()
            .map(|policy| policy.feature.len().saturating_add(1))
            .sum::<usize>()
            .saturating_add(
                library
                    .requests
                    .iter()
                    .map(|name| name.len().saturating_add(1))
                    .sum::<usize>(),
            ),
        storage,
    )?;
    let features: HashSet<_> = draft
        .policies
        .iter()
        .map(|policy| policy.feature.as_ref())
        .collect();
    for name in &library.requests {
        if !features.contains(name.as_ref())
            && !imported
                .map(|imports| imports.has_feature(name))
                .transpose()?
                .unwrap_or(false)
        {
            return Err(Error::at(
                FileId(0),
                0,
                "unsupported active source-core feature or undefined library feature request",
            ));
        }
    }
    if !library.staged && !draft.policies.is_empty() {
        return Err(Error::at(
            FileId(0),
            0,
            "local library feature definitions require staged_api",
        ));
    }
    let owners = if library.staged {
        let owners = owners::collect(package, facts, &mut draft)?;
        owners::reachable(package, facts, &owners, &mut draft)?;
        owners
    } else {
        let count = facts.definitions.len().saturating_add(facts.imports.len());
        draft.charge(
            count,
            count.saturating_mul(size_of::<Option<super::PolicyId>>()),
        )?;
        owners::Owners {
            declarations: vec![None; facts.definitions.len()],
            imports: vec![None; facts.imports.len()],
            consts: None,
        }
    };
    validate_type_boundary(package, facts, &mut draft)?;
    Ok(Some(Inventory {
        policies: draft.policies,
        declarations: owners.declarations,
        imports: owners.imports,
        consts: owners.consts,
        steps: draft.steps,
    }))
}

fn validate_type_boundary(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft,
) -> Result<(), Error> {
    // Local declarations are checked in the same scope inventory. Merely naming
    // an in-crate API is not a cross-crate unstable-feature use (rustc stability.rs).
    // Dependencies remain closed above and at both artifact publication ports.
    for reference in &facts.references {
        draft.charge(1, 0)?;
        let definition = match reference.target {
            Target::Definition(id) | Target::Associated(id) => &facts.definitions[id.0 as usize],
            _ => continue,
        };
        if definition.kind == DefKind::ExternCrate
            && facts
                .crates
                .as_deref()
                .and_then(|imports| imports.availability.as_deref())
                .is_none()
        {
            return Err(owners::error(
                package,
                reference.node,
                "external library stability contracts are not implemented",
            ));
        }
    }
    Ok(())
}
