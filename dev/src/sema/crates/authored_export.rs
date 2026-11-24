//! Source headers reserve genuine declarations before any body row is published.
mod admission_rows;
mod admitted_contracts;
mod advisory_rows;
mod availability_rows;
pub(crate) use admitted_contracts::HeaderCompletion;
mod authored_functions;
mod authored_traits;
mod bound_aliases;
mod census;
pub use census::{DeclCensus, DeclFailure};
mod canonical_nodes;
mod collected_formals;
mod collected_provider;
mod contract_rows;
mod deprecation_rows;
mod evaluated_constants;
pub(crate) use evaluated_constants::prepare_inherent_constants;
mod exposed_bindings;
mod implementation_headers;
mod modifier_rows;
mod namespace_emission;
#[path = "authored_export/record_headers.rs"]
mod nominal_headers;
#[path = "authored_export/versioned_sources.rs"]
mod non_type_sources;
mod operation_exports;
mod payments;
mod promoted_references;
mod scoped_parameters;
mod self_requests;
mod statics;
#[path = "authored_export/sum_headers.rs"]
mod sum_headers;
mod verbatim_exports;
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, DefKind, Definition, Namespace, Origin, ScopeId},
        unit::NodeKey,
    },
    sema::{associated::OwnerKind, model::Declarations},
    support::{
        bringup::waiving,
        crates::{
            self, CrateIdentity,
            generics::{self as g, source as s},
            owner_catalog::DeclarationRef,
        },
    },
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, TypeId, lifetime_proof::ScopeId as ProofScope},
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
    FeaturePredicates,
    Definitions,
    Compatibility,
    Callables,
    Safety,
    Conversions,
    Nominals,
    CompilerOperations,
}

