//! Immutable metadata contracts are joined to the actual installed catalog once.
use super::{ContractKnowledge, ContractOrigin, HeaderAttributeView, ImportedContractSite, Site};
use crate::{
    resolve::{
        body_origin::{Catalog, ImportedDeclarationId},
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::{inlining, model::Declarations, must_use::ContractView},
    support::{
        crates::{
            Dependencies, ExternalDef, MAX_BYTES, MAX_COMPONENTS, generics::source as s,
            owner_catalog::DeclarationRef,
        },
        hash::HashMap,
        retained,
    },
};
use std::{cell::Cell, rc::Rc, sync::Arc};
#[path = "supplied_policy.rs"]
mod provided;
pub(in crate::sema) use provided::plain_function;

struct Header {
    artifact: usize,
    declaration: DeclarationRef,
    handle: ImportedDeclarationId,
    external: Option<ExternalDef>,
}
pub(super) struct Headers {
    catalog: Rc<Catalog>,
    inventory: Arc<Dependencies>,
    entries: HashMap<DefId, Header>,
    /// Version thirteen: the sparse function rows with declared target
    /// features, each checked once its header is installed.
    targeted: Vec<DefId>,
    pub(super) work: Cell<usize>,
}
impl Headers {
    pub(super) fn contains(&self, definition: DefId) -> bool {
        self.entries.contains_key(&definition)
    }
    fn charge(&self, steps: usize) -> Result<(), Error> {
        self.work.set(self.work.get().saturating_add(steps));
        if self.work.get() > MAX_COMPONENTS {
            return Err(Error::unlocated(
                "imported header contract work exceeds profile limit",
            ));
        }
        Ok(())
    }
}
fn profile_has_inventory(
    descriptor: crate::support::crates::SourceDescriptor<'_>,
) -> Result<bool, Error> {
    if descriptor.profile().ordinary_functions() {
        return Ok(true);
    }
    match descriptor.profile() {
        s::Profile::Source2024ModuleContracts
        | s::Profile::Source2024ModuleAvailability
        | s::Profile::Source2024ModuleFeaturePredicates
        | s::Profile::Source2024ModuleDefinitions
        | s::Profile::Source2024ModuleCompatibility
        | s::Profile::Source2024ModuleCallables
        | s::Profile::Source2024ModuleSafety
        | s::Profile::Source2024ModuleConversions
        | s::Profile::Source2024ModuleNominals
        | s::Profile::Source2024ModuleCompilerOperations
        | s::Profile::Source2024ModuleBranches
        | s::Profile::Source2024ModuleBodies => {
            descriptor.module_contracts().map_err(Error::unlocated)?;
        }
        s::Profile::Source2024ProvidedContracts => {
            descriptor.provided_contracts().map_err(Error::unlocated)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
fn profile_has_complete_attributes(profile: s::Profile) -> bool {
    matches!(
        profile,
        s::Profile::Source2024FunctionContracts
            | s::Profile::Source2024ModuleContracts
            | s::Profile::Source2024ModuleAvailability
            | s::Profile::Source2024ModuleFeaturePredicates
            | s::Profile::Source2024ModuleDefinitions
            | s::Profile::Source2024ModuleCompatibility
            | s::Profile::Source2024ModuleCallables
            | s::Profile::Source2024ModuleSafety
            | s::Profile::Source2024ModuleConversions
            | s::Profile::Source2024ModuleNominals
            | s::Profile::Source2024ModuleCompilerOperations
            | s::Profile::Source2024ModuleBranches
            | s::Profile::Source2024ModuleBodies
            | s::Profile::Source2024ProvidedContracts
    )
}
pub(in crate::sema) fn prepare(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    let Some(catalog) = imported.origin_catalog().map_err(Error::unlocated)? else {
        return Ok(());
    };
    let (steps, reserved) = imported.original_reservation_budget();
    let mut bytes = catalog
        .retained_bytes()
        .saturating_add(reserved)
        .saturating_add(size_of::<Headers>());
    for artifact in imported.dependencies().artifacts() {
        if let Some(payload) = artifact.generic_payload() {
            bytes = bytes.saturating_add(payload.retained_bytes());
        }
    }
    let mut headers = Headers {
        catalog,
        inventory: imported.shared_dependencies(),
        entries: HashMap::default(),
        targeted: Vec::new(),
        work: Cell::new(steps),
    };
    let mut rows_with_features = Vec::new();
    for (artifact, interface) in headers.inventory.artifacts().iter().enumerate() {
        let Some(descriptor) = interface.source_descriptor() else {
            continue;
        };
        if !profile_has_inventory(descriptor)? {
            continue;
        }
        let rows = descriptor.rows();
        for declaration in rows
            .functions
            .iter()
            .map(|row| row.declaration)
            .chain(rows.traits.iter().map(|row| row.declaration))
            .chain(rows.nominals.iter().map(|row| row.declaration))
        {
            headers.charge(1)?;
            bytes = bytes.saturating_add(retained::new_map::<DefId, Header>(1));
            if bytes > MAX_BYTES {
                return Err(Error::unlocated(
                    "imported header contracts exceed retained storage",
                ));
            }
            let definition = imported
                .generic_definition(interface.identity(), declaration)
                .map_err(Error::unlocated)?;
            let handle = headers
                .catalog
                .declaration(definition)
                .map_err(Error::unlocated)?;
            if headers
                .entries
                .insert(
                    definition,
                    Header {
                        artifact,
                        declaration,
                        handle,
                        external: None,
                    },
                )
                .is_some()
            {
                return Err(Error::unlocated(
                    "duplicate imported header contract identity",
                ));
            }
        }
        if descriptor.profile().body_nodes() {
            bytes = targeted(
                &headers,
                &mut rows_with_features,
                imported,
                interface,
                bytes,
            )?;
        }
    }
    for (external, definition) in imported.definitions() {
        headers.charge(1)?;
        let Some(header) = headers.entries.get_mut(&definition) else {
            continue;
        };
        let contracts = headers.inventory.artifacts()[header.artifact]
            .source_descriptor()
            .is_some_and(|descriptor| profile_has_complete_attributes(descriptor.profile()));
        if contracts {
            // ExternalDef clones only shared Arc handles. Its already retained
            // immutable path/identity pointees are not copied into this ledger.
            header.external = Some(external.clone());
        }
    }
    headers.targeted = rows_with_features;
    if !headers.entries.is_empty() {
        super::local::bounded_ledger(facts).imported = Some(Box::new(headers));
    }
    Ok(())
}
/// One fixed charge and slot per version-thirteen row with declared target
/// features; names are paid when the row is checked.
fn targeted(
    headers: &Headers,
    found: &mut Vec<DefId>,
    imported: &crate::resolve::crates::Imported,
    interface: &crate::support::crates::Interface,
    mut bytes: usize,
) -> Result<usize, Error> {
    let Some(attributes) = interface
        .source_descriptor()
        .and_then(|descriptor| descriptor.rows().header_attributes())
    else {
        return Ok(bytes);
    };
    for contract in &attributes.functions {
        if contract.target_features.is_none() {
            continue;
        }
        headers.charge(1)?;
        bytes = bytes.saturating_add(size_of::<DefId>());
        if bytes > MAX_BYTES {
            return Err(Error::unlocated(
                "imported header contracts exceed retained storage",
            ));
        }
        found.push(
            imported
                .generic_definition(interface.identity(), contract.declaration)
                .map_err(Error::unlocated)?,
        );
    }
    Ok(bytes)
}
pub(in crate::sema) fn target_feature_rows(facts: &Declarations) -> &[DefId] {
    facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.imported.as_deref())
        .map_or(&[], |headers| headers.targeted.as_slice())
}
/// An imported function's declared target features, re-proved on the
/// consumer's installed header in the exporter's order (trait member, form,
/// unsafe) and then resolved against the consumer's own table and closed.
/// `None` when the row declares none.
pub(in crate::sema) fn target_features(
    facts: &Declarations,
    definition: DefId,
) -> Result<Option<crate::sema::target_features::FeatureSet>, &'static str> {
    use crate::types::{Kind, callable::Abi};
    let Some(headers) = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.imported.as_deref())
    else {
        return Ok(None);
    };
    let Some(header) = headers.entries.get(&definition) else {
        return Ok(None);
    };
    let interface = &headers.inventory.artifacts()[header.artifact];
    let Some(descriptor) = interface.source_descriptor() else {
        return Ok(None);
    };
    if !descriptor.profile().body_nodes() {
        return Ok(None);
    }
    let Some(declared) = descriptor
        .rows()
        .header_contract(header.declaration)?
        .and_then(|contract| contract.target_features.as_deref())
    else {
        return Ok(None);
    };
    let budget = |steps| {
        headers
            .charge(steps)
            .map_err(|_| "imported target-feature work budget exceeded")
    };
    budget(1)?;
    headers
        .catalog
        .validate_declaration(header.handle, definition, DefKind::Function)?;
    let semantic = facts
        .declaration_header(definition)?
        .ok_or("imported target-feature function has no canonical function header")?;
    if semantic.declaration
        != crate::resolve::body_origin::DeclarationOrigin::Imported(header.handle)
    {
        return Err("imported target-feature function changed its actual canonical owner");
    }
    if facts
        .associated
        .owner(definition)
        .and_then(|owner| facts.associated.get(owner))
        .is_some_and(|owner| {
            !matches!(
                owner.kind,
                crate::sema::associated::OwnerKind::ImportedInherent(_)
            )
        })
    {
        return Err(
            "source trait member target features require their trait implementation transport",
        );
    }
    let Some(Kind::Function {
        convention,
        unsafe_,
        ..
    }) = facts.types.kind(semantic.signature)
    else {
        return Err("imported target-feature function has no canonical signature");
    };
    if convention.abi != Abi::Rust || facts.is_const_function(definition) {
        return Err("source target-feature function form requires its wider profile");
    }
    if !unsafe_ {
        return Err("source target-feature function is not declared unsafe");
    }
    crate::sema::target_features::FeatureSet::imported(
        declared.names.iter().map(|name| &**name),
        |growth| budget(growth.work),
        |message| message,
    )
    .map(Some)
}
pub(super) fn query<'a>(
    facts: &'a Declarations,
    report: &Report,
    definition: DefId,
    kind: DefKind,
) -> Result<HeaderAttributeView<'a>, Error> {
    let headers = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.imported.as_deref())
        .ok_or_else(|| Error::unlocated("imported header contract origin is unknown"))?;
    headers.charge(1)?;
    let actual = report
        .resolution
        .crates
        .as_ref()
        .ok_or_else(|| Error::unlocated("imported header catalog is absent"))?
        .origin_catalog()
        .map_err(Error::unlocated)?
        .ok_or_else(|| Error::unlocated("imported header catalog was not installed"))?;
    if !Rc::ptr_eq(&headers.catalog, &actual) {
        return Err(Error::unlocated(
            "imported header contract belongs to another catalog",
        ));
    }
    let header = headers
        .entries
        .get(&definition)
        .ok_or_else(|| Error::unlocated("imported header contract producer state is missing"))?;
    headers
        .catalog
        .validate_declaration(header.handle, definition, kind)
        .map_err(Error::unlocated)?;
    let coordinate = headers
        .catalog
        .declaration_coordinate(header.handle)
        .map_err(Error::unlocated)?;
    let interface = &headers.inventory.artifacts()[header.artifact];
    let descriptor = interface
        .source_descriptor()
        .ok_or_else(|| Error::unlocated("imported header descriptor is absent"))?;
    if coordinate.0 != interface.identity()
        || coordinate.1 != header.declaration
        || coordinate.2 != descriptor.edition()
    {
        return Err(Error::unlocated("imported header canonical origin changed"));
    }
    let row = descriptor
        .rows()
        .header_contract(header.declaration)
        .map_err(Error::unlocated)?;
    let site = |index: Option<u32>| -> Result<ContractOrigin<'a>, Error> {
        Ok(ContractOrigin(Site::Imported(ImportedContractSite {
            coordinate,
            declaration: header.handle,
            site: index
                .map(|index| headers.catalog.declaration_site(header.handle, index))
                .transpose()
                .map_err(Error::unlocated)?,
            external: header.external.as_ref(),
        })))
    };
    let Some(row) = row else {
        return Ok(HeaderAttributeView {
            origin: site(None)?,
            must_use: ContractKnowledge::Unknown,
            inline: ContractKnowledge::Unknown,
        });
    };
    let must_use = if let Some(value) = &row.must_use {
        ContractKnowledge::KnownPresent(ContractView {
            origin: site(Some(value.owner_site))?,
            reason: value.reason.as_deref(),
        })
    } else {
        ContractKnowledge::KnownAbsent
    };
    let inline = if let Some(value) = row.inline {
        ContractKnowledge::KnownPresent(inlining::View {
            origin: site(Some(value.owner_site))?,
            hint: match value.hint {
                s::InlineHint::Hint => inlining::Hint::Hint,
                s::InlineHint::Always => inlining::Hint::Always,
                s::InlineHint::Never => inlining::Hint::Never,
            },
            applicability: value.applicability,
        })
    } else {
        ContractKnowledge::KnownAbsent
    };
    Ok(HeaderAttributeView {
        origin: site(Some(row.declaration_site))?,
        must_use,
        inline,
    })
}

