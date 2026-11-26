//! A shared row-domain view still cannot create a descriptor or paid body.
#[path = "domain_index/operation_domains.rs"]
mod body_ports;
#[path = "port_plan.rs"]
mod input_plan;
#[path = "domain_index/authored_inputs.rs"]
mod input_ports;
#[path = "domain_index/root_ports.rs"]
mod root_ports;
#[path = "domain_index/ssa_ports.rs"]
mod snapshots;
#[path = "domain_index/ancestry_arguments.rs"]
mod specialization_roots;
use super::{
    HeaderIndex, HeaderRef, Work,
    binder_domains::BinderIndex,
    type_domains::{Context, Types},
};
use crate::support::crates::generics::{self as g, source as s};
use crate::support::crates::owner_catalog::DeclarationRef;

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
pub use input_plan::{InputWork, OperationPort, OriginalInputPlan, RequiredInput, RequiredKind};

/// What a function application calls. A constructor's formals, premises and
/// Self are its enum's or struct's header's; its inputs are the payload fields
/// and its result is that nominal. It has no late binder and is safe Rust.
#[derive(Clone, Copy)]
enum Target<'rows> {
    Function(&'rows s::FunctionHeader),
    Constructor {
        enumeration: &'rows s::NominalHeader,
        payload: s::ConstructorPayload<'rows>,
    },
}
impl Target<'_> {
    const fn scope(self) -> crate::support::crates::owner_catalog::DeclarationRef {
        match self {
            Self::Function(header) => header.scope,
            Self::Constructor { enumeration, .. } => enumeration.scope,
        }
    }
    const fn inputs(self) -> g::Range {
        match self {
            Self::Function(header) => header.inputs,
            Self::Constructor { enumeration, .. } => enumeration.inputs,
        }
    }
    const fn parent(self) -> Option<crate::support::crates::owner_catalog::DeclarationRef> {
        match self {
            Self::Function(header) => header.parent,
            Self::Constructor { .. } => None,
        }
    }
}

/// An original carries a callable signature, a closure header, or an initializer's
/// declared result and inherited environment.
#[derive(Clone, Copy)]
enum OriginalHeader<'rows> {
    Function(&'rows s::FunctionHeader),
    Constant {
        result: g::TypeRef,
        parent: Option<DeclarationRef>,
    },
    Closure {
        header: &'rows s::ClosureHeader,
        parent: Option<DeclarationRef>,
    },
    Anonymous {
        header: &'rows s::AnonymousHeader,
        parent: Option<DeclarationRef>,
    },
}

