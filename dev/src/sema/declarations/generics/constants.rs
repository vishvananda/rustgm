//! Const arguments share the typed constant evaluator; symbolic terms stay unevaluated.
use super::{Checker, DefId, Error, Kind, NodeKey, TypeId, Work, regions};
use crate::{
    resolve::{
        model::{DefKind, Target},
        package::ModuleId,
    },
    syntax::ast::{NodeId, TypeFlavor},
    types::arguments::{ConstId, Constant},
};
impl Checker<'_> {
    pub(in super::super) fn constant_use(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        self.work.push(Work::ConstantArguments(key));
        if self.schedule_alias_variant(key, context, steps, limit, scan)? {
            return Ok(());
        }
        if matches!(self.kind(key), Kind::Path { .. })
            && let Some(owner) = self.generic_use_owner(key)
        {
            self.schedule_item_arguments(key, owner, context, steps, limit, scan)?;
        }
        Ok(())
    }
    pub(in super::super) fn constant_arguments(&mut self, key: NodeKey) -> Result<(), Error> {
        use crate::{sema::generics::SourceArgument, types::arguments::Argument};
        let prefix = self
            .facts
            .generics
            .constructor_uses
            .get(&key)
            .copied()
            .unwrap_or(key);
        self.constant_annotation(prefix)?;
        let Some(owner) = self.generic_use_owner(key) else {
            return Ok(());
        };
        let Some(use_) = self.facts.generics.item_uses.get(&key) else {
            return Ok(());
        };
        let supplied = use_.arguments.len();
        let annotations = use_
            .arguments
            .iter()
            .filter_map(|arg| {
                if let SourceArgument::Type(key) = arg {
                    Some(*key)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for annotation in annotations {
            self.constant_annotation(annotation)?;
        }
        if supplied != 0 {
            let defaults = self.facts.generics.parameters[&owner]
                .iter()
                .skip_while(|p| matches!(p.argument, Argument::Lifetime(_)))
                .skip(supplied)
                .filter_map(|p| {
                    if let Some(Argument::Type(ty)) = p.default {
                        Some(ty)
                    } else {
                        None
                    }
                });
            self.work.extend(defaults.map(Work::ValueShape));
        }
        Ok(())
    }
    fn constant_annotation(&mut self, root: NodeKey) -> Result<(), Error> {
        use crate::{sema::generics::AnnotationArgument, types::arguments::Argument};
        let mut work = vec![root];
        let mut children = Vec::new();
        let mut steps = 0_usize;
        while let Some(key) = work.pop() {
            steps += 1;
            if steps.saturating_add(work.len()) > 1_000_000 {
                return Err(self.error(root, "constant annotation dependency work budget exceeded"));
            }
            if let Some(ty) = self.facts.node(key).and_then(|f| f.ty) {
                self.work.push(Work::ValueShape(ty));
            } else if let Some(use_) = self.facts.generics.annotation_uses.get(&key) {
                self.work.push(Work::ValueType(use_.owner));
                for argument in &use_.arguments {
                    match *argument {
                        AnnotationArgument::Type(key) => work.push(key),
                        AnnotationArgument::Default(Argument::Type(ty)) => {
                            self.work.push(Work::ValueShape(ty));
                        }
                        _ => {}
                    }
                }
            } else if self.facts.partial_annotations.contains_key(&key) {
                let ast = &self.report.package.unit(key.file).configured.ast;
                children.clear();
                ast.push_children(key.node, &mut children);
                work.extend(children.iter().map(|&node| NodeKey { node, ..key }));
            }
        }
        Ok(())
    }
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
        self.constant_captures
            .check(self.report, &self.facts, key, module)?;
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
    pub(in super::super) fn constant_dependencies(
        &mut self,
        root: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let context = regions::Context {
            position: regions::Position::Constant(root.node),
            ..context
        };
        let mut work = vec![(root, context)];
        let mut children = Vec::new();
        while let Some((key, context)) = work.pop() {
            if !self.report.package.unit(key.file).configured.active[key.node.0 as usize]
                || !self.constant_dependencies.insert(key)
            {
                continue;
            }
            if self.constant_dependencies.len() > 1_000_000 {
                return Err(self.error(root, "const argument dependency budget exceeded"));
            }
            match self.kind(key) {
                Kind::Cast { ty, .. } | Kind::Let { ty: Some(ty), .. } => {
                    self.node_work(key.file, ty, context);
                }
                Kind::InferType => {
                    self.facts
                        .partial_annotations
                        .insert(key, crate::sema::model::PartialAnnotation::Hole);
                }
                _ => {}
            }
            if let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference) {
                match self.report.resolution.definitions[id.0 as usize].kind {
                    DefKind::Primitive => self.work.push(Work::Definition(id, false)),
                    DefKind::Function => {
                        self.work.push(Work::ConstantUse(key, context));
                        self.work.push(Work::Definition(id, false));
                    }
                    DefKind::Constructor | DefKind::Variant | DefKind::Struct => {
                        self.work.push(Work::ConstantUse(key, context));
                        self.constant_constructor(id)?;
                    }
                    DefKind::Enum => {
                        self.work.push(Work::ConstantUse(key, context));
                        self.work.push(Work::ValueType(id));
                    }
                    DefKind::TypeAlias | DefKind::Const | DefKind::Static => {
                        if self.report.resolution.definitions[id.0 as usize].kind
                            == DefKind::TypeAlias
                        {
                            self.work.push(Work::ConstantUse(key, context));
                        }
                        self.work.push(Work::ValueType(id));
                        self.work.push(Work::Definition(id, false));
                        if let Some(item_key) =
                            self.report.resolution.definitions[id.0 as usize].node
                            && let crate::syntax::item::ItemKind::Const {
                                value: Some(node), ..
                            }
                            | crate::syntax::item::ItemKind::Static {
                                value: Some(node), ..
                            } = self.item(item_key)?.kind
                        {
                            work.push((
                                NodeKey { node, ..item_key },
                                self.local_context(item_key, regions::Context::ALIAS),
                            ));
                        }
                    }
                    _ => {}
                }
            } else if let Some(Target::Associated(owner)) =
                self.facts.node(key).and_then(|f| f.reference)
                && self.report.resolution.definitions[owner.0 as usize].kind == DefKind::TypeAlias
            {
                self.work.push(Work::ConstantUse(key, context));
                self.work.push(Work::ValueType(owner));
                self.work.push(Work::Definition(owner, false));
            }
            let ast = &self.report.package.unit(key.file).configured.ast;
            if matches!(self.kind(key), Kind::Path { .. }) {
                // The selected item's argument schema schedules type and const
                // arguments separately. Nested constants own separate regions.
                continue;
            }
            children.clear();
            ast.push_children(key.node, &mut children);
            work.extend(
                children
                    .iter()
                    .copied()
                    .map(|node| (NodeKey { node, ..key }, context)),
            );
        }
        Ok(())
    }
}