/// Checked legacy profiles cannot encode tracking: their authored producer
/// rejects tracked headers before publication. Version thirteen carries the
/// written policy in its function row, which every consumer re-proves against
/// its own installed header. A missing descriptor is not absence.
pub(super) fn caller_tracking(
    facts: &Declarations,
    definition: DefId,
) -> Result<bool, &'static str> {
    if let Some(linkage) = facts.crates.as_deref()
        && linkage.legacy_caller_tracking_absent(definition)?
    {
        facts
            .declaration_header(definition)?
            .ok_or("legacy caller policy lacks its canonical function")?;
        return Ok(false);
    }
    if let Some(linkage) = facts.crates.as_deref()
        && linkage.selected_caller_tracking_absent(facts, definition)?
    {
        return Ok(false);
    }
    let headers = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.imported.as_deref())
        .ok_or("imported caller policy has no checked inventory")?;
    headers
        .charge(1)
        .map_err(|_| "imported caller policy work budget exceeded")?;
    let header = headers
        .entries
        .get(&definition)
        .ok_or("imported caller policy has no actual declaration")?;
    headers
        .catalog
        .validate_declaration(header.handle, definition, DefKind::Function)?;
    let semantic = facts
        .declaration_header(definition)?
        .ok_or("imported caller policy has no canonical function header")?;
    if semantic.declaration
        != crate::resolve::body_origin::DeclarationOrigin::Imported(header.handle)
    {
        return Err("imported caller policy changed its actual canonical owner");
    }
    let interface = &headers.inventory.artifacts()[header.artifact];
    let descriptor = interface
        .source_descriptor()
        .ok_or("imported caller policy profile is unknown")?;
    match descriptor.profile() {
        s::Profile::Source2024Functions | s::Profile::Source2024FunctionContracts => {}
        s::Profile::Source2024ModuleContracts
        | s::Profile::Source2024ModuleAvailability
        | s::Profile::Source2024ModuleFeaturePredicates
        | s::Profile::Source2024ModuleDefinitions
        | s::Profile::Source2024ModuleCompatibility
        | s::Profile::Source2024ModuleCallables
        | s::Profile::Source2024ModuleSafety
        | s::Profile::Source2024ModuleConversions
        | s::Profile::Source2024ModuleNominals
        | s::Profile::Source2024ModuleCompilerOperations
        | s::Profile::Source2024ModuleBranches
        | s::Profile::Source2024ModuleBodies => {
            descriptor.module_contracts()?;
        }
        _ => return Err("imported caller policy requires an implemented explicit profile"),
    }
    let coordinate = headers.catalog.declaration_coordinate(header.handle)?;
    if coordinate.0 != interface.identity()
        || coordinate.1 != header.declaration
        || coordinate.2 != descriptor.edition()
    {
        return Err("imported caller policy changed its function/profile coordinate");
    }
    if !descriptor.profile().body_nodes() {
        return Ok(false);
    }
    let tracked = descriptor
        .rows()
        .header_contract(header.declaration)?
        .ok_or("imported caller policy lacks its complete function row")?
        .caller_tracking;
    if tracked {
        tracked_header(facts, definition, semantic.signature)?;
    }
    Ok(tracked)
}

/// A transported caller policy is re-proved on the consumer's installed header:
/// a safe or unsafe Rust-ABI function that is free or an inherent method.
fn tracked_header(
    facts: &Declarations,
    definition: DefId,
    signature: crate::types::TypeId,
) -> Result<(), &'static str> {
    use crate::types::{Kind, callable::Abi};
    if !matches!(
        facts.types.kind(signature),
        Some(Kind::Function { convention, .. }) if convention.abi == Abi::Rust
    ) {
        return Err("imported caller tracking requires its Rust-ABI function signature");
    }
    if facts
        .associated
        .owner(definition)
        .and_then(|owner| facts.associated.get(owner))
        .is_some_and(|owner| {
            !matches!(
                owner.kind,
                crate::sema::associated::OwnerKind::ImportedInherent(_)
            )
        })
    {
        return Err("imported caller tracking belongs to a trait member");
    }
    Ok(())
}
