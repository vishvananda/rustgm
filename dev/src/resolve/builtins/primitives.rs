//! The primitive module exposes aliases to the existing canonical type identities.
use super::{Access, Binding, Context, DefKind, Error, Namespace, Resolver};
use crate::{
    resolve::{
        lookup::{Lookup, LookupMode},
        model::Origin,
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
        path: crate::resolve::paths::Identified<'_>,
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
            path.prefix(index),
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
        if self.facts.bindings.contains(library, name, Namespace::Type) {
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
            // The flat table records each key's first-binding order.
            self.facts
                .bindings
                .insert(
                    scope,
                    name,
                    Namespace::Type,
                    Binding {
                        target,
                        access: Access::Public,
                        origin: Origin::Direct,
                        ambiguous: false,
                    },
                )
                .map_err(|message| self.error(c, node, message))?;
        }
        Ok(())
    }
}
