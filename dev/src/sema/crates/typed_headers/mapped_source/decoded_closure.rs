//! Authenticate the installed closure declaration and recorded domain of a closure original.
use super::{Catalog, Declarations, Graph, Mapping, Original, OriginalHeader, charge, s};
use crate::{
    resolve::crates::ReservationWork,
    sema::crates::typed_headers::{Headers, unpaid_provided::ClosureHeader},
    support::crates::owner_catalog::BodyRef,
    types::closure::ClosureId,
};

impl Graph {
    pub(in crate::sema::crates::typed_headers) fn owns_closure_original(
        &self,
        catalog: &Catalog,
        identity: ClosureId,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        charge(steps)?;
        let descriptor = self.descriptor(catalog)?;
        if !descriptor.profile().declaration_policies() {
            return Ok(false);
        }
        let Some(reference) = self.closure_ref(identity) else {
            return Ok(false);
        };
        let rows = descriptor.rows();
        let Some(closures) = rows.closures() else {
            return Ok(false);
        };
        Ok((reference.0 as usize) < closures.headers.len())
    }

    pub(in crate::sema::crates::typed_headers) fn closure_identity_for_body(
        &self,
        catalog: &Catalog,
        body: BodyRef,
        steps: &mut usize,
    ) -> Result<Option<ClosureId>, &'static str> {
        charge(steps)?;
        let descriptor = self.descriptor(catalog)?;
        let rows = descriptor.rows();
        let Some(closures) = rows.closures() else {
            return Ok(None);
        };
        let Some((reference, _)) = closures.header_for_body(body) else {
            return Ok(None);
        };
        self.closure_id(reference).map(Some)
    }

    pub(in crate::sema::crates::typed_headers) fn closure_header(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        identity: ClosureId,
        steps: &mut usize,
    ) -> Result<Option<ClosureHeader>, &'static str> {
        charge(steps)?;
        let descriptor = self.descriptor(catalog)?;
        if !descriptor.profile().declaration_policies() {
            return Ok(None);
        }
        let Some(closure_ref) = self.closure_ref(identity) else {
            return Ok(None);
        };
        let rows = descriptor.rows();
        let Some(closures) = rows.closures() else {
            return Ok(None);
        };
        let Some(header) = closures.header(closure_ref) else {
            return Ok(None);
        };
        let owner = catalog.definition_at(descriptor.identity(), header.owner)?;
        let body_owner = rows
            .body_owners()
            .get(header.body.0 as usize)
            .ok_or("source closure lost its actual body owner")?;
        let parent_raw = body_owner
            .parent
            .ok_or("source closure omitted its actual parent body")?;
        let raw = header.body;
        let index = rows.original_index(raw)?;
        let original = rows.original(raw)?;
        let input = rows.body_input(raw)?;
        let unit = rows
            .body_units
            .get(index)
            .filter(|unit| unit.body == raw)
            .ok_or("source closure lost its actual numbering unit")?;
        let expected_context = if header.constant {
            s::CallContext::ConstDefinition
        } else {
            s::CallContext::Runtime
        };
        let body = catalog.body_at(descriptor.identity(), raw)?;
        let signature = self
            .ty(header.signature)
            .map_err(|_| "source closure lost its canonical public signature")?;
        let execution = header
            .execution
            .map(|exec| self.ty(exec.signature))
            .transpose()
            .map_err(|_| "source closure lost its canonical execution signature")?;
        let installed = facts
            .closures
            .identity(identity)
            .ok_or("source closure lost its installed semantic declaration")?;
        if original.declaration != header.owner
            || original.context != expected_context
            || input.frame != header.binder
            || unit.universals != header.contract_universals()
            || installed.signature != signature
            || installed.execution.map(|exec| exec.signature) != execution
            || installed.owner != Some(owner)
            || facts.closures.origin(identity) != Some(catalog.origin(body)?)
            || input.builtins.as_ref().is_some_and(|builtins| {
                builtins.environment != s::EnvironmentOrigin::Absent
                    && builtins.environment != input.environment
            })
        {
            return Err("source closure changed its actual declaration, signature or context");
        }
        Ok(Some(ClosureHeader {
            identity,
            closure_ref,
            owner,
            owner_row: header.owner,
            raw,
            parent_raw,
            body,
            signature,
            execution,
            binder: header.binder,
            universals: unit.universals,
            constant: header.constant,
            require_result_formation: header.require_result_formation,
        }))
    }

    pub(in crate::sema::crates::typed_headers) fn source_closure_original<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &'a Declarations,
        headers: &'a Headers,
        identity: ClosureId,
        steps: &mut usize,
    ) -> Result<Option<Original<'a>>, &'static str> {
        let Some(closure) = self.closure_header(catalog, facts, identity, steps)? else {
            return Ok(None);
        };
        self.source_closure(catalog, facts, headers, closure, steps)
            .map(Some)
    }

    fn source_closure<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &'a Declarations,
        headers: &'a Headers,
        closure: ClosureHeader,
        steps: &mut usize,
    ) -> Result<Original<'a>, &'static str> {
        let super::super::Canonical::Lazy(lazy) = &self.canonical else {
            return Err("source closure requires its actual lazy declaration owner");
        };
        headers
            .charge(0)
            .map_err(|_| "source closure has an exhausted header ledger")?;
        if headers.base.saturating_add(headers.bytes) > crate::support::crates::MAX_BYTES {
            return Err("source closure has an exhausted header storage ledger");
        }
        let mut work = ReservationWork::default();
        let domains = lazy.closure_body(self, facts, closure, &mut work);
        *steps = steps.saturating_add(work.steps);
        if *steps > crate::support::crates::MAX_COMPONENTS {
            return Err("source closure domain lookup work budget exceeded");
        }
        let (domain, parent_domain) =
            domains.map_err(|_| "source closure requires its actual current body domain")?;
        if domain.environment().is_none() {
            return Err("source closure omitted its actual defining environment");
        }
        let rows = self.descriptor(catalog)?.rows();
        let original = Original {
            mapping: Mapping::Ordinary {
                graph: self,
                store: &facts.types,
                domain: Some(domain),
                parent_domain,
            },
            members: &facts.members,
            catalog,
            header: OriginalHeader::Closure(closure),
            definition: closure.owner,
            body: closure.body,
            rows,
            row: u32::try_from(rows.original_index(closure.raw)?)
                .map_err(|_| "source closure row overflow")?,
        };
        original.check_locals(steps)?;
        Ok(original)
    }
}