impl ExportMode {
    fn feature_predicates(self) -> bool {
        matches!(
            self,
            Self::FeaturePredicates
                | Self::Definitions
                | Self::Compatibility
                | Self::Callables
                | Self::Safety
                | Self::Conversions
                | Self::Nominals
                | Self::CompilerOperations
        )
    }
    fn availability(self) -> bool {
        matches!(
            self,
            Self::Availability
                | Self::FeaturePredicates
                | Self::Definitions
                | Self::Compatibility
                | Self::Callables
                | Self::Safety
                | Self::Conversions
                | Self::Nominals
                | Self::CompilerOperations
        )
    }
    fn module_contracts(self) -> bool {
        matches!(
            self,
            Self::ModuleContracts
                | Self::Availability
                | Self::FeaturePredicates
                | Self::Definitions
                | Self::Compatibility
                | Self::Callables
                | Self::Safety
                | Self::Conversions
                | Self::Nominals
                | Self::CompilerOperations
        )
    }
    fn exact(self) -> bool {
        matches!(
            self,
            Self::Definitions
                | Self::Compatibility
                | Self::Callables
                | Self::Safety
                | Self::Conversions
                | Self::Nominals
                | Self::CompilerOperations
        )
    }
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
    nominal_transport: bool,
    compiler_operations: bool,
    body_transport: BodyTransport,
    /// Version thirteen: each exported enum variant's constructor declaration,
    /// indexed by its local `VariantId`.
    variants: Vec<Option<DeclarationRef>>,
    /// Version thirteen: an exported record variant gives every field row from
    /// its own on a name slot.
    record_variants: bool,
}
/// The original body vocabulary a module publication may retain.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyTransport {
    Ordinary,
    /// Versions twelve and thirteen: original CFGs may retain Branch and Panic exits.
    Branches,
    /// Version thirteen: branches plus primitive Binary operations.
    Nodes,
}
impl Stage {
    pub(crate) const fn nominal_transport(&self) -> bool {
        self.nominal_transport
    }
    pub(crate) fn branch_transport(&self) -> bool {
        self.body_transport != BodyTransport::Ordinary
    }
    pub(crate) fn body_nodes(&self) -> bool {
        self.body_transport == BodyTransport::Nodes
    }
    pub(crate) fn declaration_policies(&self) -> bool {
        self.body_nodes() && self.compiler_operations
    }
    pub(crate) fn callable_effects(&self) -> bool {
        self.functions.callable_effects
    }
    pub(crate) fn conversion_roles(&self) -> bool {
        self.functions.conversions
    }
    pub(crate) fn exact_linkage(&self) -> bool {
        self.functions.exact
    }
    pub(crate) fn feature_predicates(&self) -> bool {
        self.inputs.selected_methods.is_some()
    }
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
        let mut operations = self.rows.compiler_operations.iter().peekable();
        for (index, slot) in self.functions.originals.iter().enumerate() {
            let operation = operations
                .peek()
                .is_some_and(|operation| operation.body.0 as usize == index);
            if operation {
                operations.next();
            }
            if operation == slot.is_some() {
                return Err(
                    "source body reservation lost exclusive ordinary/compiler-operation coverage",
                );
            }
        }
        if operations.next().is_some()
            || !self.rows.originals.is_empty()
            || !self.rows.body_inputs.is_empty()
        {
            return Err("source body reservation has foreign or duplicate publication rows");
        }
        self.charge(self.rows.exact_callables.len().saturating_mul(
            self.rows.exact_callables.len().checked_ilog2().unwrap_or(0) as usize + 1,
        ))?;
        self.rows
            .exact_callables
            .sort_unstable_by_key(|row| row.declaration.0);
        let work = self.work();
        for slot in self.functions.originals.into_iter().flatten() {
            let (body, inputs) = slot;
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
    pub(crate) fn begin_module_feature_predicates(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::FeaturePredicates)
    }
    pub(crate) fn begin_module_definitions(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Definitions)
    }
    pub(crate) fn begin_module_compatibility(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Compatibility)
    }
    pub(crate) fn begin_module_callables(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Callables)
    }
    pub(crate) fn begin_module_safety(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Safety)
    }
    pub(crate) fn begin_module_conversions(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Conversions)
    }
    pub(crate) fn begin_compiler_operations(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::CompilerOperations)
    }
    pub(crate) fn begin_module_nominals(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_mode(report, facts, identity, ExportMode::Nominals)
    }
    /// The complete version-ten collection; only original CFG exits widen.
    pub(crate) fn begin_module_branches(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_with(
            report,
            facts,
            identity,
            ExportMode::Nominals,
            BodyTransport::Branches,
        )
    }
    /// The version-twelve collection; originals also keep primitive Binary values.
    pub(crate) fn begin_module_bodies(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_with(
            report,
            facts,
            identity,
            ExportMode::Nominals,
            BodyTransport::Nodes,
        )
    }
    /// Version fourteen combines node bodies with the authenticated operation owner.
    pub(crate) fn begin_module_body_operations(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
    ) -> Result<Self, &'static str> {
        Self::begin_with(
            report,
            facts,
            identity,
            ExportMode::CompilerOperations,
            BodyTransport::Nodes,
        )
    }
    fn begin_mode(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
        mode: ExportMode,
    ) -> Result<Self, &'static str> {
        Self::begin_with(report, facts, identity, mode, BodyTransport::Ordinary)
    }
    /// The body vocabulary is fixed before any header row: version thirteen
    /// also widens language-role admission.
    fn begin_with(
        report: &Report,
        facts: &mut Declarations,
        identity: &CrateIdentity,
        mode: ExportMode,
        body_transport: BodyTransport,
    ) -> Result<Self, &'static str> {
        let mut stage = Self::header_draft(report, facts, identity, mode, body_transport)?;
        if stage.declaration_policies() {
            stage.prepare_callable_arena(facts)?;
        }
        let admission_rows::CollectedDeclarations { leaves, order } =
            stage.declarations(report, facts, mode)?;
        stage.scopes(report, facts, &order)?;
        drop(order);
        stage.traits(report, facts, mode != ExportMode::Legacy)?;
        if stage.nominal_transport {
            stage.nominals(report, facts)?;
        }
        if stage.compiler_operations || stage.body_nodes() {
            stage.associated_headers(report, facts)?;
        }
        let provided_constants = stage.provided_constants(report, facts)?;
        if stage.body_nodes() && provided_constants.is_empty() {
            stage.constant_headers(report, facts, identity)?;
        }
        stage.alias_headers(facts, leaves.aliases)?;
        stage.trait_alias_headers(report, facts, leaves.trait_aliases)?;
        let statics = stage.static_headers(facts, identity, leaves.statics)?;
        stage.functions(report, facts, identity, &provided_constants)?;
        if !provided_constants.is_empty() {
            stage.constant_headers(report, facts, identity)?;
        }
        stage.complete_static_headers(facts, statics)?;
        stage.implementations(report, facts)?;
        stage.diagnostic_roles(report)?;
        stage.install_private_symbols(facts, identity)?;
        Ok(stage)
    }

    fn header_draft(
        report: &Report,
        facts: &Declarations,
        identity: &CrateIdentity,
        mode: ExportMode,
        body_transport: BodyTransport,
    ) -> Result<Self, &'static str> {
        require_export_mode(
            report,
            facts,
            identity,
            mode,
            body_transport == BodyTransport::Nodes,
        )?;
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
            nominal_transport: matches!(
                mode,
                ExportMode::Nominals | ExportMode::CompilerOperations
            ),
            compiler_operations: mode == ExportMode::CompilerOperations,
            body_transport,
            variants: Vec::new(),
            record_variants: false,
            role_installation_steps: facts
                .source_bindings
                .as_deref()
                .map_or(0, crate::sema::compiler_items::Catalog::binding_work),
        };
        if mode.feature_predicates() {
            stage.reserve(size_of::<
                crate::support::hash::HashMap<DefId, s::TraitApplicationRef>,
            >())?;
            stage.inputs.selected_methods = Some(Box::default());
        }
        stage.functions.exact = mode.exact();
        stage.functions.conversions = matches!(
            mode,
            ExportMode::Conversions | ExportMode::Nominals | ExportMode::CompilerOperations
        );
        stage.functions.callable_effects = stage.body_nodes()
            || matches!(
                mode,
                ExportMode::Callables
                    | ExportMode::Safety
                    | ExportMode::Conversions
                    | ExportMode::Nominals
            );
        stage.reserve(count.saturating_mul(size_of::<Option<DeclarationRef>>()))?;
        stage.definitions.resize(count, None);
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
    /// The cold column groups, allocated on first write with their box paid.
    pub(crate) fn cold(&mut self) -> Result<&mut s::ColdColumns, &'static str> {
        if self.rows.cold.is_none() {
            self.reserve(size_of::<s::ColdColumns>())?;
        }
        Ok(self.rows.cold_mut())
    }
    pub(crate) fn scratch(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.charge(1)?;
        self.scratch = self.scratch.max(bytes);
        if self.bytes.saturating_add(self.scratch) > crates::MAX_BYTES {
            return Err("source export retained and scratch budget exceeded");
        }
        Ok(())
    }
    /// The exported constructor declaration naming a local enum variant.
    pub(crate) fn variant(
        &mut self,
        variant: crate::sema::members::VariantId,
    ) -> Result<DeclarationRef, &'static str> {
        self.charge(1)?;
        self.variants
            .get(variant.0 as usize)
            .copied()
            .flatten()
            .ok_or("source variant is outside its exported enum header")
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

    fn declaration_parent(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owners: &mut [Option<DefId>],
        id: DefId,
        kind: s::DefinitionKind,
    ) -> Result<Option<DefId>, &'static str> {
        if Some(id) == report.resolution.crate_root {
            return Ok(None);
        }
        if kind == s::DefinitionKind::Variant {
            let variant = facts
                .members
                .constructor(id)
                .ok_or("source variant lacks its actual constructor")?;
            return Ok(Some(DefId(
                facts.members.variants[variant.0 as usize].owner.0,
            )));
        }
        let mut scope = report.resolution.definitions[id.0 as usize].scope;
        if owners.get(scope.0 as usize).copied().flatten() == Some(id) {
            scope = report
                .resolution
                .scopes
                .get(scope.0 as usize)
                .and_then(|scope| scope.parent)
                .ok_or("source declaration owns no actual parent scope")?;
        }
        find_owner(report, owners, scope, self).map(Some)
    }

    fn alias_headers(
        &mut self,
        facts: &Declarations,
        aliases: Vec<(DeclarationRef, DefId)>,
    ) -> Result<(), &'static str> {
        if !aliases.is_empty() {
            self.reserve(
                aliases
                    .len()
                    .saturating_mul(size_of::<s::TypeAliasHeader>()),
            )?;
            self.cold()?.type_aliases.reserve_exact(aliases.len());
        }
        for (declaration, definition) in aliases {
            let target = facts.definitions[definition.0 as usize]
                .ok_or("source alias lost its completed target")?;
            let context = canonical_nodes::Context {
                owner: declaration,
                frame: None,
                body: None,
            };
            let reference = self.ty(facts, target, context)?;
            self.cold()?.type_aliases.push(s::TypeAliasHeader {
                declaration,
                target: reference,
            });
            self.alias_normalization_rows(facts, definition, target, context)?;
        }
        Ok(())
    }

    fn alias_normalization_rows(
        &mut self,
        facts: &Declarations,
        definition: DefId,
        target: TypeId,
        context: canonical_nodes::Context,
    ) -> Result<(), &'static str> {
        let projected = facts.types.contains_projections(target) == Some(true);
        if facts
            .alias_requirements
            .source_target(definition, target, projected)
        {
            return Ok(());
        }
        let normalizations = facts.alias_requirements.source_normalizations(definition)?;
        self.charge(normalizations.len())?;
        self.reserve(
            normalizations
                .len()
                .saturating_mul(4 * size_of::<s::AliasNormalization>()),
        )?;
        for &(scope, ty) in normalizations {
            if scope != ProofScope::ROOT
                || !matches!(facts.types.kind(ty), Some(Kind::Projection(_)))
            {
                return Err("source alias requires a nonroot or nonprojection proof");
            }
            let target = self.ty(facts, ty, context)?;
            self.cold()?
                .alias_normalizations
                .push(s::AliasNormalization {
                    declaration: context.owner,
                    target,
                });
        }
        Ok(())
    }

    fn scoped_alias(
        &mut self,
        report: &Report,
        facts: &Declarations,
        id: DefId,
    ) -> Result<(), &'static str> {
        use crate::{syntax::item::GenericFlavor, types::arguments::Argument};
        self.charge(10)?;
        let error = "source alias formals or deferred proofs require their wider profile";
        let declaration = &report.resolution.definitions[id.0 as usize];
        let source = declaration.node.ok_or(error)?;
        let ast = &report.package.unit(source.file).configured.ast;
        let Syntax::Item(index) = ast.node(source.node).kind else {
            return Err(error);
        };
        let item = &ast.items[index as usize];
        let formals = ast.list(item.generics);
        let parameters = facts.generics.parameters.get(&id).ok_or(error)?;
        self.charge(formals.len())?;
        if formals.len() != parameters.len()
            || formals.is_empty()
            || declaration.namespace != Namespace::Type
            || facts.generics.parents.contains_key(&id)
            || !matches!(item.kind, ItemKind::TypeAlias { bounds, ty: Some(_) } if ast.list(bounds).is_empty())
        {
            return Err(error);
        }
        for (&formal, parameter) in formals.iter().zip(parameters) {
            let Syntax::Generic(index) = ast.node(formal).kind else {
                return Err(error);
            };
            let formal = &ast.generics[index as usize];
            if !matches!(
                (formal.flavor, parameter.argument),
                (GenericFlavor::Type, Argument::Type(_))
                    | (GenericFlavor::Const, Argument::Const(_))
            ) || formal.default.is_some()
                || parameter.default.is_some()
                || parameter.anonymous
            {
                return Err(error);
            }
        }
        let target = facts.definitions[id.0 as usize].ok_or(error)?;
        if facts.node(source).and_then(|node| node.ty) != Some(target)
            || facts.types.needs_region_context(target) != Some(false)
            || facts.types.contains_parameters(target) != Some(true)
            || facts
                .unreduced
                .definition(id)
                .is_some_and(|original| original != target)
        {
            return Err("source alias target has foreign, regional or unreduced inputs");
        }
        Ok(())
    }

    /// A closed transparent alias retains its completed canonical type. Only
    /// v14 also gives its public module name an independent declaration row.
    fn closed_alias(
        &mut self,
        report: &Report,
        facts: &Declarations,
        id: DefId,
    ) -> Result<(), &'static str> {
        self.charge(10)?;
        let declaration = report
            .resolution
            .definitions
            .get(id.0 as usize)
            .ok_or("source alias lost its actual declaration")?;
        if declaration.kind != DefKind::TypeAlias
            || declaration.namespace != Namespace::Type
            || !(matches!(declaration.access, Access::Within(_))
                || self.declaration_policies() && declaration.access == Access::Public)
            || facts.associated.owner(id).is_some()
            || facts
                .generics
                .parameters
                .get(&id)
                .is_some_and(|parameters| !parameters.is_empty())
            || facts
                .lifetimes
                .parameters
                .get(&id)
                .is_some_and(|parameters| !parameters.is_empty())
        {
            return Err("named, associated or generic alias transport requires its wider profile");
        }
        if declaration.access == Access::Public && facts.alias_requirements.contains(id) {
            return Err("source alias has deferred proof requirements");
        }
        let source = declaration
            .node
            .ok_or("source alias lost its actual source owner")?;
        let ty = facts
            .definitions
            .get(id.0 as usize)
            .copied()
            .flatten()
            .ok_or("source alias has no completed canonical type")?;
        let witness = facts.node(source).and_then(|node| node.ty);
        let completed = witness == Some(ty)
            || self.declaration_policies()
                && witness == Some(TypeId::UNIT)
                && matches!(declaration.access, Access::Within(_))
                && !is_module_scoped(report, declaration)
                && block_alias_target(report, facts, source) == Some(ty);
        if !completed
            || facts.types.contains_parameters(ty) != Some(false)
            || facts.types.needs_region_context(ty) != Some(false)
        {
            return Err("source alias changed its completed closed type or source witness");
        }
        Ok(())
    }
}

