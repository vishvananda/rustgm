//! A shared row-domain view still cannot create a descriptor or paid body.
#[path = "domain_index/operation_domains.rs"]
mod body_ports;
#[path = "domain_index/authored_inputs.rs"]
mod input_ports;
#[path = "domain_index/root_ports.rs"]
mod root_ports;
use super::{
    HeaderIndex, HeaderRef, Work,
    binder_domains::BinderIndex,
    type_domains::{Context, Types},
};
use crate::support::crates::generics::{self as g, source as s};

#[derive(Clone, Copy, Debug, Default)]
pub struct DomainWork {
    pub steps: usize,
    pub roots: usize,
    pub binder_frames: usize,
    pub binder_queries: usize,
    pub binder_depth: usize,
    pub type_nodes: usize,
    pub type_cache_hits: usize,
    pub operation_ports: usize,
}
/// This borrowed data view preserves exact ownership and coordinate domains;
/// actual types, complete operation-derived goals and ordinary proof remain
/// mandatory on import. It has no conversion to `ValidatedRows` or a descriptor.
pub struct DomainIndex<'index, 'rows> {
    headers: &'index HeaderIndex<'rows>,
    binders: BinderIndex<'rows>,
    types: Types,
    observed: DomainWork,
    // This first source form requires an explicit direct request/header port
    // for every application. Broader nested function-item-only forms reject.
    used_functions: Vec<bool>,
    used_traits: Vec<bool>,
}
impl<'index, 'rows> DomainIndex<'index, 'rows> {
    pub(super) fn new(
        headers: &'index HeaderIndex<'rows>,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let first = work.steps;
        let roots = work.roots;
        let functions = headers.rows().function_applications.len();
        let traits = headers.rows().trait_applications.len();
        let applications = functions.saturating_add(traits);
        work.charge(applications)?;
        let binders = BinderIndex::new(headers, work)?;
        let mut types = Types::new(headers, &binders, work)?;
        let overhead =
            size_of::<Self>().saturating_sub(size_of::<BinderIndex<'_>>() + size_of::<Types>());
        types.bytes = types
            .bytes
            .saturating_add(overhead)
            .saturating_add(applications);
        types.peak = types
            .peak
            .saturating_add(overhead)
            .saturating_add(applications);
        if types.peak > crate::support::crates::MAX_BYTES {
            return Err("source domain view exceeds the combined storage budget");
        }
        let mut index = Self {
            headers,
            binders,
            types,
            observed: DomainWork::default(),
            used_functions: vec![false; functions],
            used_traits: vec![false; traits],
        };
        index.header_roots(work)?;
        index.originals(work)?;
        work.charge(applications)?;
        if index.used_functions.contains(&false) || index.used_traits.contains(&false) {
            return Err("source application has no actual owning header/body request");
        }
        index.observed.steps = work.steps.saturating_sub(first);
        index.observed.roots = work.roots.saturating_sub(roots);
        index.observed.binder_frames = headers.rows().binders.len();
        index.observed.binder_queries = index.binders.queries;
        index.observed.binder_depth = index.binders.max_depth;
        index.observed.type_nodes = index.types.visits;
        index.observed.type_cache_hits = index.types.cache_hits;
        Ok(index)
    }
    #[must_use]
    pub const fn work(&self) -> DomainWork {
        self.observed
    }
    #[must_use]
    pub const fn retained_bytes(&self) -> usize {
        self.types.bytes
    }
    #[must_use]
    pub const fn peak_bytes(&self) -> usize {
        self.types.peak
    }
    /// # Errors
    /// Rejects unknown frame coordinates; no frontend/proof identity is minted.
    pub fn binder_contains(
        &self,
        ancestor: g::BinderRef,
        child: g::BinderRef,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        self.binders.ancestor(ancestor, child, work)
    }
    fn function(
        &self,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
        work: &mut Work,
    ) -> Result<&'rows s::FunctionHeader, &'static str> {
        let Some(HeaderRef::Function(index)) = self.headers.header(declaration, work)? else {
            return Err("source body/context has no actual function header");
        };
        self.headers
            .rows()
            .functions
            .get(index as usize)
            .ok_or("source function header is absent")
    }
    fn function_use(
        &mut self,
        context: Context,
        application: s::FunctionApplicationRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.types
            .function_application(self.headers, context, application, work)?;
        self.used_functions[application.0 as usize] = true;
        Ok(())
    }
    fn trait_use(
        &mut self,
        context: Context,
        application: s::TraitApplicationRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.types
            .trait_application(self.headers, context, application, work)?;
        self.used_traits[application.0 as usize] = true;
        Ok(())
    }
    fn scratch(&mut self, bytes: usize, work: &mut Work) -> Result<(), &'static str> {
        work.charge(1)?;
        let peak = self.types.bytes.saturating_add(bytes);
        if peak > crate::support::crates::MAX_BYTES {
            return Err("source operation index exceeds shared retained/scratch budget");
        }
        self.types.peak = self.types.peak.max(peak);
        Ok(())
    }
}
