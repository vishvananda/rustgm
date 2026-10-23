//! Validate complete incoming ownership before publishing any dense remap.
use super::{
    Catalog, ImportedBodyId, ImportedDeclarationId, ImportedOwnerId, Input, Installed,
    catalog::{Artifact, DeclarationRow, OwnerRow, Row},
};
use crate::{
    resolve::model::{DefId, DefKind, Definition},
    support::{
        crates::owner_catalog::{BodyKind, BodyRef},
        hash::HashSet,
        retained,
    },
};

pub(super) fn install(
    catalog: &mut Catalog,
    definitions: &[Definition],
    input: Input,
) -> Result<Installed, &'static str> {
    catalog.mutable()?;
    catalog.charge(
        1 + input.owners.owner().name().len() + input.owners.owner().disambiguator().len(),
    )?;
    if catalog.owners.contains_key(input.owners.owner()) {
        return Err("defining artifact already has an imported body catalog");
    }
    let first = u32::try_from(catalog.rows.len()).map_err(|_| "imported body row overflow")?;
    let declarations =
        u32::try_from(input.definitions.len()).map_err(|_| "imported declaration row overflow")?;
    let first_body = first
        .checked_add(declarations)
        .ok_or("imported owner row overflow")?;
    let len = u32::try_from(input.owners.rows().len()).map_err(|_| "imported body row overflow")?;
    let artifact =
        u32::try_from(catalog.artifacts.len()).map_err(|_| "imported artifact overflow")?;
    quotas(catalog, &input)?;
    let retained = storage(catalog, &input)?;
    validate_definitions(catalog, definitions, &input)?;
    super::locations::validate(catalog, &input)?;
    let rows = rows(catalog, definitions, &input, first_body, artifact)?;
    // Every fallible check, bounded scratch allocation and integer conversion is
    // complete. Source-free ownership remains data, not executable permission.
    let owner = input.owners.owner().clone();
    for (index, definition) in input.definitions.iter().enumerate() {
        // MAX_DECLARATIONS is checked by quotas before this exact dense cast.
        #[expect(
            clippy::cast_possible_truncation,
            reason = "bounded declaration ordinal"
        )]
        catalog.definitions.insert(
            *definition,
            ImportedDeclarationId(ImportedOwnerId {
                epoch: catalog.epoch.0,
                row: first + index as u32,
            }),
        );
    }
    catalog.owners.insert(owner.clone(), artifact);
    catalog.rows.extend(rows);
    catalog.artifacts.push(Artifact {
        owner,
        edition: input.edition,
        definitions: input.definitions,
        first_body,
        body_count: len,
        files: input.files,
        labels: input.labels,
        declaration_labels: input.declaration_labels,
    });
    catalog.bytes = retained;
    Ok(Installed::new(catalog.epoch, first_body, len))
}

fn quotas(catalog: &Catalog, input: &Input) -> Result<(), &'static str> {
    use crate::support::crates::owner_catalog::BodyCatalog;
    if input.definitions.len() > BodyCatalog::MAX_DECLARATIONS
        || input.declarations.len() != input.definitions.len()
        || input.bindings.len() != input.owners.rows().len()
        || catalog.body_count().saturating_add(input.bindings.len()) > catalog.limits.bodies
        || catalog
            .rows
            .len()
            .checked_add(input.bindings.len())
            .and_then(|n| n.checked_add(input.definitions.len()))
            .is_none_or(|n| u32::try_from(n).is_err())
        || catalog.artifacts.len() >= catalog.limits.artifacts
    {
        return Err("imported body ownership exceeds row or artifact budget");
    }
    Ok(())
}

fn storage(catalog: &Catalog, input: &Input) -> Result<usize, &'static str> {
    // Four slots per appended entry conservatively cover amortized vector/map
    // capacity including old+new allocations during growth. Shared immutable
    // owner-catalog/identity Arc pointees are excluded by retained policy.
    let count = input.bindings.len().saturating_add(input.definitions.len());
    let growth = count
        .saturating_mul(4 * size_of::<OwnerRow>())
        .saturating_add(4 * size_of::<Artifact>())
        .saturating_add(4 * size_of::<(crate::support::crates::CrateIdentity, u32)>())
        .saturating_add(
            input
                .definitions
                .len()
                .saturating_mul(4 * size_of::<(DefId, ImportedDeclarationId)>()),
        );
    let mut payload = retained::vector(&input.definitions)
        .saturating_add(retained::vector(&input.files))
        .saturating_add(retained::vector(&input.labels))
        .saturating_add(retained::vector(&input.declaration_labels));
    for file in &input.files {
        catalog.charge(1)?;
        payload = payload.saturating_add(file.path.capacity());
    }
    let retained = catalog.bytes.saturating_add(growth).saturating_add(payload);
    let scratch = retained::vector(&input.bindings)
        .saturating_add(retained::vector(&input.declarations))
        .saturating_add(count.saturating_mul(size_of::<OwnerRow>()))
        .saturating_add(
            input
                .definitions
                .len()
                .saturating_mul(4 * size_of::<DefId>()),
        );
    if retained.saturating_add(scratch) > catalog.limits.bytes {
        return Err("imported body publication exceeds storage budget");
    }
    Ok(retained)
}

