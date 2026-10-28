//! Source headers reserve genuine declarations before any body row is published.
mod admitted_contracts;
mod advisory_rows;
mod availability_rows;
pub(crate) use admitted_contracts::HeaderCompletion;
mod authored_functions;
mod authored_traits;
mod canonical_nodes;
mod collected_formals;
mod collected_provider;
mod contract_rows;
mod exposed_bindings;
mod implementation_headers;
mod modifier_rows;
mod namespace_emission;
mod scoped_parameters;
mod self_requests;
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, DefKind, Namespace, ScopeId},
    },
    sema::{associated::OwnerKind, model::Declarations},
    support::crates::{
        self, CrateIdentity,
        generics::{self as g, source as s},
        owner_catalog::DeclarationRef,
    },
};
pub(crate) use advisory_rows::require_defaultness_transport;
pub(crate) use canonical_nodes::Context;
pub(crate) use collected_provider::{
    CollectedArtifact, CollectedReceipt, CollectedRows, validate_collected_rows,
};

/// Observation of producer work, not proof or accepted metadata authority.
#[derive(Clone, Copy, Debug, Default)]
pub struct SourceWork {
    pub steps: usize,
    pub reserved_bytes: usize,
    pub scratch_bytes: usize,
    pub contextual_types: usize,
    pub ancestor_steps: usize,
    pub roots: usize,
    pub cache_hits: usize,
    pub role_installation_steps: usize,
}

/// Private draft only. Header/body validation and complete proof consumption must
/// finish before this stage can create a payload or expose a client descriptor.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExportMode {
    Legacy,
    Namespace,
    ModuleContracts,
    Availability,
}