fn block_alias_target(report: &Report, facts: &Declarations, source: NodeKey) -> Option<TypeId> {
    // Body typing assigns the item statement a unit type. The RHS still
    // witnesses the completed canonical type of the alias declaration.
    let ast = &report.package.unit(source.file).configured.ast;
    let Syntax::Item(index) = ast.node(source.node).kind else {
        return None;
    };
    let ItemKind::TypeAlias { ty: Some(rhs), .. } = ast.items[index as usize].kind else {
        return None;
    };
    facts
        .node(NodeKey {
            node: rhs,
            ..source
        })?
        .ty
}

#[derive(Default)]
struct DeclarationLeaves {
    aliases: Vec<(DeclarationRef, DefId)>,
    trait_aliases: Vec<(DeclarationRef, DefId)>,
    statics: Vec<(DeclarationRef, DefId)>,
}

impl DeclarationLeaves {
    fn record(
        &mut self,
        stage: &mut Stage,
        definition: DefId,
        row: DeclarationRef,
        kind: s::DefinitionKind,
    ) -> Result<(), &'static str> {
        match kind {
            s::DefinitionKind::TypeAlias => {
                stage.scratch(
                    self.aliases
                        .len()
                        .saturating_add(1)
                        .saturating_mul(4 * size_of::<(DeclarationRef, DefId)>()),
                )?;
                self.aliases.push((row, definition));
            }
            s::DefinitionKind::TraitAlias => {
                stage.scratch(
                    self.trait_aliases
                        .len()
                        .saturating_add(1)
                        .saturating_mul(4 * size_of::<(DeclarationRef, DefId)>()),
                )?;
                self.trait_aliases.push((row, definition));
            }
            s::DefinitionKind::Static => {
                stage.scratch(
                    self.statics
                        .len()
                        .saturating_add(1)
                        .saturating_mul(4 * size_of::<(DeclarationRef, DefId)>()),
                )?;
                self.statics.push((row, definition));
            }
            _ => {}
        }
        Ok(())
    }
}

