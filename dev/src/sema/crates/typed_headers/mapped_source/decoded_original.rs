//! Sealed source data binds one actual original to the installed canonical graph.
use super::{Catalog, DefId, DefKind, Graph, s, slot_bindings};
use crate::{
    resolve::body_origin::{BodyOrigin, BodySite, ImportedBodyId, ProofSite},
    sema::model::Declarations,
    support::crates::{generics::TypeRef, owner_catalog::DeclarationRef},
    types::{
        RegionId, TypeId,
        arguments::Argument,
        parameters::{ParameterId, predicates::Predicate},
    },
};
use std::sync::Arc;

#[path = "callee_stability.rs"]
mod constant_policies;
#[path = "admitted_original.rs"]
mod selected;
use selected::Mapping;

/// Borrowed reconstruction inputs, not a checked body, permission or executable.
/// Only the actual current Graph/header/catalog correspondence can construct one.
pub(crate) struct Original<'a> {
    mapping: Mapping<'a>,
    catalog: &'a Catalog,
    header: &'a super::super::Header,
    definition: DefId,
    body: ImportedBodyId,
    rows: s::ValidatedView<'a>,
    reference: crate::support::crates::owner_catalog::BodyRef,
}
impl Graph {
    pub(in crate::sema::crates::typed_headers) fn owns_original(
        &self,
        catalog: &Catalog,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        charge(steps)?;
        let descriptor = self.descriptor(catalog)?;
        let declaration = catalog.declaration(definition)?;
        catalog.validate_declaration(declaration, definition, DefKind::Function)?;
        let (owner, _, _) = catalog.declaration_coordinate(declaration)?;
        Ok(owner == descriptor.identity())
    }
    pub(in crate::sema::crates::typed_headers) fn source_original<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &'a Declarations,
        headers: &'a super::super::Headers,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<Option<Original<'a>>, &'static str> {
        self.function_ready(catalog, facts, headers, definition, steps)?;
        let header = headers
            .get(definition)
            .ok_or("source original header disappeared")?;
        let Some(origin) = header.body else {
            return Ok(None);
        };
        let descriptor = self.descriptor(catalog)?;
        let declaration = catalog.declaration(definition)?;
        let (_, row, _) = catalog.declaration_coordinate(declaration)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
            return Err("source original lost its actual function/header association");
        };
        let function = &rows.functions[index as usize];
        let s::FunctionBody::Original(reference) = function.body else {
            return Err("source original cannot reconstruct an absent required method");
        };
        let body = catalog.body_at(descriptor.identity(), reference)?;
        let raw = rows
            .originals
            .get(reference.0 as usize)
            .filter(|raw| raw.body == reference && raw.declaration == row)
            .ok_or("source provided body omitted its actual complete original row")?;
        let inputs = rows
            .body_inputs
            .get(reference.0 as usize)
            .filter(|input| input.body == reference)
            .ok_or("source provided body omitted its actual raw input row")?;
        let constant = function.constness == crate::support::constness::Constness::Const;
        let context = if constant {
            s::CallContext::ConstDefinition
        } else {
            s::CallContext::Runtime
        };
        if catalog.origin(body)? != origin
            || inputs.frame != function.original.binder
            || header.constant != constant
            || (constant
                && (!descriptor.profile().unconditional_callables() || function.parent.is_some()))
            || (header.universals != 0 && !matches!(self.canonical, super::Canonical::Lazy(_)))
            || (function.parent.is_some() && !descriptor.profile().feature_predicates())
            || (header.signature != header.callable
                && !matches!(self.canonical, super::Canonical::Lazy(_)))
            || raw.context != context
        {
            return Err(
                "source original requires its complete supported ordinary signature/context",
            );
        }
        if let Some(builtins) = &inputs.builtins
            && builtins.environment != s::EnvironmentOrigin::Absent
            && builtins.environment != inputs.environment
        {
            return Err(
                "source original builtin environment changed its actual construction origin",
            );
        }
        let domain = self.ordinary_domain(facts, headers, definition, header, steps)?;
        let original = Original {
            mapping: Mapping::Ordinary {
                graph: self,
                store: &facts.types,
                domain,
            },
            catalog,
            header,
            definition,
            body,
            rows,
            reference,
        };
        original.check_locals(steps)?;
        Ok(Some(original))
    }
    fn ordinary_domain(
        &self,
        facts: &Declarations,
        headers: &super::super::Headers,
        definition: DefId,
        header: &super::super::Header,
        steps: &mut usize,
    ) -> Result<Option<super::super::unpaid_provided::OrdinaryDomain>, &'static str> {
        Ok(match &self.canonical {
            super::Canonical::Eager { .. } => None,
            super::Canonical::Lazy(lazy) => {
                headers
                    .charge(0)
                    .map_err(|_| "ordinary original has an exhausted header ledger")?;
                if headers.base.saturating_add(headers.bytes) > crate::support::crates::MAX_BYTES {
                    return Err("ordinary original has an exhausted header storage ledger");
                }
                let mut work = crate::resolve::crates::ReservationWork::default();
                let result = lazy.body(self, facts, definition, header, &mut work);
                *steps = steps.saturating_add(work.steps);
                if *steps > crate::support::crates::MAX_COMPONENTS {
                    return Err("ordinary original domain lookup work budget exceeded");
                }
                let domain = result
                    .map_err(|_| "ordinary original requires its actual current body domain")?;
                let environment = header
                    .environment
                    .as_ref()
                    .ok_or("ordinary original requires its actual pending defining environment")?;
                if domain
                    .environment()
                    .is_none_or(|actual| !Arc::ptr_eq(actual, environment))
                {
                    return Err(
                        "ordinary original changed its actual constructed defining environment",
                    );
                }
                Some(domain)
            }
        })
    }
}
impl Original<'_> {
    pub(crate) fn numbered(&self) -> bool {
        matches!(
            &self.mapping,
            Mapping::Selected { .. }
                | Mapping::Ordinary {
                    domain: Some(_),
                    ..
                }
        )
    }
    /// Defining checking mode only; current body proof and execution remain separate.
    pub(crate) const fn const_context(&self) -> bool {
        self.header.constant
    }
    pub(crate) const fn universals(&self) -> u32 {
        self.header.universals
    }
    pub(crate) fn rows(&self) -> s::ValidatedView<'_> {
        self.rows
    }
    pub(crate) fn raw(&self) -> &s::Original {
        &self.rows.originals[self.reference.0 as usize]
    }
    pub(crate) fn inputs(&self) -> &s::BodyInputs {
        &self.rows.body_inputs[self.reference.0 as usize]
    }
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.header.signature
    }
    pub(crate) fn source(&self) -> Result<BodyOrigin, &'static str> {
        self.catalog.origin(self.body)
    }
    pub(crate) fn environment(
        &self,
    ) -> Result<&Arc<crate::types::parameters::environment::Environment>, &'static str> {
        if let Mapping::Selected { body, .. } = &self.mapping {
            return Ok(body.environment());
        }
        self.header
            .environment
            .as_ref()
            .ok_or("source original requires its actual pending defining environment")
    }
    pub(crate) fn ty(&self, reference: TypeRef, steps: &mut usize) -> Result<TypeId, &'static str> {
        charge(steps)?;
        match &self.mapping {
            Mapping::Ordinary { graph, .. } => graph
                .ty(reference)
                .map_err(|_| "source original has an unknown actual type root"),
            Mapping::Selected { owner, store, .. } => owner.original_type(store, reference, steps),
        }
    }
    pub(crate) fn region(
        &self,
        reference: s::RegionRef,
        steps: &mut usize,
    ) -> Result<RegionId, &'static str> {
        charge(steps)?;
        match &self.mapping {
            Mapping::Ordinary {
                graph,
                store,
                domain,
            } => {
                if let Some(domain) = domain {
                    let mut work = crate::resolve::crates::ReservationWork::default();
                    let result = domain.region(&graph.recipe_view(), store, reference, &mut work);
                    *steps = steps.saturating_add(work.steps);
                    if *steps > crate::support::crates::MAX_COMPONENTS {
                        return Err("ordinary original regional query work budget exceeded");
                    }
                    if let Some(region) = result
                        .map_err(|_| "ordinary original changed its actual recorded region")?
                    {
                        return Ok(region);
                    }
                }
                graph
                    .region(reference)
                    .map_err(|_| "source original has an unknown actual region root")
            }
            Mapping::Selected {
                owner,
                store,
                body,
                view,
            } => {
                let mut work = crate::resolve::crates::ReservationWork::default();
                let regional = body.region(view, store, reference, &mut work);
                *steps = steps.saturating_add(work.steps);
                match regional.map_err(|_| "source original changed its actual recorded region")? {
                    Some(region) => Ok(region),
                    None => owner.original_region(store, reference, steps),
                }
            }
        }
    }
    pub(crate) fn variable(&self, slot: u32, steps: &mut usize) -> Result<RegionId, &'static str> {
        charge(steps)?;
        let mut work = crate::resolve::crates::ReservationWork::default();
        let result = match &self.mapping {
            Mapping::Ordinary {
                graph,
                store,
                domain: Some(domain),
            } => domain.variable(&graph.recipe_view(), store, slot, &mut work),
            Mapping::Selected {
                body, view, store, ..
            } => body.variable(view, store, slot, &mut work),
            Mapping::Ordinary { domain: None, .. } => {
                return Err("source variable requires its actual recorded body domain");
            }
        };
        *steps = steps.saturating_add(work.steps);
        if *steps > crate::support::crates::MAX_COMPONENTS {
            return Err("source variable mapping work budget exceeded");
        }
        result.map_err(|_| "source variable changed its actual body/allocation owner")
    }
    pub(crate) fn predicate(
        &self,
        raw: &s::Requirement,
        steps: &mut usize,
    ) -> Result<Predicate, &'static str> {
        charge(steps)?;
        match &self.mapping {
            Mapping::Ordinary {
                graph,
                domain: None,
                ..
            } => graph.requirement(raw),
            Mapping::Ordinary {
                domain: Some(_), ..
            }
            | Mapping::Selected { .. } => super::original_requests::map(
                &mut Requests {
                    original: self,
                    steps,
                },
                raw,
            ),
        }
    }
    pub(crate) fn slot_type(
        &self,
        reference: s::SlotRef,
        steps: &mut usize,
    ) -> Result<TypeId, &'static str> {
        charge(steps)?;
        let Argument::Type(ty) = self.slot(reference)? else {
            return Err("source original contract parameter is not an actual type formal");
        };
        Ok(ty)
    }
    pub(crate) fn parameter(
        &self,
        reference: s::SlotRef,
        store: &crate::types::Store,
        steps: &mut usize,
    ) -> Result<ParameterId, &'static str> {
        charge(steps)?;
        self.require_store(store)?;
        if let Mapping::Selected { body, view, .. } = &self.mapping {
            let mut work = crate::resolve::crates::ReservationWork::default();
            let actual = body.parameter(view, reference, &mut work);
            *steps = steps.saturating_add(work.steps);
            let parameter =
                actual.map_err(|_| "selected original changed its exact full formal scope")?;
            let Argument::Type(ty) = self.slot(reference)? else {
                return Err("selected original formal is not an actual type argument");
            };
            if store.kind(ty) != Some(crate::types::Kind::Parameter(parameter)) {
                return Err("selected original changed its actual scope/map formal identity");
            }
            return Ok(parameter);
        }
        let Argument::Type(ty) = self.slot(reference)? else {
            return Err("source original contract parameter is not an actual type formal");
        };
        let Some(crate::types::Kind::Parameter(id)) = store.kind(ty) else {
            return Err("source original contract parameter lost its rigid identity");
        };
        Ok(id)
    }
    pub(crate) fn declaration(
        &self,
        reference: DeclarationRef,
        steps: &mut usize,
    ) -> Result<DefId, &'static str> {
        charge(steps)?;
        let descriptor = self.descriptor()?;
        let rows = descriptor.rows();
        let raw = rows
            .declarations
            .get(reference.0 as usize)
            .ok_or("source original declaration is outside its actual provider")?;
        let definition = self
            .catalog
            .definition_at(descriptor.identity(), reference)?;
        let id = self.catalog.declaration(definition)?;
        self.catalog.validate_declaration(
            id,
            definition,
            slot_bindings::kind(raw.definition.kind),
        )?;
        Ok(definition)
    }
    pub(crate) fn function(
        &self,
        reference: s::FunctionApplicationRef,
        steps: &mut usize,
    ) -> Result<DefId, &'static str> {
        charge(steps)?;
        let row = self
            .rows
            .function_applications
            .get(reference.0 as usize)
            .ok_or("source original has an unknown actual callable application")?;
        if !self.rows.descriptor().profile().feature_predicates()
            || row.declaration.artifact != s::ArtifactRef(0)
        {
            return Err("source call requires its actual v4 defining artifact");
        }
        let definition = self.declaration(row.declaration.declaration, steps)?;
        let declaration = self.catalog.declaration(definition)?;
        self.catalog
            .validate_declaration(declaration, definition, DefKind::Function)?;
        Ok(definition)
    }
    pub(crate) fn site(&self, index: u32, steps: &mut usize) -> Result<BodySite, &'static str> {
        charge(steps)?;
        self.catalog.site(self.body, index)
    }
    pub(crate) fn input_site(
        &self,
        site: s::InputSite,
        steps: &mut usize,
    ) -> Result<ProofSite, &'static str> {
        charge(steps)?;
        match site {
            s::InputSite::Body(site) => self
                .catalog
                .proof_site(self.body, self.catalog.site(self.body, site)?),
            s::InputSite::Declaration { owner, site } => {
                let definition = self.declaration(owner, steps)?;
                let declaration = self.catalog.declaration(definition)?;
                let site = self.catalog.declaration_site(declaration, site)?;
                self.catalog.declaration_proof_site(declaration, site)
            }
        }
    }
    /// An unpaid goal for the actual authenticated Sized declaration, not a flag.
    pub(crate) fn sizing_requirement(
        &self,
        facts: &Declarations,
        steps: &mut usize,
    ) -> Result<crate::types::parameters::predicates::Bound, &'static str> {
        use crate::{support::bindings::LangItem, types::parameters::predicates::Bound};
        charge(steps)?;
        self.require_facts(facts)?;
        let role = facts
            .source_bindings
            .as_deref()
            .ok_or("source original sizing lacks its actual role catalog")?
            .validated_role(LangItem::Sized)?
            .ok_or("source original sizing lacks its actual Sized declaration")?;
        self.role_origin(role)?;
        facts.associated.require_imported_role(role)?;
        let predicate = facts
            .generics
            .trait_self_bounds
            .get(&role.definition)
            .ok_or("source original sizing omitted its complete actual trait schema")?;
        let Bound::Trait(id) = predicate.bound else {
            return Err("source original sizing changed its actual ordinary trait goal");
        };
        let actual = facts
            .types
            .trait_application(id)
            .ok_or("source original sizing lost its actual full application")?;
        if actual.item.0 != role.definition.0
            || facts
                .types
                .arguments(actual.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err("source original sizing belongs to a different trait/application");
        }
        Ok(predicate.bound)
    }
    pub(crate) fn copy_identity(
        &self,
        facts: &Declarations,
        claimed: Option<s::DeclarationAddress>,
        steps: &mut usize,
    ) -> Result<Option<DefId>, &'static str> {
        charge(steps)?;
        self.require_facts(facts)?;
        let role = facts
            .source_bindings
            .as_deref()
            .ok_or("source original Copy query lacks its actual role catalog")?
            .validated_role(crate::support::bindings::LangItem::Copy)?;
        if let Some(role) = role {
            self.role_origin(role)?;
            facts.associated.require_imported_role(role)?;
        }
        let actual = role.map(|role| role.definition);
        let claimed = claimed
            .map(|address| {
                if address.artifact != s::ArtifactRef(0) {
                    return Err("source original Copy query belongs to a foreign artifact");
                }
                self.declaration(address.declaration, steps)
            })
            .transpose()?;
        if claimed != actual {
            return Err("source original Copy query changed its actual language declaration");
        }
        Ok(actual)
    }
    fn require_facts(&self, facts: &Declarations) -> Result<(), &'static str> {
        self.require_store(&facts.types)?;
        if facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.origin_catalog())
            .is_none_or(|actual| !std::ptr::eq(actual, self.catalog))
        {
            return Err("source original query belongs to another actual semantic catalog");
        }
        Ok(())
    }
    fn check_locals(&self, steps: &mut usize) -> Result<(), &'static str> {
        let descriptor = self.descriptor()?;
        let rows = descriptor.rows();
        for (index, local) in self.raw().locals.iter().enumerate() {
            charge(steps)?;
            let Some(reference) = local.definition else {
                continue;
            };
            let raw = rows
                .declarations
                .get(reference.0 as usize)
                .ok_or("source local definition is outside its actual declaration table")?;
            let actual = raw
                .local
                .ok_or("source local definition omitted its real body association")?;
            if actual.body != self.raw().body
                || actual.ordinal as usize != index
                || actual.scope != local.scope
                || raw.definition.parent != Some(self.raw().declaration)
                || !matches!(
                    raw.definition.kind,
                    s::DefinitionKind::ValueParameter | s::DefinitionKind::Local
                )
            {
                return Err("source local definition changed its actual original/ordinal/scope");
            }
            let definition = self.declaration(reference, steps)?;
            let id = self.catalog.declaration(definition)?;
            let kind = if matches!(
                local.kind,
                crate::support::crates::generics::LocalKind::Parameter(_)
            ) {
                DefKind::Parameter
            } else {
                DefKind::Local
            };
            self.catalog.validate_declaration(id, definition, kind)?;
        }
        Ok(())
    }
}
fn charge(steps: &mut usize) -> Result<(), &'static str> {
    *steps = steps.saturating_add(1);
    if *steps > crate::support::crates::MAX_COMPONENTS {
        return Err("source original query work budget exceeded");
    }
    Ok(())
}

struct Requests<'view, 'rows, 'steps> {
    original: &'view Original<'rows>,
    steps: &'steps mut usize,
}
impl super::original_requests::Requests for Requests<'_, '_, '_> {
    fn application(
        &mut self,
        reference: s::TraitApplicationRef,
    ) -> Result<Predicate, &'static str> {
        match &self.original.mapping {
            Mapping::Ordinary { graph, .. } => graph.application(reference),
            Mapping::Selected { owner, store, .. } => {
                owner.original_application(store, reference, self.steps)
            }
        }
    }
    fn ty(&mut self, reference: TypeRef) -> Result<TypeId, &'static str> {
        self.original.ty(reference, self.steps)
    }
    fn region(&mut self, reference: s::RegionRef) -> Result<RegionId, &'static str> {
        self.original.region(reference, self.steps)
    }
}
