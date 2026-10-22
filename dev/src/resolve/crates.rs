//! External metadata keeps its origin while local scopes receive canonical IDs.
mod public_names;
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
use std::{collections::HashMap, sync::Arc};

pub struct Imported {
    dependencies: Arc<Dependencies>,
    definitions: HashMap<ExternalDef, DefId>,
    functions: Vec<(DefId, usize, usize)>,
    constants: Vec<(DefId, usize, usize)>,
    records: Vec<(DefId, usize, usize)>,
    statics: Vec<(DefId, usize, usize)>,
}
impl Imported {
    #[must_use]
    pub fn dependencies(&self) -> &Dependencies {
        &self.dependencies
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
        let mut imported = Imported {
            dependencies,
            definitions: HashMap::new(),
            functions: Vec::new(),
            constants: Vec::new(),
            records: Vec::new(),
            statics: Vec::new(),
        };
        for (artifact_index, artifact) in artifacts.artifacts().iter().enumerate() {
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
            if artifact.public_names().is_some() {
                self.reserve_public_modules(scope, artifact, &mut imported)?;
            }
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
            let key = (name, Namespace::Type);
            let scope = &mut self.facts.scopes[external.0 as usize];
            scope.order.push(key);
            scope.bindings.insert(
                key,
                Binding {
                    target,
                    access: Access::Public,
                    origin: Origin::Direct,
                    ambiguous: false,
                },
            );
        }
        self.facts.extern_scope = Some(external);
        self.facts.crates = Some(Box::new(imported));
        Ok(())
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
            if let Some(binding) = self.facts.scopes[scope.0 as usize]
                .bindings
                .get(&(name, namespace))
            {
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