/// A trait implementation's type definition; version thirteen exports it as an
/// associated declaration whose header is its definition row.
fn implementation_result(report: &Report, facts: &Declarations, id: DefId) -> bool {
    report.resolution.definitions[id.0 as usize].kind == DefKind::TypeAlias
        && facts
            .associated
            .owner(id)
            .and_then(|owner| facts.associated.get(owner))
            .is_some_and(|owner| {
                matches!(
                    owner.kind,
                    OwnerKind::Impl {
                        trait_type: Some(_),
                        ..
                    }
                )
            })
}

fn require_foreign_declaration(
    report: &Report,
    facts: &Declarations,
    id: DefId,
) -> Result<(), &'static str> {
    use crate::{
        syntax::{ast::Kind, item::ItemKind},
        types::ItemId,
    };
    let definition = &report.resolution.definitions[id.0 as usize];
    let source = definition
        .node
        .ok_or("source foreign type has no actual declaration")?;
    let block = report
        .resolution
        .foreign_owners
        .get(&id)
        .copied()
        .filter(|block| report.resolution.foreign_blocks.contains(block))
        .ok_or("source foreign type has no actual external block")?;
    let item = |location: crate::resolve::unit::NodeKey| {
        let ast = &report.package.unit(location.file).configured.ast;
        let Kind::Item(index) = ast.node(location.node).kind else {
            return None;
        };
        ast.items.get(index as usize).map(|item| item.kind)
    };
    let cell = facts.definitions.get(id.0 as usize).copied().flatten();
    if definition.namespace != Namespace::Type
        || item(source) != Some(ItemKind::ForeignType)
        || !matches!(item(block), Some(ItemKind::ExternBlock { .. }))
        || cell != facts.node(source).and_then(|node| node.ty)
        || cell.and_then(|ty| facts.types.kind(ty))
            != Some(crate::types::Kind::Foreign(ItemId(id.0)))
    {
        return Err("source foreign type changed its actual opaque declaration");
    }
    Ok(())
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
        // Macro definitions are expansion facts: public ones travel in the
        // separate macro table, and none of them has a semantic row.
        DefKind::Primitive | DefKind::BuiltinMacro | DefKind::BuiltinTrait | DefKind::Macro => {
            return Ok(None);
        }
        _ => return Err("source declaration kind is outside the initial source profile"),
    }))
}

