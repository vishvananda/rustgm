//! Allocate canonical external modules before binding any dependency reexports.
use super::{
    Access, Binding, DefKind, Error, ExternalDef, ExternalKind, Imported, Interface, ModuleId,
    Namespace, Origin, Resolver, ScopeId, WireNamespace, error,
};

impl Resolver<'_> {
    pub(super) fn reserve_public_modules(
        &mut self,
        root: ScopeId,
        artifact: &Interface,
        imported: &mut Imported,
    ) -> Result<(), Error> {
        let Some(names) = artifact.public_names() else {
            return Ok(());
        };
        for module in names.modules().iter().filter(|m| !m.path().is_empty()) {
            let external = ExternalDef::new(
                artifact.identity().clone(),
                module.path().to_vec(),
                ExternalKind::Module,
            )
            .map_err(error)?;
            let definition = self.import_unbound_item(
                root,
                &external,
                DefKind::Module,
                Access::Public,
                &mut imported.definitions,
            )?;
            let scope = self.new_scope(None, ModuleId(0), true)?;
            self.facts.definitions[definition.0 as usize].members = Some(scope);
        }
        Ok(())
    }
    pub(super) fn bind_public_modules(
        &mut self,
        artifact: &Interface,
        imported: &Imported,
    ) -> Result<(), Error> {
        let Some(names) = artifact.public_names() else {
            return Ok(());
        };
        for module in names.modules() {
            let external = ExternalDef::new(
                artifact.identity().clone(),
                module.path().to_vec(),
                ExternalKind::Module,
            )
            .map_err(error)?;
            let definition = imported
                .definition(&external)
                .ok_or_else(|| error("missing public metadata module identity"))?;
            let scope = self.facts.definitions[definition.0 as usize]
                .members
                .ok_or_else(|| error("missing public metadata module scope"))?;
            for binding in module.bindings() {
                let target = imported
                    .definition(binding.target())
                    .ok_or_else(|| error("missing public metadata binding target"))?;
                if self.facts.definitions[target.0 as usize].access != Access::Public {
                    return Err(error(
                        "public metadata binding targets a private definition",
                    ));
                }
                let name = self.package.names.intern(binding.name()).map_err(error)?;
                let namespace = match binding.target().namespace() {
                    WireNamespace::Type => Namespace::Type,
                    WireNamespace::Value => Namespace::Value,
                };
                let frame = &mut self.facts.scopes[scope.0 as usize];
                let key = (name, namespace);
                if frame
                    .bindings
                    .insert(
                        key,
                        Binding {
                            target,
                            access: Access::Public,
                            origin: Origin::Direct,
                            ambiguous: false,
                        },
                    )
                    .is_some()
                {
                    return Err(error("duplicate imported public metadata binding"));
                }
                frame.order.push(key);
            }
        }
        Ok(())
    }
}
