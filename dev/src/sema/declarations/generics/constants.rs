//! Const arguments share the typed constant evaluator; symbolic terms stay unevaluated.
use super::{Checker, DefId, Error, Kind, NodeKey, TypeId, Work, regions};
use crate::{
    resolve::{
        model::{DefKind, Namespace, Target},
        package::ModuleId,
    },
    syntax::ast::NodeId,
    types::arguments::ConstId,
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
        if self.schedule_trait_callable(key, context, steps, limit, scan)? {
            return Ok(());
        }
        if self.constant_trait_prefix(key)
            && self.schedule_implicit_method(key, context, steps, limit, scan)?
        {
            return Ok(());
        }
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
        self.constant_initializer_dependencies(key)?;
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
    fn constant_trait_prefix(&self, key: NodeKey) -> bool {
        self.report
            .resolution
            .associated_paths
            .get(&key)
            .is_some_and(|path| {
                path.namespace == Namespace::Value
                    && (matches!(
                        self.report.resolution.definitions[path.owner.0 as usize].kind,
                        DefKind::Generic | DefKind::Primitive
                    ) || self.facts.associated.trait_owner(path.owner).is_some())
            })
    }
    fn constant_initializer_dependencies(&mut self, key: NodeKey) -> Result<(), Error> {
        let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference) else {
            return Ok(());
        };
        if self.report.resolution.definitions[id.0 as usize].kind == DefKind::Const
            && let Some((source, context)) = self.constant_initializer(id)?
        {
            self.constant_dependencies_in(source, context)?;
        }
        Ok(())
    }
    fn constant_initializer(
        &self,
        id: DefId,
    ) -> Result<Option<(NodeKey, regions::Context)>, Error> {
        let Some(item) = self.report.resolution.definitions[id.0 as usize].node else {
            return Ok(None);
        };
        let (crate::syntax::item::ItemKind::Const {
            value: Some(node), ..
        }
        | crate::syntax::item::ItemKind::Static {
            value: Some(node), ..
        }) = self.item(item)?.kind
        else {
            return Ok(None);
        };
        Ok(Some((
            NodeKey { node, ..item },
            self.local_context(item, regions::Context::ALIAS),
        )))
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
        key: NodeKey,
        ty: TypeId,
        owner: DefId,
    ) -> Result<ConstId, Error> {
        let module = if self.const_parameter(key).is_some() {
            None
        } else {
            Some(self.constant_module(key, owner)?)
        };
        crate::sema::arguments::Constants {
            report: self.report,
            facts: &mut self.facts,
            captures: &mut self.constant_captures,
            scratch: &mut self.constant_scratch,
            discriminants: &mut self.discriminants,
        }
        .argument(key, ty, module)
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
        self.constant_dependencies_in(root, context)
    }
    fn constant_dependencies_in(
        &mut self,
        root: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
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
            if self.constant_trait_prefix(key) {
                self.work.push(Work::ConstantUse(key, context));
                continue;
            }
            if let Some(path) = self.report.resolution.qualified_paths.get(&key)
                && path.trait_path.is_some()
                && path.namespace == Namespace::Value
            {
                self.work.push(Work::ConstantUse(key, context));
                continue;
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
                            || self.facts.associated.trait_owner(id).is_some()
                        {
                            self.work.push(Work::ConstantUse(key, context));
                        }
                        self.work.push(Work::ValueType(id));
                        self.work.push(Work::Definition(id, false));
                        if let Some(dependency) = self.constant_initializer(id)? {
                            work.push(dependency);
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
