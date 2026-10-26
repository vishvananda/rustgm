//! External metadata keeps its origin while local scopes receive canonical IDs.
mod original_declarations;
mod public_names;
#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "private namespace transaction awaits qualified module-contract publication; public profile guards remain closed"
    )
)]
pub(crate) use original_declarations::{OwnedNamespaceCatalog, PendingNamespace};
mod consumer_inputs;
#[path = "crates/dormant_catalog.rs"]
pub(crate) mod unpaid_provided;
use super::{
    index::Resolver,
    model::{Access, Binding, DefId, DefKind, Definition, Namespace, Origin, ScopeId},
    package::{Error, ModuleId},
    unit::FileId,
};
use crate::support::crates::{
    Constant, Dependencies, ExternalDef, ExternalKind, Function, Interface,
    Namespace as WireNamespace, Record, Static,
};
use crate::support::hash::HashMap;
pub(crate) use consumer_inputs::{ConsumerInput, ConsumerRequest, ConsumerReservation};
pub(in crate::resolve) use consumer_inputs::{ReservationSeed, reserve_names};
pub(crate) use original_declarations::Work as ReservationWork;
use std::sync::Arc;

pub struct Imported {
    origins: super::body_origin::CatalogSlot,
    generic_work: Option<Box<original_declarations::Work>>,
    dependencies: Arc<Dependencies>,
    definitions: HashMap<ExternalDef, DefId>,
    functions: Vec<(DefId, usize, usize)>,
    constants: Vec<(DefId, usize, usize)>,
    records: Vec<(DefId, usize, usize)>,
    statics: Vec<(DefId, usize, usize)>,
}
impl Imported {
    pub(crate) fn original_reservation_budget(&self) -> (usize, usize) {
        self.generic_work
            .as_deref()
            .map_or((0, 0), |work| (work.steps, work.retained))
    }
    fn new(dependencies: Arc<Dependencies>) -> Self {
        Self {
            origins: super::body_origin::CatalogSlot::default(),
            generic_work: None,
            dependencies,
            definitions: HashMap::default(),
            functions: Vec::new(),
            constants: Vec::new(),
            records: Vec::new(),
            statics: Vec::new(),
        }
    }
    pub(crate) fn origin_catalog(
        &self,
    ) -> Result<Option<std::rc::Rc<super::body_origin::Catalog>>, &'static str> {
        self.origins.shared()
    }
    /// Resolve only a coordinate authenticated by this completed import catalog.
    /// # Errors
    /// Rejects absent, foreign and stale artifact/declaration coordinates.
    pub fn generic_definition(
        &self,
        owner: &crate::support::crates::CrateIdentity,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
    ) -> Result<DefId, &'static str> {
        self.origins
            .get()
            .ok_or("imported body catalog is absent")?
            .definition_at(owner, declaration)
    }
    /// This identifies a typed original location, never a checked executable.
    /// # Errors
    /// Rejects missing artifacts and body ordinals outside their exact owner.
    pub fn generic_body(
        &self,
        owner: &crate::support::crates::CrateIdentity,
        body: crate::support::crates::owner_catalog::BodyRef,
    ) -> Result<super::body_origin::ImportedBodyId, &'static str> {
        self.origins
            .get()
            .ok_or("imported body catalog is absent")?
            .body_at(owner, body)
    }
    /// Only declarations from completed authenticated generic artifacts are
    /// eligible for the semantic header-readiness handshake. Bodies stay unproved.
    pub(crate) fn generic_definitions(&self) -> impl Iterator<Item = Result<DefId, &'static str>> {
        self.dependencies
            .artifacts()
            .iter()
            .filter_map(|artifact| {
                artifact.generic_payload().map(|payload| {
                    let declarations = match payload.view() {
                        crate::support::crates::generics::PayloadView::Legacy(rows) => {
                            rows.declarations.len()
                        }
                        crate::support::crates::generics::PayloadView::Source(rows) => {
                            rows.descriptor().rows().declarations.len()
                        }
                    };
                    (artifact.identity(), declarations)
                })
            })
            .flat_map(move |(owner, declarations)| {
                (0..declarations).map(move |index| {
                    let index = u32::try_from(index)
                        .map_err(|_| "imported generic declaration ordinal overflow")?;
                    self.generic_definition(
                        owner,
                        crate::support::crates::owner_catalog::DeclarationRef(index),
                    )
                })
            })
    }
    #[must_use]
    pub fn dependencies(&self) -> &Dependencies {
        &self.dependencies
    }
    /// Share this exact immutable imported inventory; no decoding or new owner
    /// table is performed by a semantic source-client view.
    pub(crate) fn shared_dependencies(&self) -> Arc<Dependencies> {
        Arc::clone(&self.dependencies)
    }
    #[must_use]
    pub fn definition(&self, external: &ExternalDef) -> Option<DefId> {
        self.definitions.get(external).copied()
    }
    pub fn definitions(&self) -> impl Iterator<Item = (&ExternalDef, DefId)> {
        self.definitions
            .iter()
            .map(|(external, &id)| (external, id))
    }
    pub fn functions(&self) -> impl Iterator<Item = (DefId, &Function)> {
        self.functions
            .iter()
            .map(|&(definition, artifact, function)| {
                (
                    definition,
                    &self.dependencies.artifacts()[artifact].functions()[function],
                )
            })
    }
    pub fn records(&self) -> impl Iterator<Item = (DefId, &Record)> {
        self.records.iter().map(|&(definition, artifact, record)| {
            (
                definition,
                &self.dependencies.artifacts()[artifact].records()[record],
            )
        })
    }
    pub fn statics(&self) -> impl Iterator<Item = (DefId, &Static)> {
        self.statics
            .iter()
            .filter_map(|&(definition, artifact, index)| {
                self.dependencies.artifacts()[artifact]
                    .storage()
                    .map(|storage| (definition, &storage.statics()[index]))
            })
    }
    pub fn constants(&self) -> impl Iterator<Item = (DefId, &Constant)> {
        self.constants
            .iter()
            .map(|&(definition, artifact, constant)| {
                (
                    definition,
                    &self.dependencies.artifacts()[artifact].constants()[constant],
                )
            })
    }
}
fn error(message: &'static str) -> Error {
    Error::at(FileId(0), 0, message)
}
impl Resolver<'_> {
    pub(super) fn import_crates(&mut self, dependencies: Arc<Dependencies>) -> Result<(), Error> {
        let external = self.new_scope(None, ModuleId(0), true)?;
        let artifacts = Arc::clone(&dependencies);
        let mut imported = Imported::new(dependencies);
        for (artifact_index, artifact) in artifacts.artifacts().iter().enumerate() {
            let scope = self.reserve_artifact_root(artifact, &mut imported)?;
            if artifact.public_names().is_some() {
                self.reserve_public_modules(scope, artifact, &mut imported)?;
            }
            self.import_original_declarations(scope, artifact, &mut imported)?;
            if artifact.nominals().is_some() || artifact.public_names().is_some() {
                self.import_nominal_items(scope, artifact, artifact_index, &mut imported)?;
                continue;
            }
            for (function_index, function) in artifact.functions().iter().enumerate() {
                let definition = self.import_item(
                    scope,
                    function.definition(),
                    DefKind::Function,
                    &mut imported.definitions,
                )?;
                imported
                    .functions
                    .push((definition, artifact_index, function_index));
            }
            for (constant_index, constant) in artifact.constants().iter().enumerate() {
                let definition = self.import_item(
                    scope,
                    constant.definition(),
                    DefKind::Const,
                    &mut imported.definitions,
                )?;
                imported
                    .constants
                    .push((definition, artifact_index, constant_index));
            }
        }
        for artifact in artifacts.artifacts() {
            self.bind_public_modules(artifact, &imported)?;
        }
        self.bind_crate_aliases(external, &imported)?;
        self.facts.extern_scope = Some(external);
        imported.origins.freeze().map_err(error)?;
        self.facts.crates = Some(Box::new(imported));
        Ok(())
    }
    fn bind_crate_aliases(&mut self, external: ScopeId, imported: &Imported) -> Result<(), Error> {
        for alias in imported.dependencies.aliases() {
            let identity =
                ExternalDef::new(alias.identity().clone(), Vec::new(), ExternalKind::Module)
                    .map_err(error)?;
            let target = imported
                .definitions
                .get(&identity)
                .copied()
                .ok_or_else(|| error("missing imported crate root"))?;
            let name = self.package.names.intern(alias.alias()).map_err(error)?;
            self.facts
                .bindings
                .insert(
                    external,
                    name,
                    Namespace::Type,
                    Binding {
                        target,
                        access: Access::Public,
                        origin: Origin::Direct,
                        ambiguous: false,
                    },
                )
                .map_err(error)?;
        }
        Ok(())
    }
    fn reserve_artifact_root(
        &mut self,
        artifact: &Interface,
        imported: &mut Imported,
    ) -> Result<ScopeId, Error> {
        let scope = self.new_scope(None, ModuleId(0), true)?;
        let name = self
            .package
            .names
            .intern(artifact.identity().name())
            .map_err(error)?;
        let root = DefId(
            u32::try_from(self.facts.definitions.len())
                .map_err(|_| error("too many imported definitions"))?,
        );
        self.facts.definitions.push(Definition {
            name,
            namespace: Namespace::Type,
            kind: DefKind::Module,
            scope,
            node: None,
            access: Access::Public,
            members: Some(scope),
        });
        imported.definitions.insert(
            ExternalDef::new(
                artifact.identity().clone(),
                Vec::new(),
                ExternalKind::Module,
            )
            .map_err(error)?,
            root,
        );
        Ok(scope)
    }
    fn import_nominal_items(
        &mut self,
        scope: ScopeId,
        artifact: &Interface,
        artifact_index: usize,
        imported: &mut Imported,
    ) -> Result<(), Error> {
        let mut entries: Vec<_> =
            artifact
                .functions()
                .iter()
                .enumerate()
                .map(|(index, entry)| (entry.definition(), DefKind::Function, index, true))
                .chain(
                    artifact
                        .constants()
                        .iter()
                        .enumerate()
                        .map(|(index, entry)| (entry.definition(), DefKind::Const, index, true)),
                )
                .chain(artifact.records().iter().enumerate().map(|(index, entry)| {
                    (entry.definition(), DefKind::Struct, index, entry.public())
                }))
                .chain(
                    artifact
                        .storage()
                        .into_iter()
                        .flat_map(|storage| storage.statics().iter().enumerate())
                        .map(|(index, entry)| {
                            (entry.definition(), DefKind::Static, index, entry.public())
                        }),
                )
                .collect();
        entries.sort_unstable_by_key(|&(identity, ..)| identity);
        for (identity, kind, index, public) in entries {
            let definition = if public && artifact.public_names().is_none() {
                self.import_item(scope, identity, kind, &mut imported.definitions)?
            } else {
                self.import_unbound_item(
                    scope,
                    identity,
                    kind,
                    if public {
                        Access::Public
                    } else {
                        Access::ExternalPrivate
                    },
                    &mut imported.definitions,
                )?
            };
            match kind {
                DefKind::Function => {
                    imported.functions.push((definition, artifact_index, index));
                }
                DefKind::Const => {
                    imported.constants.push((definition, artifact_index, index));
                }
                DefKind::Struct => {
                    imported.records.push((definition, artifact_index, index));
                }
                DefKind::Static => {
                    imported.statics.push((definition, artifact_index, index));
                }
                _ => return Err(error("invalid nominal artifact declaration")),
            }
        }
        Ok(())
    }
    fn import_unbound_item(
        &mut self,
        scope: ScopeId,
        external: &ExternalDef,
        kind: DefKind,
        access: Access,
        definitions: &mut HashMap<ExternalDef, DefId>,
    ) -> Result<DefId, Error> {
        let spelling = external
            .path()
            .last()
            .ok_or_else(|| error("missing hidden metadata name"))?;
        let name = self.package.names.intern(spelling).map_err(error)?;
        let id = DefId(
            u32::try_from(self.facts.definitions.len())
                .map_err(|_| error("too many imported definitions"))?,
        );
        self.facts.definitions.push(Definition {
            name,
            namespace: match external.namespace() {
                WireNamespace::Type => Namespace::Type,
                WireNamespace::Value => Namespace::Value,
            },
            kind,
            scope,
            node: None,
            access,
            members: None,
        });
        if definitions.insert(external.clone(), id).is_some() {
            return Err(error("duplicate hidden metadata identity"));
        }
        Ok(id)
    }
    fn import_item(
        &mut self,
        mut scope: ScopeId,
        external: &ExternalDef,
        kind: DefKind,
        definitions: &mut HashMap<ExternalDef, DefId>,
    ) -> Result<DefId, Error> {
        let path = external.path();
        for (index, component) in path.iter().enumerate() {
            let name = self.package.names.intern(component).map_err(error)?;
            let last = index + 1 == path.len();
            let namespace = if last {
                match external.namespace() {
                    WireNamespace::Type => Namespace::Type,
                    WireNamespace::Value => Namespace::Value,
                }
            } else {
                Namespace::Type
            };
            if let Some(binding) = self.facts.bindings.get(scope, name, namespace) {
                scope = self.facts.definitions[binding.target.0 as usize]
                    .members
                    .ok_or_else(|| error("conflicting imported module path"))?;
                continue;
            }
            let members = if last {
                None
            } else {
                Some(self.new_scope(None, ModuleId(0), true)?)
            };
            let definition = self.define(
                Definition {
                    name,
                    namespace,
                    kind: if last { kind } else { DefKind::Module },
                    scope,
                    node: None,
                    access: Access::Public,
                    members,
                },
                false,
            )?;
            let identity = if last {
                external.clone()
            } else {
                ExternalDef::new(
                    external.owner().clone(),
                    path[..=index].to_vec(),
                    ExternalKind::Module,
                )
                .map_err(error)?
            };
            definitions.insert(identity, definition);
            if last {
                return Ok(definition);
            }
            scope = members.ok_or_else(|| error("missing imported module scope"))?;
        }
        Err(error("missing imported value path"))
    }
}
