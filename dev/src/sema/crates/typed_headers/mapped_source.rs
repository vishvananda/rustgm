//! One private canonical graph is sealed to an actual immutable source inventory.
pub(super) mod checked_rows;
mod dag_recipes;
mod decoded_original;
#[path = "mapped_source/request_mapping.rs"]
mod original_requests;
pub(crate) use decoded_original::Original as SourceOriginal;
mod ingested_readiness;
pub(in crate::sema) use ingested_readiness::Readiness;
mod origin_ports;
pub(in crate::sema::crates::typed_headers) use origin_ports::require_function;
mod owner_members;
mod versioned_members;
pub(in crate::sema) use versioned_members::PrimitiveMethods;
mod role_headers;
pub(in crate::sema::crates::typed_headers) use role_headers::installed as installed_trait_header;
mod scheme_inputs;
use scheme_inputs::SignaturePort;
pub(in crate::sema::crates::typed_headers) use scheme_inputs::require_regions as require_method_regions;
mod scope_inputs;
pub(super) mod slot_bindings;
use super::{Declarations, Error, Headers, error};
use crate::{
    resolve::{
        body_origin::{Catalog, ImportedDeclarationId},
        crates::Imported,
        model::{DefId, DefKind},
    },
    support::crates::{
        Dependencies, SourceDescriptor, generics::source as s, owner_catalog::DeclarationRef,
    },
    types::{RegionId, TypeId, arguments::Argument, parameters::predicates::Predicate},
};
use std::rc::Rc;
use std::sync::Arc;