pub(crate) struct Stage {
    pub(crate) rows: s::SourceRows,
    pub(crate) definitions: Vec<Option<DeclarationRef>>,
    steps: usize,
    bytes: usize,
    scratch: usize,
    graph: canonical_nodes::Graph,
    inputs: contract_rows::Index,
    functions: authored_functions::Index,
    role_installation_steps: usize,
}
impl Stage {
    pub(crate) fn work(&self) -> SourceWork {
        let (contextual_types, ancestor_steps, roots, cache_hits) = self.graph.work();
        SourceWork {
            steps: self.steps,
            reserved_bytes: self.bytes,
            scratch_bytes: self.scratch,
            contextual_types,
            ancestor_steps,
            roots,
            cache_hits,
            role_installation_steps: self.role_installation_steps,
        }
    }
    pub(crate) fn local_declaration(
        &mut self,
        declaration: DeclarationRef,
        context: Context,
        scope: u32,
        ordinal: u32,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let body = context
            .body()
            .ok_or("source local has no original body context")?;
        let row = self
            .rows
            .declarations
            .get_mut(declaration.0 as usize)
            .ok_or("source local declaration is unknown")?;
        if row.definition.parent != Some(context.owner())
            || row.local.is_some()
            || !matches!(
                row.definition.kind,
                s::DefinitionKind::ValueParameter | s::DefinitionKind::Local
            )
        {
            return Err("source local declaration has a duplicate or foreign actual owner");
        }
        row.local = Some(s::LocalDeclaration {
            body,
            scope,
            ordinal,
        });
        Ok(())
    }
    /// Metadata identity only. The higher body owner must still consume an exact
    /// successful original proof and paired input before retaining operation rows.
    pub(crate) fn original_context(&mut self, definition: DefId) -> Result<Context, &'static str> {
        self.charge(1)?;
        let body = self
            .functions
            .bodies
            .get(&definition)
            .copied()
            .ok_or("source declaration has no reserved provided original")?;
        if !matches!(self.functions.originals.get(body.0 as usize), Some(None)) {
            return Err("source original is already collected or has no reserved coverage row");
        }
        self.functions
            .contexts
            .get(body.0 as usize)
            .copied()
            .ok_or("source original context is absent")
    }

    pub(crate) fn retain_original(
        &mut self,
        context: Context,
        original: s::Original,
        inputs: s::BodyInputs,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let body = context
            .body()
            .ok_or("source original context has no body identity")?;
        if self.functions.contexts.get(body.0 as usize) != Some(&context)
            || !matches!(self.functions.originals.get(body.0 as usize), Some(None))
            || original.body != body
            || original.declaration != context.owner()
            || inputs.body != body
            || inputs.frame != self.binder(context)?
        {
            return Err("source original changed its exact reserved body or input context");
        }
        self.reserve(4 * (size_of::<s::Original>() + size_of::<s::BodyInputs>()))?;
        self.functions.originals[body.0 as usize] = Some((original, inputs));
        Ok(())
    }

    pub(crate) fn into_rows(mut self) -> Result<(s::SourceRows, SourceWork), &'static str> {
        self.charge(self.functions.originals.len())?;
        if self.functions.originals.iter().any(Option::is_none)
            || !self.rows.originals.is_empty()
            || !self.rows.body_inputs.is_empty()
        {
            return Err(
                "source metadata is missing an actual provided original or its complete inputs",
            );
        }
        let work = self.work();
        for slot in self.functions.originals {
            let (body, inputs) =
                slot.ok_or("source original coverage changed during final staging")?;
            self.rows.originals.push(body);
            self.rows.body_inputs.push(inputs);
        }
        Ok((self.rows, work))
    }
    pub(crate) fn begin(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Legacy)
    }

    /// Private inspection only; no profile or descriptor admission is changed.
    pub(crate) fn begin_with_ordinary_traits(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Namespace)
    }

    pub(crate) fn begin_module_contracts(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::ModuleContracts)
    }
    pub(crate) fn begin_module_availability(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Availability)
    }
    fn begin_mode(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
        mode: ExportMode,
    ) -> Result<Self, &'static str> {
        if let Some(producer) = report.lint_producer() {
            producer
                .require_deprecation_transport(&report.package)
                .map_err(|error| error.message)?;
        }
        let provider = report
            .package
            .source_core()
            .ok_or("source export requires its actual own-provider policy")?;
        if provider.current != *identity || provider.provider != *identity {
            return Err("source export identity differs from its actual provider");
        }
        if facts.members.has_contextual_fields() {
            return Err("contextual aggregate field metadata is not implemented");
        }
        if report.package.library_policy().is_some() && mode != ExportMode::Availability {
            return Err("source stability contracts are outside this metadata profile");
        }
        if mode == ExportMode::Availability
            && report
                .package
                .library_policy()
                .and_then(crate::resolve::stability::Library::staged_inventory)
                .is_none()
        {
            return Err("source availability export requires its completed actual library policy");
        }
        if report.resolution.bindings.has_contextual_items()
            || report.package.marks.context_count() != 1
        {
            return Err("source macro/semantic-name provenance metadata is not implemented");
        }
        facts
            .source_bindings
            .as_deref()
            .ok_or("source export has no actual role catalog")?
            .require_provider_report(report)?;
        let unit = report
            .package
            .units
            .first()
            .ok_or("source export has no defining unit")?;
        if unit.edition() != crate::support::edition::Edition::Rust2024 {
            return Err("source export requires defining Rust2024 edition");
        }
        let count = report.resolution.definitions.len();
        let mut stage = Self {
            rows: rows(unit.edition()),
            definitions: Vec::new(),
            steps: 0,
            bytes: size_of::<Self>(),
            scratch: 0,
            graph: canonical_nodes::Graph::default(),
            inputs: contract_rows::Index::default(),
            functions: authored_functions::Index::default(),
            role_installation_steps: facts
                .source_bindings
                .as_deref()
                .map_or(0, crate::sema::compiler_items::Catalog::binding_work),
        };
        stage.reserve(count.saturating_mul(size_of::<Option<DeclarationRef>>()))?;
        stage.definitions.resize(count, None);
        stage.declarations(report, facts, mode)?;
        stage.scopes(report, facts)?;
        stage.traits(report, facts, mode != ExportMode::Legacy)?;
        stage.functions(report, facts)?;
        stage.implementations(facts)?;
        Ok(stage)
    }

    pub(crate) fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > crates::MAX_COMPONENTS {
            return Err("source export work budget exceeded");
        }
        Ok(())
    }
    pub(crate) fn reserve(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.charge(1)?;
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes.saturating_add(self.scratch) > crates::MAX_BYTES {
            return Err("source export retained and scratch budget exceeded");
        }
        Ok(())
    }
    pub(crate) fn scratch(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.charge(1)?;
        self.scratch = self.scratch.max(bytes);
        if self.bytes.saturating_add(self.scratch) > crates::MAX_BYTES {
            return Err("source export retained and scratch budget exceeded");
        }
        Ok(())
    }
    pub(crate) fn declaration(
        &mut self,
        definition: DefId,
    ) -> Result<DeclarationRef, &'static str> {
        self.charge(1)?;
        self.definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or("source header names an unreserved actual declaration")
    }

    fn declarations(
        &mut self,
        report: &Report,
        facts: &Declarations,
        mode: ExportMode,
    ) -> Result<(), &'static str> {
        let mut owners = scope_owners(report, facts, self)?;
        let root = report
            .resolution
            .crate_root
            .ok_or("source provider has no actual root")?;
        let provider = report
            .package
            .source_core()
            .ok_or("source provider policy disappeared")?;
        self.reserve(
            report
                .resolution
                .definitions
                .len()
                .saturating_mul(size_of::<u32>() + size_of::<Option<DefId>>()),
        )?;
        let mut ordinals = vec![0_u32; report.resolution.definitions.len()];
        let mut parents = Vec::with_capacity(report.resolution.definitions.len());
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.charge(1)?;
            let Some(kind) = kind(definition.kind, definition.namespace)? else {
                continue;
            };
            let id =
                DefId(u32::try_from(index).map_err(|_| "source declaration identity overflow")?);
            if !matches!(mode, ExportMode::ModuleContracts | ExportMode::Availability)
                && definition.kind == DefKind::Trait
                && crate::resolve::impl_restrictions::clause(report, id)
                    .map_err(|_| "source trait implementation domain is invalid")?
                    .is_some()
            {
                return Err("restricted trait implementation policy transport is not implemented");
            }
            let parent = if Some(id) == report.resolution.crate_root {
                None
            } else {
                let mut scope = definition.scope;
                if owners.get(scope.0 as usize).copied().flatten() == Some(id) {
                    scope = report
                        .resolution
                        .scopes
                        .get(scope.0 as usize)
                        .and_then(|scope| scope.parent)
                        .ok_or("source declaration owns no actual parent scope")?;
                }
                Some(find_owner(report, &mut owners, scope, self)?)
            };
            let ordinal = if let Some(parent) = parent {
                let next = ordinals
                    .get_mut(parent.0 as usize)
                    .ok_or("source lexical owner is unknown")?;
                let ordinal = *next;
                *next = next.checked_add(1).ok_or("source child ordinal overflow")?;
                ordinal
            } else {
                0
            };
            // The resolver names its exact root `crate`; its external module
            // spelling belongs to the authenticated artifact identity.
            let name = if id == root {
                provider.provider.name()
            } else {
                report.package.names.get(definition.name)
            };
            self.reserve(4 * size_of::<s::Declaration>() + name.len())?;
            let row = DeclarationRef(
                u32::try_from(self.rows.declarations.len())
                    .map_err(|_| "source declaration table overflow")?,
            );
            self.definitions[index] = Some(row);
            parents.push(parent);
            self.rows.declarations.push(s::Declaration {
                definition: s::Definition { kind, parent: None },
                ordinal,
                name: Some(name.to_owned()),
                public: definition.access == Access::Public,
                sites: 1,
                local: None,
            });
        }
        for (index, parent) in parents.into_iter().enumerate() {
            self.charge(1)?;
            if let Some(parent) = parent {
                let parent = self.declaration(parent)?;
                self.rows.declarations[index].definition.parent = Some(parent);
            }
        }
        Ok(())
    }
}

