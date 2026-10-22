//! The primitive module exposes aliases to the existing canonical type identities.
use super::{Access, Binding, Context, DefKind, Error, Namespace, Resolver};
use crate::{
    resolve::{
        lookup::{Lookup, LookupMode},
        model::Origin,
        names::NameId,
        package::ModuleId,
    },
    syntax::ast::NodeId,
};
impl Resolver<'_> {
    pub(super) fn prepare_primitive_module(
        &mut self,
        c: Context,
        node: NodeId,
        absolute: bool,
        path: &[NameId],
    ) -> Result<(), Error> {
        // Expansion may poll ordinary imports before the canonical primitive
        // index exists. Leave that import pending; ordinary collection retries
        // it after publishing the existing primitive identities.
        if self.facts.primitives.is_empty() {
            return Ok(());
        }
        let Some(index) = path
            .iter()
            .position(|name| self.package.names.get(*name) == "primitive")
        else {
            return Ok(());
        };
        if index == 0 {
            return Ok(());
        }
        let Some(library) = self.facts.builtin_traits.library else {
            return Ok(());
        };
        let Lookup::Found(root) = self.lookup_path(
            c,
            node,
            absolute,
            &path[..index],
            LookupMode::Import(Namespace::Type),
            None,
        )?
        else {
            return Ok(());
        };
        if self.facts.definitions[root.target.0 as usize].members != Some(library) {
            return Ok(());
        }
        let name = path[index];
        if self.facts.scopes[library.0 as usize]
            .bindings
            .contains_key(&(name, Namespace::Type))
        {
            return Ok(());
        }
        let scope = self.new_scope(None, ModuleId(0), true)?;
        self.builtin_definition(
            Context {
                scope: library,
                ..c
            },
            node,
            "primitive",
            DefKind::Module,
            Some(scope),
        )?;
        let mut primitives: Vec<_> = self
            .facts
            .primitives
            .iter()
            .map(|(&name, &target)| (name, target))
            .collect();
        primitives.sort_by_key(|(_, target)| target.0);
        for (name, target) in primitives {
            let key = (name, Namespace::Type);
            let scope = &mut self.facts.scopes[scope.0 as usize];
            scope.bindings.insert(
                key,
                Binding {
                    target,
                    access: Access::Public,
                    origin: Origin::Direct,
                    ambiguous: false,
                },
            );
            scope.order.push(key);
        }
        Ok(())
    }
}
