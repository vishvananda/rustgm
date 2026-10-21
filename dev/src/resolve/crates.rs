//! External metadata keeps its origin while local scopes receive canonical IDs.
use super::{
    index::Resolver,
    model::{Access, Binding, DefId, DefKind, Definition, Namespace, Origin, ScopeId},
    package::{Error, ModuleId},
    unit::FileId,
};
use crate::support::crates::{Dependencies, ExternalDef, ExternalKind, Function};
use std::{collections::HashMap, sync::Arc};

pub struct Imported {
    dependencies: Arc<Dependencies>,
    definitions: HashMap<ExternalDef, DefId>,
    functions: Vec<(DefId, usize, usize)>,
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
}
fn error(message: &'static str) -> Error {
    Error::at(FileId(0), 0, message)
}
impl Resolver<'_> {
    pub(super) fn import_crates(&mut self, dependencies: Arc<Dependencies>) -> Result<(), Error> {
        let external = self.new_scope(None, ModuleId(0), true)?;
        let mut imported = Imported {
            dependencies,
            definitions: HashMap::new(),
            functions: Vec::new(),
        };
        for (artifact_index, artifact) in imported.dependencies.artifacts().iter().enumerate() {
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
            for (function_index, function) in artifact.functions().iter().enumerate() {
                let definition =
                    self.import_function(scope, function, &mut imported.definitions)?;
                imported
                    .functions
                    .push((definition, artifact_index, function_index));
            }
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
    fn import_function(
        &mut self,
        mut scope: ScopeId,
        function: &Function,
        definitions: &mut HashMap<ExternalDef, DefId>,
    ) -> Result<DefId, Error> {
        let path = function.definition().path();
        for (index, component) in path.iter().enumerate() {
            let name = self.package.names.intern(component).map_err(error)?;
            let last = index + 1 == path.len();
            let namespace = if last {
                Namespace::Value
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
                    kind: if last {
                        DefKind::Function
                    } else {
                        DefKind::Module
                    },
                    scope,
                    node: None,
                    access: Access::Public,
                    members,
                },
                false,
            )?;
            let identity = if last {
                function.definition().clone()
            } else {
                ExternalDef::new(
                    function.definition().owner().clone(),
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
        Err(error("missing imported function path"))
    }
}