impl OriginalHeader<'_> {
    const fn parent(self) -> Option<DeclarationRef> {
        match self {
            Self::Function(header) => header.parent,
            Self::Constant { parent, .. }
            | Self::Closure { parent, .. }
            | Self::Anonymous { parent, .. } => parent,
        }
    }
    const fn unsafe_function(self) -> bool {
        matches!(self, Self::Function(header) if header.callable.unsafe_)
    }
    fn entry(
        self,
        rows: &s::SourceRows,
        original: &s::Original,
    ) -> Result<(g::Range, g::TypeRef, Option<g::BinderRef>), &'static str> {
        match self {
            Self::Function(header) => {
                Ok((header.original.parameters, header.original.result, None))
            }
            Self::Constant { result, .. } => Ok((g::Range { start: 0, end: 0 }, result, None)),
            Self::Anonymous { .. } => Ok((g::Range { start: 0, end: 0 }, original.result, None)),
            Self::Closure { header, .. } => {
                let Some(&s::Type::Function(signature)) =
                    rows.types.get(header.body_signature().0 as usize)
                else {
                    return Err("source closure header lacks its actual function signature");
                };
                Ok((
                    signature.parameters,
                    signature.result,
                    Some(signature.binder),
                ))
            }
        }
    }
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
    /// Version thirteen: branching snapshot bodies and plain reborrow inputs.
    bodies: bool,
    declaration_policies: bool,
    bootstrap_bindings: bool,
    attributes: Option<&'index s::HeaderAttributeColumns>,
    metadata_member: Option<s::DeclarationAddress>,
}
impl<'index, 'rows> DomainIndex<'index, 'rows> {
    pub(super) fn new(
        headers: &'index HeaderIndex<'rows>,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::new_profile(headers, s::Profile::Source2024FunctionContracts, None, work)
    }
    pub(super) fn new_profile(
        headers: &'index HeaderIndex<'rows>,
        profile: s::Profile,
        attributes: Option<&'index s::HeaderAttributeColumns>,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let conversions = profile.conversion_roles();
        let bodies = profile.body_nodes();
        let first = work.steps;
        let roots = work.roots;
        let functions = headers.rows().function_applications.len();
        let traits = headers.rows().trait_applications.len();
        let applications = functions.saturating_add(traits);
        work.charge(applications)?;
        let binders = BinderIndex::new(headers, work, profile)?;
        let mut types = Types::new(headers, &binders, profile, work)?;
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
            bodies,
            declaration_policies: profile.declaration_policies(),
            bootstrap_bindings: profile.bootstrap_bindings(),
            attributes,
            metadata_member: slice_metadata_member(headers, profile, work)?,
        };
        index.header_roots(work)?;
        index.originals(work, conversions, bodies)?;
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
    /// Complete substituted signatures and operation-derived input requests.
    /// This neutral result cannot publish a descriptor or authorize import.
    /// # Errors
    /// Rejects unsupported or incomplete originals and aggregate work/storage limits.
    pub fn original_inputs(
        &self,
        work: &mut Work,
    ) -> Result<OriginalInputPlan<'rows>, &'static str> {
        input_plan::build(self, work)
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
    fn original_header(
        &self,
        declaration: DeclarationRef,
        work: &mut Work,
    ) -> Result<OriginalHeader<'rows>, &'static str> {
        let rows = self.headers.rows();
        match self.headers.header(declaration, work)? {
            Some(HeaderRef::Function(index)) => Ok(OriginalHeader::Function(
                rows.functions
                    .get(index as usize)
                    .ok_or("source function header is absent")?,
            )),
            Some(HeaderRef::Constant(index)) if self.declaration_policies => {
                let header = &rows.constants()[index as usize];
                if !matches!(header.body, s::ConstantBody::Original(_)) {
                    return Err("source constant has no actual initializer body");
                }
                let lexical = rows.declarations[declaration.0 as usize]
                    .definition
                    .parent
                    .ok_or("source constant initializer has no declaring owner")?;
                let parent = rows
                    .declarations
                    .get(lexical.0 as usize)
                    .is_some_and(|row| {
                        matches!(
                            row.definition.kind,
                            s::DefinitionKind::Trait | s::DefinitionKind::SelfType
                        )
                    })
                    .then_some(lexical);
                Ok(OriginalHeader::Constant {
                    result: header.ty,
                    parent,
                })
            }
            _ => Err("source body/context has no actual function header"),
        }
    }
    fn original_header_for_body(
        &self,
        body: crate::support::crates::owner_catalog::BodyRef,
        declaration: DeclarationRef,
        work: &mut Work,
    ) -> Result<OriginalHeader<'rows>, &'static str> {
        if self.declaration_policies
            && let Some(closures) = self.headers.rows().closures()
        {
            if let Some((_, header)) = closures.header_for_body(body) {
                if header.owner != declaration {
                    return Err("source closure body belongs to another declaration owner");
                }
                let parent = self.original_header(declaration, work)?.parent();
                return Ok(OriginalHeader::Closure { header, parent });
            }
            if let Some((_, header)) = closures.anonymous_for_body(body) {
                if header.owner != declaration {
                    return Err("source anonymous body belongs to another declaration owner");
                }
                let parent = self.original_header(declaration, work)?.parent();
                return Ok(OriginalHeader::Anonymous { header, parent });
            }
        }
        self.original_header(declaration, work)
    }
    /// A function application's target: its function header, or under
    /// version thirteen a called tuple-variant or tuple-struct constructor.
    fn target(
        &self,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
        work: &mut Work,
    ) -> Result<Target<'rows>, &'static str> {
        let rows = self.headers.rows();
        if let Some(constructor) =
            s::Constructor::of((rows.enums(), &rows.declarations), declaration, |row| {
                self.headers.header(row, work)
            })?
        {
            let enumeration = rows
                .nominals
                .get(constructor.nominal as usize)
                .ok_or("source constructor enum header is absent")?;
            return Ok(Target::Constructor {
                enumeration,
                payload: constructor.payload((rows.enums(), &rows.nominals, &rows.fields))?,
            });
        }
        self.function(declaration, work).map(Target::Function)
    }
    fn function_use(
        &mut self,
        context: Context,
        application: s::FunctionApplicationRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let mut pending = vec![application];
        while let Some(current) = pending.pop() {
            self.types
                .function_application(self.headers, context, current, work)?;
            self.used_functions[current.0 as usize] = true;
            if !self.declaration_policies {
                continue;
            }
            let rows = self.headers.rows();
            for argument in rows.function_applications[current.0 as usize]
                .arguments
                .slice(&rows.arguments)?
            {
                work.charge(1)?;
                if let s::Argument::Type(ty) = *argument
                    && let Some(&s::Type::FunctionItem(nested)) = rows.types.get(ty.0 as usize)
                    && !self.used_functions[nested.0 as usize]
                {
                    pending.push(nested);
                }
            }
        }
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
        if !self.declaration_policies {
            return Ok(());
        }
        let rows = self.headers.rows();
        let raw = rows.trait_applications[application.0 as usize];
        let mut nested = Vec::new();
        if let Some(&s::Type::FunctionItem(item)) = rows.types.get(raw.subject.0 as usize)
            && !self.used_functions[item.0 as usize]
        {
            nested.push(item);
        }
        for argument in raw.arguments.slice(&rows.arguments)? {
            work.charge(1)?;
            if let s::Argument::Type(ty) = *argument
                && let Some(&s::Type::FunctionItem(item)) = rows.types.get(ty.0 as usize)
                && !self.used_functions[item.0 as usize]
            {
                nested.push(item);
            }
        }
        for item in nested {
            self.function_use(context, item, work)?;
        }
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

/// Preserve the exact provider-owned Pointee member used by slice and str
/// signature recipes. The consumer still checks the complete typed role.
fn slice_metadata_member(
    headers: &HeaderIndex<'_>,
    profile: s::Profile,
    work: &mut Work,
) -> Result<Option<s::DeclarationAddress>, &'static str> {
    use crate::support::bindings::LangItem;
    if !profile.declaration_policies() {
        return Ok(None);
    }
    let rows = headers.rows();
    let mut parent = None;
    let mut member = None;
    for binding in &rows.roles {
        work.charge(1)?;
        if binding.declaration.artifact != rows.policy.provider {
            continue;
        }
        match binding.role {
            LangItem::PointeeTrait => parent = Some(binding.declaration.declaration),
            LangItem::MetadataType => member = Some(binding.declaration),
            _ => {}
        }
    }
    let (Some(parent), Some(member)) = (parent, member) else {
        return Ok(None);
    };
    let Some(HeaderRef::AssociatedType(index)) = headers.header(member.declaration, work)? else {
        return Err("source slice metadata requires its actual associated declaration");
    };
    if rows.associated_types[index as usize].parent != parent {
        return Err("source slice metadata belongs to another trait");
    }
    Ok(Some(member))
}
