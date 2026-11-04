//! Final feature names are resolved only after enabled expansion and name resolution.
use super::{Draft, Inventory, owners};
use crate::resolve::{
    model::{DefKind, Resolution, Target},
    package::{Error, Package},
    unit::FileId,
};

pub(in crate::resolve) fn complete(
    package: &Package,
    facts: &Resolution,
    draft: Option<Draft<'_>>,
    pending: Option<&super::Library>,
) -> Result<Option<Inventory>, Error> {
    let Some(library) = package.library.as_deref().or(pending) else {
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
    draft.check_feature_requests(package, imported, &library.requests)?;
    if !library.staged && !draft.policies.is_empty() {
        return Err(Error::at(
            FileId(0),
            0,
            "local library feature definitions require staged_api",
        ));
    }
    let mut owners = if library.staged {
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
            fields: None,
        }
    };
    super::fn_allowances::finish(package, facts, &mut draft, &mut owners.consts)?;
    validate_type_boundary(package, facts, &mut draft)?;
    let mut inventory = Inventory {
        policies: std::mem::take(&mut draft.policies),
        relationships: std::mem::take(&mut draft.relationships),
        declarations: owners.declarations,
        imports: owners.imports,
        consts: owners.consts,
        fields: owners.fields,
        steps: draft.steps,
    };
    draft.retain_inventory(library, &mut inventory)?;
    Ok(Some(inventory))
}

fn validate_type_boundary(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft<'_>,
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
