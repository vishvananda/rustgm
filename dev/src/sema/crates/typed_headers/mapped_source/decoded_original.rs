//! Sealed source data binds one actual original to the installed canonical graph.
use super::{Catalog, DefId, DefKind, Graph, s, slot_bindings};
// A const inherent method's original is a const definition, as a free one is.
use super::super::staged_headers::scheme_ports::{const_impl_body, inherent_captures as inherent};
use crate::{
    resolve::body_origin::{BodyOrigin, BodySite, ImportedBodyId, ProofSite},
    sema::model::Declarations,
    support::crates::{
        generics::{BinderRef, TypeRef},
        owner_catalog::DeclarationRef,
    },
    types::{
        RegionId, TypeId,
        arguments::Argument,
        parameters::{ParameterId, predicates::Predicate},
    },
};
use std::sync::Arc;

#[path = "decoded_closure.rs"]
mod closure;
#[path = "callee_stability.rs"]
mod constant_policies;
#[path = "decoded_initializer.rs"]
mod initializer;
#[path = "mapped_members.rs"]
mod nominal_members;
#[path = "admitted_original.rs"]
mod selected;
use selected::Mapping;

/// Borrowed reconstruction inputs, not a checked body, permission or executable.
/// Only the actual current Graph/header/catalog correspondence can construct one.
pub(crate) struct Original<'a> {
    mapping: Mapping<'a>,
    members: &'a crate::sema::members::Members,
    catalog: &'a Catalog,
    header: OriginalHeader<'a>,
    definition: DefId,
    body: ImportedBodyId,
    rows: s::ValidatedView<'a>,
    row: u32,
}
enum OriginalHeader<'a> {
    Function(&'a super::super::Header),
    Initializer(super::super::unpaid_provided::InitializerHeader),
    Closure(super::super::unpaid_provided::ClosureHeader),
    Anonymous(super::super::unpaid_provided::AnonymousHeader),
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
        let (owner, row, _) = catalog.declaration_coordinate(declaration)?;
        if owner != descriptor.identity() {
            return Ok(false);
        }
        let rows = descriptor.rows();
        if let Some(s::HeaderRef::Constant(index)) = rows.header(row)? {
            catalog.validate_declaration(declaration, definition, DefKind::Const)?;
            Ok(matches!(
                rows.constants()[index as usize].body,
                s::ConstantBody::Original(_)
            ))
        } else {
            catalog.validate_declaration(declaration, definition, DefKind::Function)?;
            Ok(true)
        }
    }
    pub(in crate::sema::crates::typed_headers) fn source_original<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &'a Declarations,
        headers: &'a super::super::Headers,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<Option<Original<'a>>, &'static str> {
        if headers.get(definition).is_none()
            && let Some(initializer) = self.initializer_header(catalog, facts, definition, steps)?
        {
            return self
                .source_initializer(catalog, facts, headers, initializer, steps)
                .map(Some);
        }
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
            return Err(
                "compiler-operation execution requires genuine future state ownership; required methods have no original",
            );
        };
        let original_index = rows.original_index(reference)?;
        let body = catalog.body_at(descriptor.identity(), reference)?;
        let raw = rows
            .originals
            .get(original_index)
            .filter(|raw| raw.body == reference && raw.declaration == row)
            .ok_or("source provided body omitted its actual complete original row")?;
        let inputs = rows.body_input(reference)?;
        let constant = function.constness == crate::support::constness::Constness::Const;
        let default = matches!(inputs.environment,
            s::EnvironmentOrigin::ConstTraitDefault { member, trait_ }
                if member == row && trait_.artifact == rows.policy.provider
                    && Some(trait_.declaration) == function.parent);
        let context = if !(constant || default)
            || raw.context == s::CallContext::Runtime
                && descriptor.profile().declaration_policies()
                && (rows.is_unchecked_const_body(row)
                    || crate::sema::compiler_items::runtime_format_entry(facts, definition)?)
        {
            s::CallContext::Runtime
        } else if descriptor.profile().declaration_policies() && rows.is_comptime(row) {
            s::CallContext::ClosedConst
        } else {
            s::CallContext::ConstDefinition
        };
        if catalog.origin(body)? != origin
            || inputs.frame != function.original.binder
            || header.constant != constant
            || (constant
                && (!descriptor.profile().unconditional_callables()
                    || function.parent.is_some()
                        && !inherent(descriptor, function)
                        && !const_impl_body(descriptor, function)))
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
            members: &facts.members,
            mapping: Mapping::Ordinary {
                graph: self,
                store: &facts.types,
                domain,
                parent_domain: None,
            },
            catalog,
            header: OriginalHeader::Function(header),
            definition,
            body,
            rows,
            row: u32::try_from(original_index).map_err(|_| "source original row overflow")?,
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
    pub(crate) fn const_context(&self) -> bool {
        self.raw().context != s::CallContext::Runtime
    }
    pub(crate) fn unchecked_const_body(&self) -> bool {
        self.rows.is_unchecked_const_body(self.raw().declaration)
    }
    pub(crate) const fn universals(&self) -> u32 {
        match self.header {
            OriginalHeader::Function(header) => header.universals,
            OriginalHeader::Initializer(header) => header.universals,
            OriginalHeader::Closure(header) => header.universals,
            OriginalHeader::Anonymous(header) => header.universals,
        }
    }
    pub(crate) fn rows(&self) -> s::ValidatedView<'_> {
        self.rows
    }
    pub(crate) fn raw(&self) -> &s::Original {
        &self.rows.originals[self.row as usize]
    }
    pub(crate) fn inputs(&self) -> &s::BodyInputs {
        &self.rows.body_inputs[self.row as usize]
    }
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> Option<TypeId> {
        match self.header {
            OriginalHeader::Function(header) => Some(header.signature),
            OriginalHeader::Initializer(_) | OriginalHeader::Anonymous(_) => None,
            OriginalHeader::Closure(header) => match header.execution {
                Some(execution) => Some(execution),
                None => Some(header.signature),
            },
        }
    }
    pub(crate) const fn closure_owner(
        &self,
    ) -> Option<(crate::types::closure::ClosureId, TypeId, Option<TypeId>)> {
        match self.header {
            OriginalHeader::Closure(header) => {
                Some((header.identity, header.signature, header.execution))
            }
            OriginalHeader::Function(_)
            | OriginalHeader::Initializer(_)
            | OriginalHeader::Anonymous(_) => None,
        }
    }
    pub(crate) const fn is_anonymous(&self) -> bool {
        matches!(self.header, OriginalHeader::Anonymous(_))
    }
    pub(crate) const fn require_result_formation(&self) -> bool {
        match self.header {
            OriginalHeader::Closure(header) => header.require_result_formation,
            OriginalHeader::Function(_)
            | OriginalHeader::Initializer(_)
            | OriginalHeader::Anonymous(_) => false,
        }
    }
    pub(crate) fn source(&self) -> Result<BodyOrigin, &'static str> {
        self.catalog.origin(self.body)
    }
    pub(crate) fn anonymous_body(
        &self,
        body: crate::support::crates::owner_catalog::BodyRef,
        steps: &mut usize,
    ) -> Result<BodyOrigin, &'static str> {
        charge(steps)?;
        if !self.rows.descriptor().profile().declaration_policies()
            || self
                .rows
                .closures()
                .and_then(|closures| closures.anonymous_for_body(body))
                .is_none()
        {
            return Err("source inline constant names an unreserved anonymous body");
        }
        let id = self.catalog.body_at(self.descriptor()?.identity(), body)?;
        self.catalog.origin(id)
    }
    pub(crate) fn environment(
        &self,
    ) -> Result<&Arc<crate::types::parameters::environment::Environment>, &'static str> {
        let actual = match (&self.mapping, &self.header) {
            (Mapping::Selected { body, .. }, _) => Some(body.environment()),
            (
                Mapping::Ordinary { domain, .. },
                OriginalHeader::Initializer(_)
                | OriginalHeader::Closure(_)
                | OriginalHeader::Anonymous(_),
            ) => domain.as_ref().and_then(|domain| domain.environment()),
            (_, OriginalHeader::Function(header)) => header.environment.as_ref(),
        };
        actual.ok_or("source original requires its actual pending defining environment")
    }
    pub(crate) fn nested_output(
        &self,
        steps: &mut usize,
    ) -> Result<Option<Arc<crate::sema::nested_outputs::Link>>, &'static str> {
        let (parent_raw, output, bridge_slice, closure_identity) = match self.header {
            OriginalHeader::Closure(closure) => {
                let table = self
                    .rows
                    .closures()
                    .ok_or("source closure output lost its columns")?;
                let raw = table
                    .header(closure.closure_ref)
                    .ok_or("source closure output lost its header")?;
                let Some(output) = raw.output else {
                    return Ok(None);
                };
                (
                    closure.parent_raw,
                    output,
                    output.bridges.slice(&table.bridges)?,
                    Some((closure.identity, table)),
                )
            }
            OriginalHeader::Anonymous(anonymous) => {
                let table = self
                    .rows
                    .closures()
                    .ok_or("source anonymous output lost its columns")?;
                let (_, raw) = table
                    .anonymous_for_body(anonymous.raw)
                    .ok_or("source anonymous output lost its header")?;
                let Some(output) = raw.output else {
                    return Ok(None);
                };
                (
                    anonymous.parent_raw,
                    output,
                    output.bridges.slice(&table.anonymous_bridges)?,
                    None,
                )
            }
            OriginalHeader::Function(_) | OriginalHeader::Initializer(_) => return Ok(None),
        };
        let parent_body = self
            .catalog
            .body_at(self.descriptor()?.identity(), parent_raw)?;
        let parent = self.catalog.origin(parent_body)?;
        let mut bridges = Vec::with_capacity(bridge_slice.len());
        for &bridge in bridge_slice {
            charge(steps)?;
            bridges.push(crate::sema::nested_outputs::Bridge {
                slot: bridge.slot,
                parent: self.parent_region(bridge.parent, steps)?,
                output: bridge.output,
            });
        }
        let formation = if let (Some(raw_formation), Some((identity, table))) =
            (output.formation, closure_identity)
        {
            let raw_inputs = raw_formation.inputs.slice(&table.formation_inputs)?;
            let mut inputs = Vec::with_capacity(raw_inputs.len());
            for &input in raw_inputs {
                charge(steps)?;
                inputs.push(crate::sema::closures::FormationInput {
                    root: crate::sema::closures::captures::PathId::from_raw(input.root),
                    child: self.ty(input.child, steps)?,
                    parent: self.ty(input.parent, steps)?,
                });
            }
            let raw_slots = raw_formation.callable_slots.slice(&table.callable_slots)?;
            charge(steps)?;
            Some(crate::sema::nested_outputs::Formation {
                identity,
                inputs: inputs.into_boxed_slice(),
                inherited: raw_formation.inherited,
                callable_slots: raw_slots.to_vec().into_boxed_slice(),
            })
        } else {
            None
        };
        Ok(Some(Arc::new(crate::sema::nested_outputs::Link {
            source: self.source()?,
            parent,
            revision: output.revision,
            universals: output.universals,
            bridges: bridges.into_boxed_slice(),
            formation,
            creator_only_from: output.creator_only_from,
        })))
    }
    fn parent_region(
        &self,
        reference: s::RegionRef,
        steps: &mut usize,
    ) -> Result<RegionId, &'static str> {
        charge(steps)?;
        let Mapping::Ordinary {
            graph,
            store,
            parent_domain,
            ..
        } = &self.mapping
        else {
            return Err("source closure parent region requires an ordinary graph");
        };
        if let Some(parent) = parent_domain {
            let mut work = crate::resolve::crates::ReservationWork::default();
            let result = parent.region(&graph.recipe_view(), store, reference, &mut work);
            *steps = steps.saturating_add(work.steps);
            if *steps > crate::support::crates::MAX_COMPONENTS {
                return Err("ordinary closure parent region query work budget exceeded");
            }
            if let Some(region) =
                result.map_err(|_| "ordinary closure changed its actual recorded parent region")?
            {
                return Ok(region);
            }
        }
        graph
            .region(reference)
            .map_err(|_| "source closure has an unknown actual parent region root")
    }
    pub(crate) fn normalization_scope(
        &self,
        scoped: BinderRef,
        steps: &mut usize,
    ) -> Result<u32, &'static str> {
        charge(steps)?;
        self.rows
            .binders
            .get(scoped.0 as usize)
            .map(|scope| scope.count)
            .ok_or("selected normalization lacks its actual scope")
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
                ..
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
                ..
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
    /// A formal's rigid type, or `None` for a version-thirteen const formal,
    /// which has no type to form or size.
    pub(crate) fn slot_type(
        &self,
        reference: s::SlotRef,
        steps: &mut usize,
    ) -> Result<Option<TypeId>, &'static str> {
        charge(steps)?;
        match self.slot(reference)? {
            Argument::Type(ty) => Ok(Some(ty)),
            Argument::Const(_) => Ok(None),
            Argument::Lifetime(_) => {
                Err("source original contract parameter is not an actual type formal")
            }
        }
    }
    pub(crate) fn const_argument(
        &self,
        reference: s::SlotRef,
        steps: &mut usize,
    ) -> Result<crate::types::arguments::ConstId, &'static str> {
        charge(steps)?;
        match self.slot(reference)? {
            Argument::Const(term) => Ok(term),
            _ => Err("source const argument lost its actual canonical const slot"),
        }
    }
    pub(crate) fn membership_captures(
        &self,
        facts: &Declarations,
        captures: crate::support::crates::generics::Range,
        steps: &mut usize,
    ) -> Result<crate::types::GenericArgs, &'static str> {
        charge(steps)?;
        self.require_facts(facts)?;
        let raw = captures.slice(&self.rows.arguments)?;
        let ids = facts
            .opaques
            .owners
            .get(&self.definition)
            .ok_or("source membership obligation lacks an owned opaque declaration")?;
        for &id in ids {
            charge(steps)?;
            let Some(declaration) = facts.opaques.declarations.get(&id) else {
                continue;
            };
            let Some(crate::types::Kind::Opaque(_, candidate)) =
                facts.types.kind(declaration.application)
            else {
                continue;
            };
            let Some(actual) = facts.types.arguments(candidate) else {
                continue;
            };
            if actual.len() != raw.len() {
                continue;
            }
            let mut matched = true;
            for (expected, &wire) in actual.zip(raw) {
                charge(steps)?;
                let ok = match (expected, wire) {
                    (Argument::Lifetime(a), s::Argument::Region(b)) => {
                        a == self.region(b, steps)?
                    }
                    (Argument::Type(a), s::Argument::Type(b)) => a == self.ty(b, steps)?,
                    (Argument::Const(a), s::Argument::Const(b)) => {
                        match self.rows.consts().get(b.0 as usize) {
                            Some(&s::ConstTerm::Parameter(slot)) => {
                                a == self.const_argument(slot, steps)?
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                };
                if !ok {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Ok(candidate);
            }
        }
        Err("source membership captures do not match any owned opaque declaration")
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
    /// The record's type formals in complete order. A version-thirteen const
    /// formal shares that order but has no region or sizing, so a rebuilt
    /// contract omits it once its rigid identity is checked.
    pub(crate) fn type_parameters(
        &self,
        store: &crate::types::Store,
        steps: &mut usize,
    ) -> Result<Vec<ParameterId>, &'static str> {
        let record = &self.inputs().parameters;
        let mut parameters = Vec::with_capacity(record.len());
        for &reference in record {
            charge(steps)?;
            if let Argument::Const(term) = self.slot(reference)? {
                if !matches!(
                    store.const_kind(term),
                    Some(crate::types::arguments::ConstKind::Parameter(_))
                ) {
                    return Err("source original const formal lost its rigid identity");
                }
                continue;
            }
            parameters.push(self.parameter(reference, store, steps)?);
        }
        Ok(parameters)
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
        if row.declaration.artifact.0 != 0 {
            let (descriptor, _) = self.dependency_function(row.declaration)?;
            let definition = self
                .catalog
                .definition_at(descriptor.identity(), row.declaration.declaration)?;
            let declaration = self.catalog.declaration(definition)?;
            self.catalog
                .validate_declaration(declaration, definition, DefKind::Function)?;
            return Ok(definition);
        }
        if !self.rows.descriptor().profile().feature_predicates() {
            return Err("source call requires its actual v4 defining artifact");
        }
        let definition = self.declaration(row.declaration.declaration, steps)?;
        let declaration = self.catalog.declaration(definition)?;
        // Version thirteen: a called tuple-variant constructor is its variant.
        let body_nodes = self.rows.descriptor().profile().body_nodes();
        if body_nodes {
            charge(steps)?;
        }
        let kind = if body_nodes
            && s::Constructor::of(
                (self.rows.enums(), &self.rows.declarations),
                row.declaration.declaration,
                |row| self.rows.header(row),
            )?
            .is_some()
        {
            DefKind::Variant
        } else {
            DefKind::Function
        };
        self.catalog
            .validate_declaration(declaration, definition, kind)?;
        Ok(definition)
    }
    pub(crate) fn dependency_function(
        &self,
        address: s::DeclarationAddress,
    ) -> Result<
        (
            crate::support::crates::SourceDescriptor<'_>,
            s::FunctionHeader,
        ),
        &'static str,
    > {
        let Mapping::Ordinary { graph, .. } = &self.mapping else {
            return Err("selected original requires a local definition");
        };
        let descriptor = graph.dependency_descriptor(address.artifact)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(address.declaration)? else {
            return Err("source dependency application has no actual callable header");
        };
        Ok((descriptor, rows.functions[index as usize]))
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
        if self.rows().descriptor().profile().bootstrap_bindings() {
            return Ok(Bound::Sized);
        }
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
        if self.rows().descriptor().profile().bootstrap_bindings() {
            return if claimed.is_none() {
                Ok(None)
            } else {
                Err("bootstrap Copy query cannot name a source role")
            };
        }
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
            let kind = match (local.kind, raw.definition.kind) {
                (
                    crate::support::crates::generics::LocalKind::Parameter(_),
                    s::DefinitionKind::ValueParameter,
                ) => DefKind::Parameter,
                (
                    crate::support::crates::generics::LocalKind::Binding,
                    s::DefinitionKind::ValueParameter,
                ) if descriptor.profile().declaration_policies() => DefKind::Parameter,
                (
                    crate::support::crates::generics::LocalKind::Binding,
                    s::DefinitionKind::Local,
                ) => DefKind::Local,
                _ => {
                    return Err(
                        "source local definition changed its actual original/ordinal/scope",
                    );
                }
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