fn validate_definitions(
    catalog: &Catalog,
    definitions: &[Definition],
    input: &Input,
) -> Result<(), &'static str> {
    let mut seen = HashSet::default();
    let mut next_site = 0;
    for (id, binding) in input.definitions.iter().zip(&input.declarations) {
        catalog.charge(1)?;
        let definition = definitions
            .get(id.0 as usize)
            .ok_or("imported declaration remap is unknown")?;
        if definition.node.is_some()
            || !matches!(
                definition.kind,
                DefKind::Function
                    | DefKind::Const
                    | DefKind::Static
                    | DefKind::Struct
                    | DefKind::Enum
                    | DefKind::Trait
                    | DefKind::TypeAlias
                    | DefKind::SelfType
                    | DefKind::Generic
                    | DefKind::Parameter
                    | DefKind::Local
            )
            || binding.kind != definition.kind
        {
            return Err("imported body declaration is not a source-free declaration");
        }
        if catalog.definitions.contains_key(id) || !seen.insert(*id) {
            return Err("imported declaration remap collides with another declaration");
        }
        if binding.sites.start != next_site
            || binding.sites.end <= next_site
            || binding.sites.end as usize > input.declaration_labels.len()
        {
            return Err("imported declaration sites are not a complete ordered partition");
        }
        next_site = binding.sites.end;
    }
    if next_site as usize != input.declaration_labels.len() {
        return Err("imported declaration diagnostic sites are unreachable");
    }
    Ok(())
}

fn rows(
    catalog: &Catalog,
    definitions: &[Definition],
    input: &Input,
    first: u32,
    artifact: u32,
) -> Result<Vec<OwnerRow>, &'static str> {
    let mut result = declaration_rows(catalog, input, artifact)?;
    let mut next_site = 0;
    for (index, (owner, binding)) in input.owners.rows().iter().zip(&input.bindings).enumerate() {
        catalog.charge(1)?;
        let definition = *input
            .definitions
            .get(owner.declaration.0 as usize)
            .ok_or("body declaration is outside its complete remap")?;
        let kind = definitions[definition.0 as usize].kind;
        if !matches!(
            (owner.kind, kind),
            (BodyKind::Function, DefKind::Function)
                | (BodyKind::Constant, DefKind::Const)
                | (BodyKind::Static, DefKind::Static)
                | (BodyKind::Closure | BodyKind::Anonymous, _)
        ) {
            return Err("body kind does not match the real imported declaration");
        }
        let parent = owner.parent.map(|parent| id(catalog, first, parent));
        #[expect(clippy::cast_possible_truncation, reason = "bounded body ordinal")]
        let body = BodyRef(index as u32);
        if binding.unit.0 as usize > index {
            return Err("body numbering unit is forward or unknown");
        }
        let unit = id(catalog, first, binding.unit);
        let inherited = owner.parent.and_then(|parent| {
            match &result[input.definitions.len() + parent.0 as usize] {
                OwnerRow::Body(row) => Some(row.unit),
                OwnerRow::Declaration(_) => None,
            }
        });
        if binding.unit != body && inherited != Some(unit) {
            return Err("body numbering unit is neither this body nor its exact parent's unit");
        }
        if binding.sites.start != next_site
            || binding.sites.end <= next_site
            || binding.sites.end as usize > input.labels.len()
        {
            return Err("imported body operation sites are not a complete ordered partition");
        }
        next_site = binding.sites.end;
        result.push(OwnerRow::Body(Row {
            definition,
            kind: owner.kind,
            parent,
            unit,
            artifact,
            body,
            sites: binding.sites.clone(),
        }));
    }
    if next_site as usize != input.labels.len() {
        return Err("imported body diagnostic sites are unreachable");
    }
    Ok(result)
}
fn id(catalog: &Catalog, first: u32, body: BodyRef) -> ImportedBodyId {
    ImportedBodyId(ImportedOwnerId {
        epoch: catalog.epoch.0,
        row: first + body.0,
    })
}

fn declaration_rows(
    catalog: &Catalog,
    input: &Input,
    artifact: u32,
) -> Result<Vec<OwnerRow>, &'static str> {
    let mut rows = Vec::with_capacity(input.definitions.len() + input.bindings.len());
    for (ordinal, (definition, binding)) in input
        .definitions
        .iter()
        .zip(&input.declarations)
        .enumerate()
    {
        catalog.charge(1)?;
        let ordinal = u32::try_from(ordinal).map_err(|_| "imported declaration row overflow")?;
        rows.push(OwnerRow::Declaration(DeclarationRow {
            definition: *definition,
            kind: binding.kind,
            artifact,
            ordinal,
            sites: binding.sites.clone(),
        }));
    }
    Ok(rows)
}
