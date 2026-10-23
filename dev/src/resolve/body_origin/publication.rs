//! Validate complete incoming ownership before publishing any dense remap.
use super::{
    Catalog, ImportedBodyId, Input, Installed,
    catalog::{Artifact, Row},
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
    catalog.charge(
        1 + input.owners.owner().name().len() + input.owners.owner().disambiguator().len(),
    )?;
    if catalog.owners.contains_key(input.owners.owner()) {
        return Err("defining artifact already has an imported body catalog");
    }
    let first = u32::try_from(catalog.rows.len()).map_err(|_| "imported body row overflow")?;
    let len = u32::try_from(input.owners.rows().len()).map_err(|_| "imported body row overflow")?;
    let artifact =
        u32::try_from(catalog.artifacts.len()).map_err(|_| "imported artifact overflow")?;
    quotas(catalog, &input)?;
    let retained = storage(catalog, &input)?;
    validate_definitions(catalog, definitions, &input)?;
    super::locations::validate(catalog, &input)?;
    let rows = rows(catalog, definitions, &input, first, artifact)?;
    // Every fallible check, bounded scratch allocation and integer conversion is
    // complete. Source-free ownership remains data, not executable permission.
    let owner = input.owners.owner().clone();
    for (index, definition) in input.definitions.iter().enumerate() {
        // MAX_DECLARATIONS is checked by quotas before this exact dense cast.
        #[expect(
            clippy::cast_possible_truncation,
            reason = "bounded declaration ordinal"
        )]
        catalog
            .definitions
            .insert(*definition, (artifact, index as u32));
    }
    catalog.owners.insert(owner.clone(), artifact);
    catalog.rows.extend(rows);
    catalog.artifacts.push(Artifact {
        owner,
        edition: input.edition,
        definitions: input.definitions,
        files: input.files,
        labels: input.labels,
    });
    catalog.bytes = retained;
    Ok(Installed::new(catalog.epoch, first, len))
}

fn quotas(catalog: &Catalog, input: &Input) -> Result<(), &'static str> {
    use crate::support::crates::owner_catalog::BodyCatalog;
    if input.definitions.len() > BodyCatalog::MAX_DECLARATIONS
        || input.bindings.len() != input.owners.rows().len()
        || catalog.rows.len().saturating_add(input.bindings.len()) > catalog.limits.bodies
        || catalog
            .rows
            .len()
            .checked_add(input.bindings.len())
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
    let growth = input
        .bindings
        .len()
        .saturating_mul(4 * size_of::<Row>())
        .saturating_add(4 * size_of::<Artifact>())
        .saturating_add(4 * size_of::<(crate::support::crates::CrateIdentity, u32)>())
        .saturating_add(
            input
                .definitions
                .len()
                .saturating_mul(4 * size_of::<(DefId, (u32, u32))>()),
        );
    let mut payload = retained::vector(&input.definitions)
        .saturating_add(retained::vector(&input.files))
        .saturating_add(retained::vector(&input.labels));
    for file in &input.files {
        catalog.charge(1)?;
        payload = payload.saturating_add(file.path.capacity());
    }
    let retained = catalog.bytes.saturating_add(growth).saturating_add(payload);
    let scratch = retained::vector(&input.bindings)
        .saturating_add(input.bindings.len().saturating_mul(size_of::<Row>()))
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
    for id in &input.definitions {
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
            )
        {
            return Err("imported body declaration is not a source-free declaration");
        }
        if catalog.definitions.contains_key(id) || !seen.insert(*id) {
            return Err("imported declaration remap collides with another declaration");
        }
    }
    Ok(())
}

fn rows(
    catalog: &Catalog,
    definitions: &[Definition],
    input: &Input,
    first: u32,
    artifact: u32,
) -> Result<Vec<Row>, &'static str> {
    let mut result = Vec::<Row>::with_capacity(input.bindings.len());
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
        if binding.unit != body
            && owner
                .parent
                .is_none_or(|parent| result[parent.0 as usize].unit != unit)
        {
            return Err("body numbering unit is neither this body nor its exact parent's unit");
        }
        if binding.sites.start != next_site
            || binding.sites.end <= next_site
            || binding.sites.end as usize > input.labels.len()
        {
            return Err("imported body operation sites are not a complete ordered partition");
        }
        next_site = binding.sites.end;
        result.push(Row {
            definition,
            kind: owner.kind,
            parent,
            unit,
            artifact,
            body,
            sites: binding.sites.clone(),
        });
    }
    if next_site as usize != input.labels.len() {
        return Err("imported body diagnostic sites are unreachable");
    }
    Ok(result)
}
fn id(catalog: &Catalog, first: u32, body: BodyRef) -> ImportedBodyId {
    ImportedBodyId {
        epoch: catalog.epoch.0,
        row: first + body.0,
    }
}
