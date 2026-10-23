//! Const arguments share the typed constant evaluator; symbolic terms stay unevaluated.
use super::{Checker, DefId, Error, Kind, NodeKey, Work, regions};
use crate::{
    resolve::{
        model::{DefKind, Namespace, Target},
        package::ModuleId,
    },
    syntax::ast::NodeId,
    types::arguments::ConstId,
};
#[derive(Clone, Copy)]
enum Readiness {
    Value,
    Contract,
}
impl Readiness {
    fn work(self, ty: crate::types::TypeId) -> Work {
        match self {
            Self::Value => Work::ValueShape(ty),
            Self::Contract => Work::ValueContract(ty, false),
        }
    }
}
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
        if self.schedule_self_constructor(key, context)? {
            return Ok(());
        }
        if self.schedule_method_path(key, context, steps, limit, scan)? {
            return Ok(());
        }
        if self.schedule_alias_variant(key, context, steps, limit, scan)? {
            return Ok(());
        }
        if matches!(self.kind(key), Kind::Name(..) | Kind::Path { .. })
            && let Some(owner) = self.generic_use_owner(key)
        {
            self.schedule_item_arguments(key, owner, context, steps, limit, scan)?;
        }
        Ok(())
    }
    pub(in super::super) fn constant_arguments(&mut self, key: NodeKey) -> Result<(), Error> {
        use crate::{sema::generics::SourceArgument, types::arguments::Argument};
        let constructor = self.facts.generics.constructor_uses.get(&key).copied();
        match constructor {
            Some(crate::sema::generics::ConstructorUse::SelfType(ty)) => {
                self.work.push(Work::ValueShape(ty));
            }
            Some(crate::sema::generics::ConstructorUse::Prefix(prefix)) => {
                self.constant_annotation(prefix, Readiness::Value)?;
            }
            None => self.constant_annotation(key, Readiness::Value)?,
        }
        self.constant_trait_contracts(key)?;
        if let Some(Target::Definition(owner)) =
            self.facts.node(key).and_then(|fact| fact.reference)
            && self.report.resolution.definitions[owner.0 as usize].kind == DefKind::Const
        {
            // Associated lookup can reveal this declaration only after the
            // initial dependency scan. Its value type needs the same field
            // templates as an ordinary constant before body inference starts.
            self.work.push(Work::ValueType(owner));
        }
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
            self.constant_annotation(annotation, Readiness::Value)?;
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
    pub(in super::super) fn constant_initializer(
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
    fn constant_trait_contracts(&mut self, key: NodeKey) -> Result<(), Error> {
        use crate::sema::generics::SourceArgument;
        let Some(use_) = self.facts.generics.trait_uses.get(&key) else {
            return Ok(());
        };
        let annotations = use_
            .subject
            .into_iter()
            .chain(
                use_.arguments
                    .iter()
                    .flat_map(|use_| &use_.arguments)
                    .filter_map(|argument| {
                        if let SourceArgument::Type(key) = argument {
                            Some(*key)
                        } else {
                            None
                        }
                    }),
            )
            .collect::<Vec<_>>();
        for annotation in annotations {
            self.constant_annotation(annotation, Readiness::Contract)?;
        }
        Ok(())
    }
    fn constant_annotation(&mut self, root: NodeKey, readiness: Readiness) -> Result<(), Error> {
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
                self.work.push(readiness.work(ty));
            } else if let Some(use_) = self.facts.generics.annotation_uses.get(&key) {
                match readiness {
                    Readiness::Value => self.work.push(Work::ValueType(use_.owner)),
                    Readiness::Contract => {
                        let ty = self.facts.definitions[use_.owner.0 as usize]
                            .ok_or_else(|| self.error(key, "missing constant type identity"))?;
                        self.work.push(readiness.work(ty));
                    }
                }
                for argument in &use_.arguments {
                    match *argument {
                        AnnotationArgument::Type(key) => work.push(key),
                        AnnotationArgument::Default(Argument::Type(ty)) => {
                            self.work.push(readiness.work(ty));
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
    pub(in super::super) fn const_parameter(&self, key: NodeKey) -> Option<ConstId> {
        self.facts.const_parameter(self.report, key)
    }
    pub(in super::super) fn constant_module(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<ModuleId, Error> {
        // Non-block arguments are standalone literals/paths. Their references
        // already passed visibility checking; named initializers own their scopes.
        if !matches!(self.kind(key), Kind::Block { .. }) {
            let scope = self.report.resolution.definitions[owner.0 as usize].scope;
            return Ok(self.report.resolution.scopes[scope.0 as usize].module);
        }
        let modules = self.constant_modules.get_or_insert_with(|| {
            let mut modules = crate::support::hash::HashMap::default();
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
    pub(in super::super) fn constant_dependencies_in(
        &mut self,
        root: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let mut work = vec![(root, context)];
        let mut children = Vec::new();
        while let Some((key, inherited)) = work.pop() {
            // Nested items own separate declaration preparation and binders.
            // Their bodies must not be memoized under an enclosing const call.
            if matches!(self.kind(key), Kind::Item(_)) {
                continue;
            }
            let kind = self.kind(key);
            let context = self.local_context_of(key, kind, inherited);
            if !self.report.package.unit(key.file).configured.active[key.node.get() as usize]
                || !self.constant_dependencies.insert(key)
            {
                continue;
            }
            if self.constant_dependencies.len() > 1_000_000 {
                return Err(self.error(root, "const argument dependency budget exceeded"));
            }
            if matches!(kind, Kind::Closure { .. } | Kind::Block { .. }) {
                self.nest_local_body(key, inherited, context)?;
            }
            match kind {
                Kind::Closure { .. } => {
                    self.work.push(Work::ClosureAnnotation(key, context, None));
                }
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
            let associated = self.constant_reference_dependencies(key, context, &mut work)?;
            let ast = &self.report.package.unit(key.file).configured.ast;
            if associated || matches!(self.kind(key), Kind::Path { .. }) {
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

    /// Resolve a value path only after its ordinary declaration/constructor prerequisites.
    fn constant_reference_dependencies(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        scan: &mut Vec<(NodeKey, regions::Context)>,
    ) -> Result<bool, Error> {
        let associated = self
            .report
            .resolution
            .associated_paths
            .get(&key)
            .is_some_and(|path| path.namespace == Namespace::Value)
            || self
                .report
                .resolution
                .qualified_paths
                .get(&key)
                .is_some_and(|path| path.namespace == Namespace::Value);
        let target = self.facts.node(key).and_then(|f| f.reference);
        let owner = match target {
            Some(Target::Definition(id)) => Some(id),
            Some(Target::Associated(id))
                if self.report.resolution.definitions[id.0 as usize].kind == DefKind::TypeAlias =>
            {
                Some(id)
            }
            _ => None,
        };
        let kind = owner.map(|id| self.report.resolution.definitions[id.0 as usize].kind);
        if associated
            || matches!(
                kind,
                Some(
                    DefKind::Function
                        | DefKind::Constructor
                        | DefKind::Variant
                        | DefKind::Struct
                        | DefKind::Enum
                        | DefKind::TypeAlias
                )
            )
            || (kind == Some(DefKind::Const)
                && owner.is_some_and(|id| self.facts.associated.trait_owner(id).is_some()))
        {
            self.work.push(Work::ConstantUse(key, context));
        }
        if let Some(id) = owner {
            match kind {
                Some(DefKind::Primitive | DefKind::Function) => {
                    self.work.push(Work::Definition(id, false));
                }
                Some(DefKind::Constructor | DefKind::Variant | DefKind::Struct) => {
                    self.constant_constructor(id)?;
                }
                Some(DefKind::Enum) => self.work.push(Work::ValueType(id)),
                Some(DefKind::TypeAlias | DefKind::Const | DefKind::Static) => {
                    self.work.push(Work::ValueType(id));
                    self.work.push(Work::Definition(id, false));
                    // An associated declaration can name a default that this use
                    // overrides. The value query requests the selected body;
                    // scanning the default here can invent a dependency cycle.
                    if self.facts.associated.owner(id).is_none()
                        && let Some(dependency) = self.constant_initializer(id)?
                    {
                        scan.push(dependency);
                    }
                }
                _ => {}
            }
        }
        Ok(associated)
    }
}
