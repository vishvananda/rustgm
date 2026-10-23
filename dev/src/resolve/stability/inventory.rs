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
    if facts.crates.is_some() {
        return Err(Error::at(
            FileId(0),
            0,
            "dependency library stability contracts are not implemented",
        ));
    }
    let storage = crate::support::retained::new_map::<&str, ()>(draft.policies.len());
    draft.charge(draft.policies.len(), storage)?;
    let features: HashSet<_> = draft
        .policies
        .iter()
        .map(|policy| policy.feature.as_ref())
        .collect();
    if library
        .requests
        .iter()
        .any(|name| !features.contains(name.as_ref()))
    {
        return Err(Error::at(
            FileId(0),
            0,
            "unsupported active source-core feature or undefined library feature request",
        ));
    }
    if !library.staged {
        return Err(Error::at(
            FileId(0),
            0,
            "local library feature definitions require staged_api",
        ));
    }
    let owners = owners::collect(package, facts, &mut draft)?;
    owners::reachable(package, facts, &owners, &mut draft)?;
    validate_type_boundary(package, facts, &mut draft)?;
    Ok(Some(Inventory {
        policies: draft.policies,
        declarations: owners.declarations,
        imports: owners.imports,
        steps: draft.steps,
    }))
}

fn validate_type_boundary(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft,
) -> Result<(), Error> {
    // Nominal signature exposure needs the semantic type graph. Until that
    // consumer exists, reject nominal/trait use explicitly, including bootstrap
    // types, instead of silently missing a public stability requirement.
    for reference in &facts.references {
        draft.charge(1, 0)?;
        let definition = match reference.target {
            Target::Definition(id) | Target::Associated(id) => &facts.definitions[id.0 as usize],
            _ => continue,
        };
        if matches!(
            definition.kind,
            DefKind::Struct
                | DefKind::Enum
                | DefKind::Trait
                | DefKind::BuiltinTrait
                | DefKind::TypeAlias
                | DefKind::ExternCrate
        ) {
            return Err(owners::error(
                package,
                reference.node,
                "nominal and external library stability reachability is not implemented",
            ));
        }
    }
    Ok(())
}
