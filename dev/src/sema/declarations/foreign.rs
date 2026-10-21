//! Foreign ABI and safety come from the retained declaration's actual block.
use super::{Checker, DefId, Error, Item, ItemKind, Kind, NodeKey, Safety, callable_convention};
use crate::{
    syntax::item::Abi,
    types::{TypeId, callable::Convention},
};

impl Checker<'_> {
    pub(super) fn validate_foreign_blocks(&self) -> Result<(), Error> {
        let mut blocks: Vec<_> = self
            .report
            .resolution
            .foreign_blocks
            .iter()
            .copied()
            .collect();
        blocks.sort_unstable_by_key(|key| (key.file.0, key.node.0));
        for key in blocks {
            let ItemKind::ExternBlock { unsafe_, abi, .. } = self.item(key)?.kind else {
                return Err(self.error(key, "foreign owner is not an external block"));
            };
            if !unsafe_ {
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
            if flags.safety == Safety::Safe {
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
        let ast = &self.report.package.unit(key.file).configured.ast;
        for &node in ast.list(item.generics) {
            if !self.report.package.unit(key.file).configured.active[node.0 as usize] {
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
        if !self
            .report
            .resolution
            .foreign_owners
            .contains_key(&definition)
        {
            return Ok(());
        }
        let name = self.report.resolution.definitions[definition.0 as usize].name;
        let spelling = self.report.package.names.get(name);
        self.facts
            .foreign
            .get_or_insert_default()
            .import_function(definition, signature, safety, spelling)
            .map_err(|e| self.error(key, e))
    }
}