pub(super) struct Context<'a> {
    pub imported: &'a Imported,
    pub catalog: &'a Catalog,
    pub descriptor: SourceDescriptor<'a>,
    pub provider_index: u32,
}
/// Private fields prevent an arbitrary type vector from acquiring a source
/// declaration/input correspondence. This seal is not original/caller proof.
pub(in crate::sema) struct Graph {
    store: crate::types::arena::Identity,
    root: ImportedDeclarationId,
    root_row: DeclarationRef,
    provider_index: u32,
    inventory: Arc<Dependencies>,
    origins: Rc<Catalog>,
    nodes: Vec<TypeId>,
    slots: Vec<Argument>,
    regions: Vec<RegionId>,
    applications: Vec<Predicate>,
    trait_inputs: Vec<Option<Arc<[crate::sema::declaration_inputs::Input]>>>,
    impl_inputs: Vec<Option<Arc<[crate::sema::declaration_inputs::Input]>>>,
}
impl Context<'_> {
    pub(super) fn require_functions(&self, budget: &mut Headers) -> Result<(), Error> {
        self.authenticate(budget)
    }
    fn authenticate(&self, budget: &mut Headers) -> Result<(), Error> {
        budget.charge(2)?;
        let catalog = self
            .imported
            .origin_catalog()
            .map_err(error)?
            .ok_or_else(|| error("source graph lacks its completed import catalog"))?;
        if !std::ptr::eq(catalog.as_ref(), self.catalog) {
            return Err(error("source graph changed its actual catalog allocation"));
        }
        let artifact = self
            .imported
            .dependencies()
            .artifacts()
            .get(self.provider_index as usize)
            .ok_or_else(|| error("source graph provider index is outside its inventory"))?;
        let actual = artifact
            .source_descriptor()
            .ok_or_else(|| error("source graph requires its complete actual descriptor"))?;
        if actual.identity() != self.descriptor.identity()
            || actual.profile() != self.descriptor.profile()
            || actual.edition() != self.descriptor.edition()
            || !std::ptr::eq(
                &raw const *actual.rows(),
                &raw const *self.descriptor.rows(),
            )
        {
            return Err(error(
                "source graph belongs to another immutable artifact descriptor",
            ));
        }
        if actual.edition() != crate::support::edition::Edition::Rust2024 {
            return Err(error(
                "canonical source graph requires its explicit supported defining profile",
            ));
        }
        if !actual.profile().ordinary_functions() {
            match actual.profile() {
                s::Profile::Source2024ModuleContracts
                | s::Profile::Source2024ModuleAvailability
                | s::Profile::Source2024ModuleFeaturePredicates
                | s::Profile::Source2024ModuleDefinitions
                | s::Profile::Source2024ModuleCompatibility => {
                    actual.module_contracts().map_err(error)?;
                }
                _ => {
                    return Err(error(
                        "canonical source graph requires its supported defining profile",
                    ));
                }
            }
        }
        Ok(())
    }
    pub(super) fn definition(
        &self,
        reference: DeclarationRef,
        budget: &mut Headers,
    ) -> Result<DefId, Error> {
        budget.charge(1)?;
        let rows = self.descriptor.rows();
        let row = rows
            .declarations
            .get(reference.0 as usize)
            .ok_or_else(|| error("source graph declaration is outside its actual owner table"))?;
        let definition = self
            .imported
            .generic_definition(self.descriptor.identity(), reference)
            .map_err(error)?;
        let declaration = self.catalog.declaration(definition).map_err(error)?;
        self.catalog
            .validate_declaration(
                declaration,
                definition,
                slot_bindings::kind(row.definition.kind),
            )
            .map_err(error)?;
        Ok(definition)
    }
}
impl Graph {
    pub(super) fn prepare(
        context: &Context<'_>,
        facts: &mut Declarations,
        budget: &mut Headers,
    ) -> Result<Self, Error> {
        context.authenticate(budget)?;
        if facts
            .crates
            .as_ref()
            .and_then(|linkage| linkage.origin_catalog())
            .is_none_or(|catalog| !std::ptr::eq(catalog, context.catalog))
        {
            return Err(error(
                "source graph remap belongs to another actual semantic catalog",
            ));
        }
        let rows = context.descriptor.rows();
        super::epoch_fences::require(rows, |steps| budget.charge(steps))?;
        budget.reserve(crate::types::arena::Identity::allocation_bytes())?;
        let store = facts.types.arena_identity();
        let mut root_row = None;
        for (index, row) in rows.declarations.iter().enumerate() {
            budget.charge(1)?;
            if row.definition.parent.is_none() {
                let reference = DeclarationRef(
                    u32::try_from(index)
                        .map_err(|_| error("source graph root ordinal overflow"))?,
                );
                if root_row.replace(reference).is_some() {
                    return Err(error("source graph has more than one actual module owner"));
                }
            }
        }
        let root_row = root_row.ok_or_else(|| error("source graph has no actual module owner"))?;
        let definition = context.definition(root_row, budget)?;
        let root = context.catalog.declaration(definition).map_err(error)?;
        context
            .catalog
            .validate_declaration(root, definition, DefKind::Module)
            .map_err(error)?;
        // Retained maps coexist with the visiting/output vectors and a cursor
        // frontier bounded by node count, even for repeated shared children.
        let bytes = facts
            .types
            .node_allocation_bound(rows.types.len().saturating_add(rows.slots.len()))
            .saturating_add(rows.types.len().saturating_mul(
                size_of::<Option<TypeId>>() + size_of::<bool>() + 2 * size_of::<usize>(),
            ))
            .saturating_add(rows.slots.len().saturating_mul(size_of::<Argument>()))
            .saturating_add(rows.regions.len().saturating_mul(size_of::<RegionId>()));
        budget.reserve(size_of::<Self>().saturating_add(bytes))?;
        let slots = slot_bindings::prepare(context, facts, budget)?;
        let regions = slot_bindings::regions(context, budget)?;
        let nodes = dag_recipes::prepare(context, facts, &slots, &regions, root, budget)?;
        budget.observed.vector(&slots);
        budget.observed.vector(&regions);
        budget.observed.vector(&nodes);
        let origins = context
            .imported
            .origin_catalog()
            .map_err(error)?
            .ok_or_else(|| error("source graph lost its completed catalog before sealing"))?;
        Ok(Self {
            store,
            root,
            root_row,
            provider_index: context.provider_index,
            inventory: context.imported.shared_dependencies(),
            origins,
            nodes,
            slots,
            regions,
            applications: Vec::new(),
            trait_inputs: Vec::new(),
            impl_inputs: Vec::new(),
        })
    }
    pub(in crate::sema) fn require_store(
        &self,
        store: &crate::types::Store,
    ) -> Result<(), &'static str> {
        if !store.has_identity(&self.store) {
            return Err("source canonical headers belong to another actual type arena");
        }
        Ok(())
    }
    pub(super) fn require(
        &self,
        imported: &Imported,
        catalog: &Catalog,
        descriptor: SourceDescriptor<'_>,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        if !std::ptr::eq(self.origins.as_ref(), catalog) {
            return Err(error(
                "source canonical graph belongs to another actual catalog allocation",
            ));
        }
        let context = Context {
            imported,
            catalog,
            descriptor,
            provider_index: self.provider_index,
        };
        context.authenticate(budget)?;
        let definition = context.definition(self.root_row, budget)?;
        context
            .catalog
            .validate_declaration(self.root, definition, DefKind::Module)
            .map_err(error)?;
        Ok(())
    }
    pub(super) fn finish_applications(&mut self, applications: Vec<Predicate>) {
        self.applications = applications;
    }
    pub(super) fn application(
        &self,
        reference: s::TraitApplicationRef,
    ) -> Result<Predicate, &'static str> {
        self.applications
            .get(reference.0 as usize)
            .copied()
            .ok_or("source canonical application is outside its completed table")
    }
    pub(in crate::sema::crates::typed_headers) fn requirement(
        &self,
        raw: &s::Requirement,
    ) -> Result<Predicate, &'static str> {
        let mut mapping = self;
        original_requests::map(&mut mapping, raw)
    }

    pub(super) fn slot(&self, reference: s::SlotRef) -> Result<Argument, Error> {
        self.slots
            .get(reference.0 as usize)
            .copied()
            .ok_or_else(|| error("source graph has no canonical argument at its actual slot"))
    }
    pub(super) fn signature(
        &self,
        context: &Context<'_>,
        facts: &mut Declarations,
        owner: DeclarationRef,
        expected: s::BinderKind,
        signature: crate::support::crates::generics::Signature,
        budget: &mut Headers,
    ) -> Result<TypeId, Error> {
        self.require_store(&facts.types).map_err(error)?;
        self.require(
            context.imported,
            context.catalog,
            context.descriptor,
            budget,
        )?;
        let rows = context.descriptor.rows();
        budget.charge(1)?;
        let count = super::signature_shapes::check(rows, Some((owner, expected)), signature)?;
        let roots = signature.parameters.slice(&rows.roots).map_err(error)?;
        budget.charge(roots.len().saturating_add(1))?;
        budget.reserve(
            roots
                .len()
                .saturating_mul(4 * size_of::<TypeId>())
                .saturating_add(facts.types.node_allocation_bound(1)),
        )?;
        let parameters = roots
            .iter()
            .map(|&root| self.ty(root))
            .collect::<Result<Vec<_>, _>>()?;
        super::signature_shapes::intern(
            &mut facts.types,
            count,
            signature,
            &parameters,
            self.ty(signature.result)?,
        )
    }
    pub(super) fn ty(
        &self,
        reference: crate::support::crates::generics::TypeRef,
    ) -> Result<TypeId, Error> {
        self.nodes
            .get(reference.0 as usize)
            .copied()
            .ok_or_else(|| error("source graph has no canonical type at its actual root"))
    }
    pub(super) fn region(&self, reference: s::RegionRef) -> Result<RegionId, Error> {
        self.regions
            .get(reference.0 as usize)
            .copied()
            .ok_or_else(|| error("source graph has no canonical region at its actual root"))
    }
}