fn scoped_module_alias(
    report: &Report,
    facts: &Declarations,
    definition: &Definition,
    id: DefId,
) -> bool {
    definition.kind == DefKind::TypeAlias
        && matches!(definition.access, Access::Public | Access::Within(_))
        && facts.associated.owner(id).is_none()
        && facts
            .generics
            .parameters
            .get(&id)
            .is_some_and(|parameters| !parameters.is_empty())
        && is_module_scoped(report, definition)
}

fn is_module_scoped(report: &Report, definition: &Definition) -> bool {
    let module = report.resolution.scopes[definition.scope.0 as usize].module;
    report.resolution.module_scopes.get(module.0 as usize) == Some(&definition.scope)
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
        if stage.declaration_policies()
            && matches!(definition.kind, DefKind::TypeAlias | DefKind::TraitAlias)
        {
            let alias = DefId(u32::try_from(index).map_err(|_| "source owner identity overflow")?);
            if definition.kind == DefKind::TraitAlias
                || scoped_module_alias(report, facts, definition, alias)
            {
                let source = definition
                    .node
                    .ok_or("source alias has no actual source owner")?;
                let module = report.resolution.scopes[definition.scope.0 as usize].module;
                let scope = *report
                    .resolution
                    .node_scopes
                    .get(&(module, source.node))
                    .ok_or("source alias has no actual lexical scope")?;
                owners[scope.0 as usize] = Some(alias);
            }
        }
        if let Some(scope) = definition.members
            && definition.kind != DefKind::ExternCrate
        {
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
            OwnerKind::ImportedImpl(_) | OwnerKind::ImportedInherent(_) => {
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
    if stage.declaration_policies() {
        stage.constant_lexical_owners(report, facts, &mut owners)?;
        stage.require_contextual_bindings(report, facts, &mut owners)?;
        authored_traits::require_role_transport(report, facts, true, true, true)?;
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
        associated_types: Vec::new(),
        cold: None,
        projections: Vec::new(),
        opaque_futures: Vec::new(),
        compiler_operations: Vec::new(),
        nominals: Vec::new(),
        fields: Vec::new(),
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
        exact_callables: Vec::new(),
    }
}

fn require_export_mode(
    report: &Report,
    facts: &Declarations,
    identity: &CrateIdentity,
    mode: ExportMode,
    languages: bool,
) -> Result<(), &'static str> {
    // The bring-up profile publishes without the facts these fences protect.
    waiving(facts.require_declaration_policy_transport(
        report,
        languages && mode == ExportMode::CompilerOperations,
    ))?;
    waiving(facts.require_incoherent_target_transport(
        report,
        languages && mode == ExportMode::CompilerOperations,
    ))?;
    waiving(facts.require_dispatch_policy_transport(
        report,
        languages && mode == ExportMode::CompilerOperations,
    ))?;
    if !languages || mode != ExportMode::CompilerOperations {
        waiving(facts.require_optimization_transport(report))?;
    }
    waiving(crate::sema::coherence_policy::require_transport(facts))?;
    if !languages || mode != ExportMode::CompilerOperations {
        waiving(super::export::require_local_simd_limit_transport(
            report, facts,
        ))?;
    }
    if let Some(producer) = report.lint_producer() {
        if !languages || mode != ExportMode::CompilerOperations {
            waiving(
                producer
                    .require_deprecation_transport(&report.package)
                    .map_err(|error| error.message),
            )?;
        }
        waiving(
            producer
                .require_impl_annotation_transport(
                    &report.package,
                    mode.feature_predicates(),
                    languages && mode == ExportMode::CompilerOperations,
                )
                .map_err(|error| error.message),
        )?;
    }
    require_provider_identity(report, identity)?;
    if facts.members.has_contextual_fields() {
        return Err("contextual aggregate field metadata is not implemented");
    }
    if report.package.library_policy().is_some() && !mode.availability() {
        return Err("source stability contracts are outside this metadata profile");
    }
    if mode.availability() {
        let library = report.package.library_policy();
        let complete = if matches!(
            mode,
            ExportMode::Conversions | ExportMode::Nominals | ExportMode::CompilerOperations
        ) {
            library.is_none_or(|library| library.inventory().is_some())
        } else {
            library
                .and_then(crate::resolve::stability::Library::staged_inventory)
                .is_some()
        };
        if !complete {
            return Err("source availability export requires its completed actual library policy");
        }
    }
    if !languages || mode != ExportMode::CompilerOperations {
        require_legacy_name_provenance(report, languages)?;
    }
    facts
        .source_bindings
        .as_deref()
        .ok_or("source export has no actual role catalog")?
        .require_provider_report(report)?;
    if !languages || mode != ExportMode::CompilerOperations {
        authored_traits::require_role_transport(
            report,
            facts,
            matches!(
                mode,
                ExportMode::Conversions | ExportMode::Nominals | ExportMode::CompilerOperations
            ),
            mode == ExportMode::CompilerOperations,
            languages,
        )?;
    }
    Ok(())
}

fn require_provider_identity(
    report: &Report,
    identity: &CrateIdentity,
) -> Result<(), &'static str> {
    let provider = report
        .package
        .source_core()
        .ok_or("source export requires its actual own-provider policy")?;
    if provider.current != *identity || provider.provider != *identity {
        return Err("source export identity differs from its actual provider");
    }
    Ok(())
}