fn kind(kind: DefKind, namespace: Namespace) -> Result<Option<s::DefinitionKind>, &'static str> {
    Ok(Some(match kind {
        DefKind::Module => s::DefinitionKind::Module,
        DefKind::Trait => s::DefinitionKind::Trait,
        DefKind::SelfType => s::DefinitionKind::SelfType,
        DefKind::Function => s::DefinitionKind::Function,
        DefKind::Generic => match namespace {
            Namespace::Type => s::DefinitionKind::TypeParameter,
            Namespace::Lifetime => s::DefinitionKind::RegionParameter,
            _ => return Err("source const parameters are outside the initial source profile"),
        },
        DefKind::Lifetime => s::DefinitionKind::RegionParameter,
        DefKind::Parameter => s::DefinitionKind::ValueParameter,
        DefKind::Local => s::DefinitionKind::Local,
        DefKind::Primitive | DefKind::BuiltinMacro | DefKind::BuiltinTrait => return Ok(None),
        _ => return Err("source declaration kind is outside the initial source profile"),
    }))
}

fn scope_owners(
    report: &Report,
    facts: &Declarations,
    stage: &mut Stage,
) -> Result<Vec<Option<DefId>>, &'static str> {
    stage.reserve(
        report
            .resolution
            .scopes
            .len()
            .saturating_mul(size_of::<Option<DefId>>()),
    )?;
    let mut owners = vec![None; report.resolution.scopes.len()];
    let root = report
        .resolution
        .crate_root
        .ok_or("source provider has no actual root declaration")?;
    let scope = report
        .resolution
        .definitions
        .get(root.0 as usize)
        .and_then(|definition| definition.members)
        .ok_or("source provider root has no actual scope")?;
    owners[scope.0 as usize] = Some(root);
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        stage.charge(1)?;
        if let Some(scope) = definition.members {
            owners[scope.0 as usize] = Some(DefId(
                u32::try_from(index).map_err(|_| "source owner identity overflow")?,
            ));
        }
    }
    for owner in facts.associated.owners() {
        stage.charge(1)?;
        let owner = owner.source_owner()?;
        let definition = match owner.kind {
            OwnerKind::Trait { definition, .. } => definition,
            OwnerKind::Impl { .. } => owner.self_definition,
            OwnerKind::ImportedImpl(_) => {
                return Err("source export cannot acquire an imported syntax owner");
            }
        };
        let module = report.resolution.scopes
            [report.resolution.definitions[definition.0 as usize].scope.0 as usize]
            .module;
        let scope = *report
            .resolution
            .node_scopes
            .get(&(module, owner.source.node))
            .ok_or("source associated owner has no actual indexed scope")?;
        owners[scope.0 as usize] = Some(definition);
    }
    for function in &facts.functions {
        stage.charge(1)?;
        let module = report.resolution.scopes[report.resolution.definitions
            [function.definition.0 as usize]
            .scope
            .0 as usize]
            .module;
        let scope = *report
            .resolution
            .node_scopes
            .get(&(module, function.node.node))
            .ok_or("source function has no actual indexed scope")?;
        owners[scope.0 as usize] = Some(function.definition);
    }
    Ok(owners)
}

