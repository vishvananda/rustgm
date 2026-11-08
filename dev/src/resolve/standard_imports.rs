//! A `no_std` source client receives rustc's standard library imports. Its
//! injected `core` extern crate is its `--extern core=` dependency or else the
//! authenticated provider, nameable at the crate root in Rust 2015, and its
//! glob prelude is that crate's `prelude::rust_20xx` module for the client's
//! own edition. Injected `core` carries no `macro_use`: provider macros come
//! through that prelude or a path.
use super::{
    index::Resolver,
    model::{Access, Binding, DefId, DefKind, Namespace, Origin, ScopeId},
    package::Error,
    unit::FileId,
};
use crate::support::{
    crates::{ExternalDef, ExternalKind},
    edition::Edition,
};

fn error(message: &'static str) -> Error {
    Error::at(FileId(0), 0, message)
}

/// The injected `core` root and the edition prelude module selected from it.
#[derive(Clone, Copy)]
pub(super) struct Injected {
    pub(super) core: DefId,
    pub(super) prelude: DefId,
    pub(super) scope: ScopeId,
}

impl Resolver<'_> {
    pub(super) fn inject_core(&mut self) -> Result<(), Error> {
        let Some(client) = self.package.source_client() else {
            return Ok(());
        };
        let provider = client.provider().clone();
        let unit = self
            .package
            .units
            .first()
            .ok_or_else(|| error("source client lost its crate root"))?;
        let no_core = unit
            .word_attribute(Some(unit.configured.ast.root), "no_core")
            .map_err(|e| Error::at(FileId(0), e.offset, e.message))?;
        if no_core || !self.package.no_std {
            return Ok(());
        }
        let module = match unit.edition() {
            Edition::Rust2015 => "rust_2015",
            Edition::Rust2018 => "rust_2018",
            Edition::Rust2021 => "rust_2021",
            Edition::Rust2024 => "rust_2024",
        };
        let root = self.provider_root(&provider)?;
        let prelude = self.member_module(self.module_scope(root)?, "prelude")?;
        let selected = self.member_module(self.module_scope(prelude)?, module)?;
        let scope = self.module_scope(selected)?;
        let owner = if self.facts.introductions.item_scopes.is_none() {
            size_of::<super::container_scopes::ItemScopes>()
        } else {
            0
        };
        if let Some(context) = self.package.macro_context().map_err(error)? {
            context
                .contracts()
                .charge(2, size_of::<Injected>() + owner)
                .map_err(error)?;
        }
        self.facts
            .introductions
            .item_scopes
            .get_or_insert_with(Box::default)
            .injected = Some(Injected {
            core: root,
            prelude: selected,
            scope,
        });
        Ok(())
    }

    fn member_module(&mut self, scope: ScopeId, name: &str) -> Result<DefId, Error> {
        let name = self.package.names.intern(name).map_err(error)?;
        self.facts
            .bindings
            .get(scope, name, Namespace::Type)
            .filter(|binding| binding.access == Access::Public && !binding.ambiguous)
            .map(|binding| binding.target)
            .ok_or_else(|| error("no_std source client's core provider lacks its edition prelude"))
    }

    /// Resolve the injected `extern crate core` as rustc does. An
    /// `--extern core=` alias names the direct dependency, which may be the
    /// provider or a crate re-exporting it; otherwise the entry is bound to the
    /// authenticated provider's imported root.
    fn provider_root(
        &mut self,
        provider: &crate::support::crates::CrateIdentity,
    ) -> Result<DefId, Error> {
        let external = self
            .facts
            .extern_scope
            .ok_or_else(|| error("no_std source client requires its imported core provider"))?;
        let root = self
            .facts
            .crates
            .as_deref()
            .ok_or_else(|| error("no_std source client requires its imported core provider"))?
            .definition(
                &ExternalDef::new(provider.clone(), Vec::new(), ExternalKind::Module)
                    .map_err(error)?,
            )
            .ok_or_else(|| error("no_std source client lost its core provider root"))?;
        let core = self.package.names.intern("core").map_err(error)?;
        if let Some(alias) = self.facts.bindings.get(external, core, Namespace::Type) {
            return self.injected_alias(*alias);
        }
        self.facts
            .bindings
            .insert(
                external,
                core,
                Namespace::Type,
                Binding {
                    target: root,
                    access: Access::Public,
                    origin: Origin::Direct,
                    ambiguous: false,
                },
            )
            .map_err(error)?;
        Ok(root)
    }

    /// Only a dependency's imported crate root may stand for `core`.
    fn injected_alias(&self, alias: Binding) -> Result<DefId, Error> {
        let root = self
            .facts
            .definitions
            .get(alias.target.0 as usize)
            .filter(|row| {
                row.kind == DefKind::Module && row.node.is_none() && row.members.is_some()
            });
        if root.is_none()
            || alias.access != Access::Public
            || alias.origin != Origin::Direct
            || alias.ambiguous
        {
            return Err(error(
                "no_std source client's `core` alias is not a dependency root",
            ));
        }
        Ok(alias.target)
    }

    fn module_scope(&self, module: DefId) -> Result<ScopeId, Error> {
        self.facts
            .definitions
            .get(module.0 as usize)
            .filter(|row| row.kind == DefKind::Module)
            .and_then(|row| row.members)
            .ok_or_else(|| error("no_std source client's core prelude is not a module"))
    }
}