const NAME_PROVENANCE: &str = "source macro/semantic-name provenance metadata is not implemented";

fn require_legacy_name_provenance(report: &Report, languages: bool) -> Result<(), &'static str> {
    let bindings = &report.resolution.bindings;
    let marked = if languages {
        report.package.marks.has_item_contexts() || bindings.has_contextual_locals()
    } else {
        report.package.marks.context_count() != 1
    };
    if marked || bindings.has_contextual_items() {
        waiving(Err(NAME_PROVENANCE))
    } else {
        Ok(())
    }
}

impl Stage {
    fn require_contextual_bindings(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owners: &mut [Option<DefId>],
    ) -> Result<(), &'static str> {
        // Originals reconstruct checked slots and block targets. An item
        // name can use that path only under its real lexical or trait owner.
        for (scope, name, namespace, binding) in report.resolution.bindings.contextual_items() {
            self.charge(1)?;
            let definition = binding.target;
            let item = report
                .resolution
                .definitions
                .get(definition.0 as usize)
                .ok_or("source contextual binding lost its actual declaration")?;
            if binding.ambiguous
                || binding.origin != Origin::Direct
                || item.scope != scope
                || item.name != name
                || item.namespace != namespace
                || item.node.is_none()
                || binding.access != item.access
                || !matches!(item.access, Access::Within(_))
            {
                return waiving(Err(NAME_PROVENANCE));
            }
            if contextual_slot(facts, definition, item.kind) {
                continue;
            }
            if item.kind == DefKind::Function && facts.associated.owner(definition).is_none() {
                let owner = find_owner(report, owners, scope, self)?;
                if owner != definition
                    && report.resolution.definitions[owner.0 as usize].kind == DefKind::Function
                {
                    continue;
                }
            }
            return waiving(Err(NAME_PROVENANCE));
        }
        Ok(())
    }
}

fn contextual_slot(facts: &Declarations, target: DefId, kind: DefKind) -> bool {
    match kind {
        // An actual formal owns a typed edge in its function or type scheme.
        DefKind::Generic => true,
        // A trait impl uses the checked owner and member mapping on import.
        DefKind::Function | DefKind::TypeAlias => facts
            .associated
            .owner(target)
            .and_then(|owner| facts.associated.get(owner))
            .is_some_and(|owner| {
                owner.source.is_some()
                    && matches!(
                        owner.kind,
                        OwnerKind::Impl {
                            trait_type: Some(_),
                            ..
                        }
                    )
                    && facts
                        .generics
                        .implementations
                        .as_ref()
                        .is_some_and(|registry| registry.is_validated(owner.self_definition))
            }),
        _ => false,
    }
}
