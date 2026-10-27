//! Foreign ABI and safety come from the retained declaration's actual block.
mod linkage_attributes;
mod simd;
use super::{Checker, DefId, Error, Item, ItemKind, Kind, NodeKey, Safety, callable_convention};
use crate::{
    syntax::item::Abi,
    types::{TypeId, callable::Convention},
};
pub(super) use linkage_attributes::{Attributes, collect};

impl Checker<'_> {
    pub(super) fn foreign_type(
        &mut self,
        id: DefId,
        key: NodeKey,
        item: Item,
    ) -> Result<(), Error> {
        let declaration = self
            .report
            .resolution
            .definitions
            .get(id.0 as usize)
            .ok_or_else(|| self.error(key, "foreign type declaration disappeared"))?;
        let block = self
            .report
            .resolution
            .foreign_owners
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(key, "foreign type requires its actual external owner"))?;
        if declaration.node != Some(key)
            || declaration.kind != crate::resolve::model::DefKind::ForeignType
            || item.kind != ItemKind::ForeignType
            || item.generics.start != item.generics.end
            || item.where_clause.start != item.where_clause.end
            || !self.report.resolution.foreign_blocks.contains(&block)
            || !matches!(self.item(block)?.kind, ItemKind::ExternBlock { .. })
        {
            return Err(self.error(key, "foreign type changed its actual declaration or owner"));
        }
        let ty = self.intern(key, crate::types::Kind::Foreign(crate::types::ItemId(id.0)))?;
        self.complete(id, key, ty)
    }
    pub(super) fn validate_foreign_blocks(&self) -> Result<(), Error> {
        let mut blocks: Vec<_> = self
            .report
            .resolution
            .foreign_blocks
            .iter()
            .copied()
            .collect();
        blocks.sort_unstable_by_key(|key| (key.file.0, key.node.get()));
        for key in blocks {
            let ItemKind::ExternBlock { unsafe_, abi, .. } = self.item(key)?.kind else {
                return Err(self.error(key, "foreign owner is not an external block"));
            };
            let package = &self.report.package;
            let span = package.unit(key.file).configured.ast.node(key.node).span;
            if !unsafe_
                && crate::resolve::unit::editions::introducer(key.file, span, |file| {
                    package.units.get(file.0 as usize)
                })
                .map_err(|error| self.error(key, error.message))?
                    == crate::support::edition::Edition::Rust2024
            {
                return Err(self.error(key, "external blocks require unsafe"));
            }
            callable_convention(self.report, key, Abi::Extern(abi))?;
        }
        Ok(())
    }
    pub(super) fn foreign_signature(
        &self,
        key: NodeKey,
        owner: Option<DefId>,
        item: Item,
    ) -> Result<(bool, Convention), Error> {
        let ItemKind::Function {
            flags, abi, body, ..
        } = item.kind
        else {
            return Err(self.error(key, "foreign signature is not a function"));
        };
        let block = owner.and_then(|id| self.report.resolution.foreign_owners.get(&id));
        let Some(&block) = block else {
            if flags.safety == Safety::Safe
                && owner.is_none_or(|id| self.facts.associated.trait_owner(id).is_none())
            {
                return Err(self.error(key, "safe qualifier requires a foreign declaration"));
            }
            return Ok((
                flags.safety == Safety::Unsafe,
                callable_convention(self.report, key, abi)?,
            ));
        };
        if body.is_some() || flags.constant || flags.asynchronous || abi != Abi::Rust {
            return Err(self.error(key, "invalid foreign function definition or qualifier"));
        }
        self.foreign_member_safety(key, block, flags.safety)?;
        let ast = &self.report.package.unit(key.file).configured.ast;
        for &node in ast.list(item.generics) {
            if !self.report.package.unit(key.file).configured.active[node.get() as usize] {
                continue;
            }
            if let Kind::Generic(index) = ast.node(node).kind
                && ast.generics[index as usize].flavor
                    != crate::syntax::item::GenericFlavor::Lifetime
            {
                return Err(self.error(
                    key,
                    "foreign functions cannot have type or const parameters",
                ));
            }
        }
        let ItemKind::ExternBlock { abi, .. } = self.item(block)?.kind else {
            return Err(self.error(block, "foreign owner is not an external block"));
        };
        Ok((
            flags.safety != Safety::Safe,
            callable_convention(self.report, block, Abi::Extern(abi))?,
        ))
    }
    pub(super) fn register_foreign_function(
        &mut self,
        definition: DefId,
        key: NodeKey,
        signature: TypeId,
        safety: Safety,
    ) -> Result<(), Error> {
        let Some((naming, linkage)) = self.foreign_linkage(definition, key)? else {
            return Ok(());
        };
        if linkage == crate::support::linkage::Linkage::Export
            && (self.facts.generics.parameters.contains_key(&definition)
                || self.facts.associated.schema(definition).is_some())
        {
            return Err(self.error(
                key,
                "generic function exports require a stable external argument identity",
            ));
        }
        self.facts
            .foreign
            .get_or_insert_default()
            .insert_function(
                definition,
                crate::sema::foreign::Function {
                    signature,
                    safety,
                    naming,
                    linkage,
                },
            )
            .map_err(|e| self.error(key, e))
    }
    pub(super) fn register_foreign_global(
        &mut self,
        definition: DefId,
        key: NodeKey,
        ty: TypeId,
        mutable: bool,
        safety: Safety,
    ) -> Result<bool, Error> {
        let block = self
            .report
            .resolution
            .foreign_owners
            .get(&definition)
            .copied();
        if let Some(block) = block {
            self.foreign_member_safety(key, block, safety)?;
        }
        let imported = block.is_some();
        if !imported && safety != Safety::Default {
            return Err(self.error(
                key,
                "static safety qualifier requires a foreign declaration",
            ));
        }
        if imported
            && matches!(
                self.item(key)?.kind,
                ItemKind::Static { value: Some(_), .. }
            )
        {
            return Err(self.error(key, "foreign static cannot have an initializer"));
        }
        if let Some((naming, linkage)) = self.foreign_linkage(definition, key)? {
            self.facts
                .foreign
                .get_or_insert_default()
                .insert_global(
                    definition,
                    crate::sema::foreign::Global {
                        ty,
                        mutable,
                        safety,
                        naming,
                        linkage,
                    },
                )
                .map_err(|e| self.error(key, e))?;
        }
        Ok(imported)
    }
    fn foreign_member_safety(
        &self,
        key: NodeKey,
        block: NodeKey,
        safety: Safety,
    ) -> Result<(), Error> {
        let ItemKind::ExternBlock { unsafe_, .. } = self.item(block)?.kind else {
            return Err(self.error(block, "foreign owner is not an external block"));
        };
        if !unsafe_ && safety != Safety::Default {
            return Err(self.error(
                key,
                "foreign safety qualifier requires an unsafe external block",
            ));
        }
        Ok(())
    }
    fn foreign_linkage(
        &mut self,
        definition: DefId,
        key: NodeKey,
    ) -> Result<
        Option<(
            crate::support::linkage::Naming,
            crate::support::linkage::Linkage,
        )>,
        Error,
    > {
        use crate::support::linkage::{Linkage, Naming};
        let imported = self
            .report
            .resolution
            .foreign_owners
            .contains_key(&definition);
        let attributes = self.linkage_sources.get(&key).copied().unwrap_or_default();
        let export = attributes.no_mangle || attributes.export.is_some();
        if !imported && !export {
            return Ok(None);
        }
        if imported && export {
            return Err(self.error(
                key,
                "linkage attribute does not match declaration ownership",
            ));
        }
        let exact = if imported {
            attributes.link
        } else {
            attributes.export
        };
        let name = self.report.resolution.definitions[definition.0 as usize].name;
        let spelling = self.report.package.names.get(name);
        let registry = self.facts.foreign.get_or_insert_default();
        let exact = exact
            .map_or_else(|| registry.names_mut().intern(spelling), Ok)
            .map_err(|e| self.error(key, e))?;
        Ok(Some((
            Naming::Exact(exact),
            if imported {
                Linkage::Import
            } else {
                Linkage::Export
            },
        )))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/external_opacity.rs"]
mod tests;