fn find_owner(
    report: &Report,
    owners: &mut [Option<DefId>],
    mut scope: ScopeId,
    stage: &mut Stage,
) -> Result<DefId, &'static str> {
    let first = scope;
    let owner = loop {
        stage.charge(1)?;
        if let Some(owner) = owners.get(scope.0 as usize).copied().flatten() {
            break owner;
        }
        scope = report
            .resolution
            .scopes
            .get(scope.0 as usize)
            .and_then(|scope| scope.parent)
            .ok_or("source declaration has no enclosing real owner")?;
    };
    let mut scope = first;
    while owners[scope.0 as usize].is_none() {
        stage.charge(1)?;
        owners[scope.0 as usize] = Some(owner);
        scope = report.resolution.scopes[scope.0 as usize]
            .parent
            .ok_or("source scope lost its actual parent")?;
    }
    Ok(owner)
}

fn rows(edition: crate::support::edition::Edition) -> s::SourceRows {
    s::SourceRows {
        policy: s::Policy {
            edition,
            provider: s::ArtifactRef(0),
            roles: g::Range { start: 0, end: 0 },
        },
        declarations: Vec::new(),
        body_owners: Vec::new(),
        slots: Vec::new(),
        scopes: Vec::new(),
        binders: Vec::new(),
        types: Vec::new(),
        roots: Vec::new(),
        regions: Vec::new(),
        arguments: Vec::new(),
        clauses: Vec::new(),
        clause_indices: Vec::new(),
        region_bounds: Vec::new(),
        region_slots: Vec::new(),
        trait_applications: Vec::new(),
        function_applications: Vec::new(),
        traits: Vec::new(),
        implementations: Vec::new(),
        functions: Vec::new(),
        formals: Vec::new(),
        members: Vec::new(),
        member_links: Vec::new(),
        roles: Vec::new(),
        originals: Vec::new(),
        body_inputs: Vec::new(),
        body_units: Vec::new(),
    }
}
