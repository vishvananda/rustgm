//! Only actual reserved schema can populate the private lazy slot/region maps.
use super::super::{error, mapped_source::slot_bindings};
use super::{Availability, Demand, Dependency, Port};
use crate::{
    resolve::{
        body_origin::ImportedDeclarationId,
        crates::{ReservationWork, unpaid_provided::RecipeView},
        model::DefId,
        package::Error,
    },
    support::crates::{generics::source as s, owner_catalog::DeclarationRef},
    types::{RegionId, Store, arguments::Argument},
};

pub(in crate::sema::crates::typed_headers) struct Maps {
    pub slots: Vec<Option<Argument>>,
    pub regions: Vec<Option<RegionId>>,
}
pub(in crate::sema::crates::typed_headers) struct Reserved<'rows, 'maps> {
    view: RecipeView<'rows>,
    rows: s::ValidatedView<'rows>,
    maps: &'maps mut Maps,
}
impl<'rows, 'maps> Reserved<'rows, 'maps> {
    pub(in crate::sema::crates::typed_headers) fn recipe_view(&self) -> &RecipeView<'rows> {
        &self.view
    }
    pub(in crate::sema::crates::typed_headers) fn input_site(
        &self,
        owner: DeclarationRef,
        site: u32,
        work: &mut ReservationWork,
    ) -> Result<crate::resolve::body_origin::ProofSite, Error> {
        self.view.input_site(owner, site, work)
    }
    pub(in crate::sema::crates::typed_headers) fn new(
        view: RecipeView<'rows>,
        maps: &'maps mut Maps,
    ) -> Result<Self, Error> {
        let rows = view.rows()?;
        Ok(Self { view, rows, maps })
    }
    pub(in crate::sema::crates::typed_headers) fn declaration(
        &self,
        row: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<ImportedDeclarationId, Error> {
        self.view.declaration(row, work)
    }
    pub(in crate::sema::crates::typed_headers) fn body(
        &self,
        row: crate::support::crates::owner_catalog::BodyRef,
        work: &mut ReservationWork,
    ) -> Result<crate::resolve::body_origin::ImportedBodyId, Error> {
        self.view.body(row, work)
    }
}
impl super::sealed::Sealed for Reserved<'_, '_> {}
impl<'rows> Port<'rows, ReservationWork> for Reserved<'rows, '_> {
    fn rows(&self) -> s::ValidatedView<'rows> {
        self.rows
    }
    fn definition(
        &self,
        row: DeclarationRef,
        budget: &mut ReservationWork,
    ) -> Result<DefId, Error> {
        self.view.definition(row, budget)
    }
    fn root(&self) -> ImportedDeclarationId {
        self.view.root()
    }
    fn availability(&self) -> Availability {
        Availability::DeclaredSignatures
    }
    fn slot(
        &mut self,
        store: &mut Store,
        reference: s::SlotRef,
        budget: &mut ReservationWork,
    ) -> Result<Argument, Error> {
        budget.charge(1)?;
        let memo = self
            .maps
            .slots
            .get_mut(reference.0 as usize)
            .ok_or_else(|| error("reserved recipe has an unknown formal slot"))?;
        if let Some(argument) = *memo {
            return Ok(argument);
        }
        let slot = self
            .rows
            .slots
            .get(reference.0 as usize)
            .ok_or_else(|| error("reserved formal slot is outside its immutable schema"))?;
        let s::ParameterKind::Type { sized } = slot.kind else {
            return Err(error(
                "reserved type recipe requires its actual type formal",
            ));
        };
        let owner = self.view.definition(slot.owner, budget)?;
        if let Some(binding) = slot.definition {
            self.view.definition(binding, budget)?;
        }
        if let Some(address) = sized {
            if address.artifact != self.rows.policy.provider {
                return Err(error("reserved Sized formal belongs to another provider"));
            }
            let raw = self
                .rows
                .declarations
                .get(address.declaration.0 as usize)
                .ok_or_else(|| error("reserved Sized declaration is outside its actual table"))?;
            if raw.definition.kind != s::DefinitionKind::Trait {
                return Err(error(
                    "reserved Sized formal lost its actual trait declaration",
                ));
            }
            self.view.definition(address.declaration, budget)?;
        }
        let argument = slot_bindings::intern_type(store, owner, slot.index, sized.is_some())?;
        *memo = Some(argument);
        Ok(argument)
    }
    fn region(
        &mut self,
        reference: s::RegionRef,
        budget: &mut ReservationWork,
    ) -> Result<Demand<RegionId>, Error> {
        budget.charge(1)?;
        let memo = self
            .maps
            .regions
            .get_mut(reference.0 as usize)
            .ok_or_else(|| error("reserved recipe has an unknown declared region"))?;
        if let Some(region) = *memo {
            return Ok(Demand::Ready(region));
        }
        let region = match *self
            .rows
            .regions
            .get(reference.0 as usize)
            .ok_or_else(|| error("reserved region is outside its immutable schema"))?
        {
            s::Region::Static => RegionId::STATIC,
            s::Region::Bound {
                binder,
                depth,
                slot,
            } => {
                let frame = self
                    .rows
                    .binders
                    .get(binder.0 as usize)
                    .ok_or_else(|| error("reserved region lost its actual binder"))?;
                self.view.definition(frame.owner, budget)?;
                slot_bindings::bound(depth, slot, frame.count)?
            }
            s::Region::Variable { body, .. } => {
                return self
                    .view
                    .body(body, budget)
                    .map(|body| Demand::Blocked(Dependency::OwnedBodyDomain(body)));
            }
            s::Region::Parameter { slot, .. } => {
                let row = self
                    .rows
                    .slots
                    .get(slot.0 as usize)
                    .ok_or_else(|| error("reserved declared lifetime lost its actual slot"))?;
                return self
                    .view
                    .declaration(row.owner, budget)
                    .map(|owner| Demand::Blocked(Dependency::DeclaredLifetime(owner)));
            }
        };
        *memo = Some(region);
        Ok(Demand::Ready(region))
    }
    fn application(
        &self,
        reference: s::FunctionApplicationRef,
        budget: &mut ReservationWork,
    ) -> Result<Dependency, Error> {
        budget.charge(1)?;
        let row = self
            .rows
            .function_applications
            .get(reference.0 as usize)
            .ok_or_else(|| error("reserved application is outside its actual table"))?;
        if row.declaration.artifact != self.rows.policy.provider {
            return Err(error(
                "reserved callable dependency belongs to another provider",
            ));
        }
        self.view
            .declaration(row.declaration.declaration, budget)
            .map(Dependency::CallableTemplate)
    }
}
