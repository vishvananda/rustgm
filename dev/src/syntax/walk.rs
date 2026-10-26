//! Append child identities in source/grammar order without per-node allocation.
use super::{
    ast::{Kind, List, NodeId, Syntax},
    item::{ItemKind, Visibility},
};

impl<L> Syntax<L> {
    fn children<E>(
        &self,
        list: List,
        visit: &mut impl FnMut(NodeId) -> Result<(), E>,
    ) -> Result<(), E> {
        visit_all(self.list(list).iter().copied(), visit)
    }

    /// Append direct child IDs. Callers may reverse the appended suffix for a
    /// LIFO worklist, or omit descendants of a disabled attributed owner.
    pub fn push_children(&self, id: NodeId, output: &mut Vec<NodeId>) {
        let result: Result<(), std::convert::Infallible> =
            self.try_for_each_child(id, &mut |child| {
                output.push(child);
                Ok(())
            });
        match result {
            Ok(()) => {}
            Err(never) => match never {},
        }
    }
    /// Visit actual direct edges in grammar order, stopping at the first failure.
    #[expect(
        clippy::too_many_lines,
        reason = "exhaustive typed child dispatch is kept together and audited as a dispatch function"
    )]
    pub(crate) fn try_for_each_child<E>(
        &self,
        id: NodeId,
        visit: &mut impl FnMut(NodeId) -> Result<(), E>,
    ) -> Result<(), E> {
        match self.nodes[id.get() as usize].kind {
            Kind::GenericMethodCall {
                receiver,
                method,
                arguments,
            } => {
                visit_all([receiver, method], visit)?;
                self.children(arguments, visit)?;
            }
            Kind::CrateRoot(_)
            | Kind::Receiver { .. }
            | Kind::DeferredLiteral(_)
            | Kind::Literal(_)
            | Kind::Name(..)
            | Kind::EmptyStatement
            | Kind::Continue(_)
            | Kind::Wildcard
            | Kind::RestPattern
            | Kind::InferType
            | Kind::NeverType
            | Kind::Variadic => {}
            Kind::Item(index) => self.visit_item_children(index, visit)?,
            Kind::PreciseCapture(list)
            | Kind::Crate(list)
            | Kind::Tuple(list)
            | Kind::Array(list)
            | Kind::Type { elements: list, .. }
            | Kind::TraitType { bounds: list, .. }
            | Kind::Path { segments: list, .. } => self.children(list, visit)?,
            Kind::Attributed { value, .. }
            | Kind::TypePattern { base: value, .. }
            | Kind::Group(value)
            | Kind::Statement(value)
            | Kind::Unary { operand: value, .. }
            | Kind::Field { value, .. }
            | Kind::FieldInit { value, .. }
            | Kind::Loop { body: value }
            | Kind::Label { value, .. }
            | Kind::ReferencePattern { pattern: value, .. }
            | Kind::PatternField { pattern: value, .. }
            | Kind::ImplRestriction { path: value }
            | Kind::Macro { path: value, .. } => visit(value)?,
            Kind::Binary { left, right, .. }
            | Kind::Repeat {
                value: left,
                count: right,
            }
            | Kind::Cast {
                value: left,
                ty: right,
            }
            | Kind::Index {
                value: left,
                index: right,
            }
            | Kind::LetCondition {
                pattern: left,
                value: right,
            }
            | Kind::While {
                condition: left,
                body: right,
            } => visit_all([left, right], visit)?,
            Kind::Range { start, end, .. } | Kind::PatternRange { start, end, .. } => {
                visit_all(start, visit)?;
                visit_all(end, visit)?;
            }
            Kind::LibraryValue { value, .. }
            | Kind::Return(value)
            | Kind::Break { value, .. }
            | Kind::Binding {
                subpattern: value, ..
            } => visit_all(value, visit)?,
            Kind::Call { callee, arguments } => {
                visit(callee)?;
                self.children(arguments, visit)?;
            }
            Kind::MethodCall {
                receiver,
                arguments,
                ..
            } => {
                visit(receiver)?;
                self.children(arguments, visit)?;
            }
            Kind::Struct { path, fields, base } => {
                visit(path)?;
                self.children(fields, visit)?;
                visit_all(base, visit)?;
            }
            Kind::Block {
                statements, tail, ..
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                self.children(statements, visit)?;
                visit_all(tail, visit)?;
            }
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise,
            } => {
                visit(pattern)?;
                visit_all(ty, visit)?;
                visit_all(value, visit)?;
                visit_all(otherwise, visit)?;
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                visit_all([condition, then], visit)?;
                visit_all(otherwise, visit)?;
            }
            Kind::For {
                pattern,
                iterator,
                body,
            } => visit_all([pattern, iterator, body], visit)?,
            Kind::Match { value, arms } => {
                visit(value)?;
                self.children(arms, visit)?;
            }
            Kind::Arm {
                pattern,
                guard,
                value,
            } => {
                visit(pattern)?;
                visit_all(guard, visit)?;
                visit(value)?;
            }
            Kind::Closure {
                parameters,
                result,
                body,
                ..
            } => {
                self.children(parameters, visit)?;
                visit_all(result, visit)?;
                visit(body)?;
            }
            Kind::Parameter { pattern, ty } => {
                visit(pattern)?;
                visit_all(ty, visit)?;
            }
            Kind::Pattern { path, elements, .. } => {
                visit_all(path, visit)?;
                self.children(elements, visit)?;
            }
            Kind::Applied {
                base,
                arguments,
                output: result,
                ..
            } => {
                visit(base)?;
                self.children(arguments, visit)?;
                visit_all(result, visit)?;
            }
            Kind::Qualified {
                ty,
                trait_path,
                path,
            } => {
                visit(ty)?;
                visit_all(trait_path, visit)?;
                visit(path)?;
            }
            Kind::Constraint {
                target,
                value,
                bounds,
            } => {
                visit(target)?;
                visit_all(value, visit)?;
                self.children(bounds, visit)?;
            }
            Kind::Bound { binders, value, .. } => {
                self.children(binders, visit)?;
                visit(value)?;
            }
            Kind::Generic(index) => {
                let parameter = self.generics[index as usize];
                visit_all(parameter.ty, visit)?;
                self.children(parameter.bounds, visit)?;
                visit_all(parameter.default, visit)?;
            }
            Kind::Where {
                binders,
                ty,
                bounds,
                ..
            } => {
                self.children(binders, visit)?;
                visit(ty)?;
                self.children(bounds, visit)?;
            }
            Kind::FunctionType(index) => {
                let function = self.function_types[index as usize];
                self.children(function.binders, visit)?;
                visit_all(function.abi.literal(), visit)?;
                self.children(function.parameters, visit)?;
                visit_all(function.result, visit)?;
            }
            Kind::FieldDefinition { visibility, ty, .. } => {
                if let Visibility::Restricted(path) = visibility {
                    visit(path)?;
                }
                visit(ty)?;
            }
            Kind::Variant(index) => {
                let variant = self.variants[index as usize];
                self.children(variant.fields, visit)?;
                visit_all(variant.discriminant, visit)?;
            }
            Kind::UseTree(index) => {
                let tree = self.use_trees[index as usize];
                visit_all(tree.prefix, visit)?;
                self.children(tree.children, visit)?;
                visit_all(tree.rename, visit)?;
            }
            Kind::StructPattern { path, fields, .. } => {
                visit(path)?;
                self.children(fields, visit)?;
            }
        }
        Ok(())
    }

    fn visit_item_children<E>(
        &self,
        index: u32,
        visit: &mut impl FnMut(NodeId) -> Result<(), E>,
    ) -> Result<(), E> {
        let item = &self.items[index as usize];
        if let Visibility::Restricted(path) = item.visibility {
            visit(path)?;
        }
        self.children(item.generics, visit)?;
        match item.kind {
            ItemKind::Function {
                abi,
                parameters,
                result,
                body,
                ..
            } => {
                visit_all(abi.literal(), visit)?;
                self.children(parameters, visit)?;
                visit_all(result, visit)?;
                self.children(item.where_clause, visit)?;
                visit_all(body, visit)?;
            }
            ItemKind::Struct { fields, .. } | ItemKind::Union { fields } => {
                self.children(item.where_clause, visit)?;
                self.children(fields, visit)?;
            }
            ItemKind::Enum { variants } => {
                self.children(item.where_clause, visit)?;
                self.children(variants, visit)?;
            }
            ItemKind::Trait {
                restriction,
                bounds,
                items,
                ..
            } => {
                visit_all(restriction, visit)?;
                self.children(bounds, visit)?;
                self.children(item.where_clause, visit)?;
                self.children(items, visit)?;
            }
            ItemKind::Impl {
                trait_path,
                ty,
                items,
                ..
            } => {
                visit_all(trait_path, visit)?;
                visit(ty)?;
                self.children(item.where_clause, visit)?;
                self.children(items, visit)?;
            }
            ItemKind::Const { ty, value } | ItemKind::Static { ty, value, .. } => {
                visit(ty)?;
                visit_all(value, visit)?;
            }
            ItemKind::TraitAlias { bounds, .. } => {
                self.children(bounds, visit)?;
                self.children(item.where_clause, visit)?;
            }
            ItemKind::TypeAlias { bounds, ty } => {
                self.children(bounds, visit)?;
                self.children(item.where_clause, visit)?;
                visit_all(ty, visit)?;
            }
            ItemKind::Module { items } => {
                if let Some(items) = items {
                    self.children(items, visit)?;
                }
            }
            ItemKind::Use(value) | ItemKind::MacroInvocation(value) => visit(value)?,
            ItemKind::ExternBlock { abi, items, .. } => {
                visit_all(abi, visit)?;
                self.children(items, visit)?;
            }
            ItemKind::ForeignType
            | ItemKind::ExternCrate { .. }
            | ItemKind::MacroRules { .. }
            | ItemKind::DeclarativeMacro { .. } => {}
        }
        Ok(())
    }
}
fn visit_all<E>(
    children: impl IntoIterator<Item = NodeId>,
    visit: &mut impl FnMut(NodeId) -> Result<(), E>,
) -> Result<(), E> {
    for child in children {
        visit(child)?;
    }
    Ok(())
}
