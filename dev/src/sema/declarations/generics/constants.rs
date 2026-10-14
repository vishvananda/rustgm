//! Const arguments share the typed constant evaluator; symbolic terms stay unevaluated.
use super::{Checker, DefId, Error, Kind, NodeKey, TypeId, Work};
use crate::{
    resolve::{
        model::{DefKind, Target},
        package::ModuleId,
    },
    syntax::ast::TypeFlavor,
    types::arguments::{ConstId, Constant},
};
impl Checker<'_> {
    pub(super) fn const_parameter(&self, key: NodeKey) -> Option<ConstId> {
        self.facts.const_parameter(self.report, key)
    }
    pub(super) fn generic_constant(
        &mut self,
        mut key: NodeKey,
        ty: TypeId,
        owner: DefId,
    ) -> Result<ConstId, Error> {
        if let Some(id) = self.const_parameter(key) {
            if self.facts.types.const_type(id) != Some(ty) {
                return Err(self.error(key, "const argument has the wrong type"));
            }
            return Ok(id);
        }
        if let Kind::Type {
            flavor: TypeFlavor::Path,
            elements,
        } = self.kind(key)
            && let Some(node) = self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .list(elements)
                .first()
        {
            key.node = *node;
        }
        let module = self.constant_module(key, owner)?;
        let value = crate::sema::constants::generic_argument(
            self.report,
            &mut self.facts,
            &mut self.constant_scratch,
            &mut self.discriminants,
            key,
            ty,
            module,
        )?;
        self.facts
            .types
            .intern_constant(Constant { ty, bits: value })
            .map_err(|e| self.error(key, e))
    }
    fn constant_module(&mut self, key: NodeKey, owner: DefId) -> Result<ModuleId, Error> {
        // Non-block arguments are standalone literals/paths. Their references
        // already passed visibility checking; named initializers own their scopes.
        if !matches!(self.kind(key), Kind::Block { .. }) {
            let scope = self.report.resolution.definitions[owner.0 as usize].scope;
            return Ok(self.report.resolution.scopes[scope.0 as usize].module);
        }
        let modules = self.constant_modules.get_or_insert_with(|| {
            let mut modules = std::collections::HashMap::new();
            for &(module, node) in self.report.resolution.node_scopes.keys() {
                let file = self.report.package.modules[module.0 as usize].file;
                modules
                    .entry(NodeKey { file, node })
                    .and_modify(|current| {
                        if *current != Some(module) {
                            *current = None;
                        }
                    })
                    .or_insert(Some(module));
            }
            modules
        });
        modules
            .get(&key)
            .copied()
            .flatten()
            .ok_or_else(|| self.error(key, "shape constant requires an unambiguous lexical module"))
    }
    pub(super) fn constant_dependencies(&mut self, root: NodeKey) -> Result<(), Error> {
        let mut work = vec![root];
        let mut children = Vec::new();
        while let Some(key) = work.pop() {
            if !self.constant_dependencies.insert(key) {
                continue;
            }
            if self.constant_dependencies.len() > 1_000_000 {
                return Err(self.error(root, "const argument dependency budget exceeded"));
            }
            if let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference)
                && self.report.resolution.definitions[id.0 as usize].kind == DefKind::Const
            {
                self.work.push(Work::Definition(id, false));
                if let Some(item_key) = self.report.resolution.definitions[id.0 as usize].node
                    && let crate::syntax::item::ItemKind::Const {
                        value: Some(node), ..
                    } = self.item(item_key)?.kind
                {
                    work.push(NodeKey { node, ..item_key });
                }
            }
            let ast = &self.report.package.unit(key.file).configured.ast;
            children.clear();
            ast.push_children(key.node, &mut children);
            work.extend(children.iter().copied().map(|node| NodeKey { node, ..key }));
        }
        Ok(())
    }
}
