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
        work: Cell::new(steps),
    };
    for (artifact, interface) in headers.inventory.artifacts().iter().enumerate() {
        let Some(descriptor) = interface.source_descriptor() else {
            continue;
        };
        if !descriptor.profile().ordinary_functions() {
            match descriptor.profile() {
                s::Profile::Source2024ModuleContracts => {
                    descriptor.module_contracts().map_err(Error::unlocated)?;
                }
                _ => continue,
            }
        }
        let rows = descriptor.rows();
        for declaration in rows
            .functions
            .iter()
            .map(|row| row.declaration)
            .chain(rows.traits.iter().map(|row| row.declaration))
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
    }
    for (external, definition) in imported.definitions() {
        headers.charge(1)?;
        let Some(header) = headers.entries.get_mut(&definition) else {
            continue;
        };
        let contracts = headers.inventory.artifacts()[header.artifact]
            .source_descriptor()
            .is_some_and(|descriptor| {
                matches!(
                    descriptor.profile(),
                    s::Profile::Source2024FunctionContracts | s::Profile::Source2024ModuleContracts
                )
            });
        if contracts {
            // ExternalDef clones only shared Arc handles. Its already retained
            // immutable path/identity pointees are not copied into this ledger.
            header.external = Some(external.clone());
        }
    }
    if !headers.entries.is_empty() {
        facts
            .must_use_contracts
            .get_or_insert_default()
            .headers
            .get_or_insert_default()
            .imported = Some(Box::new(headers));
    }
    Ok(())
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

/// These checked legacy profiles cannot encode tracking: their authored producer
/// rejects tracked headers before publication. A missing descriptor is not absence.
pub(super) fn caller_tracking_absent(
    facts: &Declarations,
    definition: DefId,
) -> Result<(), &'static str> {
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
        s::Profile::Source2024ModuleContracts => {
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
    Ok(())
}
