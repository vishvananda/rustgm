//! Append child identities in source/grammar order without per-node allocation.
use super::{
    ast::{Kind, List, NodeId, Syntax},
    item::{ItemKind, Visibility},
};

impl<L> Syntax<L> {
    /// Child lists are short; copying element-wise avoids a library copy call.
    fn children(&self, list: List, output: &mut Vec<NodeId>) {
        output.extend(self.list(list).iter().copied());
    }

    /// Append direct child IDs. Callers may reverse the appended suffix for a
    /// LIFO worklist, or omit descendants of a disabled attributed owner.
    #[expect(
        clippy::too_many_lines,
        reason = "exhaustive typed child dispatch is kept together and audited as a dispatch function"
    )]
    pub fn push_children(&self, id: NodeId, output: &mut Vec<NodeId>) {
        match self.nodes[id.get() as usize].kind {
            Kind::GenericMethodCall {
                receiver,
                method,
                arguments,
            } => {
                output.extend([receiver, method]);
                self.children(arguments, output);
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
            Kind::Item(index) => self.item_children(index, output),
            Kind::PreciseCapture(list)
            | Kind::Crate(list)
            | Kind::Tuple(list)
            | Kind::Array(list)
            | Kind::Type { elements: list, .. }
            | Kind::TraitType { bounds: list, .. }
            | Kind::Path { segments: list, .. } => self.children(list, output),
            Kind::Attributed { value, .. }
            | Kind::Group(value)
            | Kind::Statement(value)
            | Kind::Unary { operand: value, .. }
            | Kind::Field { value, .. }
            | Kind::FieldInit { value, .. }
            | Kind::Loop { body: value }
            | Kind::Label { value, .. }
            | Kind::ReferencePattern { pattern: value, .. }
            | Kind::PatternField { pattern: value, .. }
            | Kind::Macro { path: value, .. } => output.push(value),
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
            } => output.extend([left, right]),
            Kind::Range { start, end, .. } | Kind::PatternRange { start, end, .. } => {
                output.extend(start);
                output.extend(end);
            }
            Kind::LibraryValue { value, .. }
            | Kind::Return(value)
            | Kind::Break { value, .. }
            | Kind::Binding {
                subpattern: value, ..
            } => output.extend(value),
            Kind::Call { callee, arguments } => {
                output.push(callee);
                self.children(arguments, output);
            }
            Kind::MethodCall {
                receiver,
                arguments,
                ..
            } => {
                output.push(receiver);
                self.children(arguments, output);
            }
            Kind::Struct { path, fields, base } => {
                output.push(path);
                self.children(fields, output);
                output.extend(base);
            }
            Kind::Block {
                statements, tail, ..
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                self.children(statements, output);
                output.extend(tail);
            }
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise,
            } => {
                output.push(pattern);
                output.extend(ty);
                output.extend(value);
                output.extend(otherwise);
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                output.extend([condition, then]);
                output.extend(otherwise);
            }
            Kind::For {
                pattern,
                iterator,
                body,
            } => output.extend([pattern, iterator, body]),
            Kind::Match { value, arms } => {
                output.push(value);
                self.children(arms, output);
            }
            Kind::Arm {
                pattern,
                guard,
                value,
            } => {
                output.push(pattern);
                output.extend(guard);
                output.push(value);
            }
            Kind::Closure {
                parameters,
                result,
                body,
                ..
            } => {
                self.children(parameters, output);
                output.extend(result);
                output.push(body);
            }
            Kind::Parameter { pattern, ty } => {
                output.push(pattern);
                output.extend(ty);
            }
            Kind::Pattern { path, elements, .. } => {
                output.extend(path);
                self.children(elements, output);
            }
            Kind::Applied {
                base,
                arguments,
                output: result,
                ..
            } => {
                output.push(base);
                self.children(arguments, output);
                output.extend(result);
            }
            Kind::Qualified {
                ty,
                trait_path,
                path,
            } => {
                output.push(ty);
                output.extend(trait_path);
                output.push(path);
            }
            Kind::Constraint {
                target,
                value,
                bounds,
            } => {
                output.push(target);
                output.extend(value);
                self.children(bounds, output);
            }
            Kind::Bound { binders, value, .. } => {
                self.children(binders, output);
                output.push(value);
            }
            Kind::Generic(index) => {
                let parameter = self.generics[index as usize];
                output.extend(parameter.ty);
                self.children(parameter.bounds, output);
                output.extend(parameter.default);
            }
            Kind::Where {
                binders,
                ty,
                bounds,
                ..
            } => {
                self.children(binders, output);
                output.push(ty);
                self.children(bounds, output);
            }
            Kind::FunctionType(index) => {
                let function = self.function_types[index as usize];
                self.children(function.binders, output);
                output.extend(function.abi.literal());
                self.children(function.parameters, output);
                output.extend(function.result);
            }
            Kind::FieldDefinition { visibility, ty, .. } => {
                if let Visibility::Restricted(path) = visibility {
                    output.push(path);
                }
                output.push(ty);
            }
            Kind::Variant(index) => {
                let variant = self.variants[index as usize];
                self.children(variant.fields, output);
                output.extend(variant.discriminant);
            }
            Kind::UseTree(index) => {
                let tree = self.use_trees[index as usize];
                output.extend(tree.prefix);
                self.children(tree.children, output);
                output.extend(tree.rename);
            }
            Kind::StructPattern { path, fields, .. } => {
                output.push(path);
                self.children(fields, output);
            }
        }
    }

    fn item_children(&self, index: u32, output: &mut Vec<NodeId>) {
        let item = &self.items[index as usize];
        if let Visibility::Restricted(path) = item.visibility {
            output.push(path);
        }
        self.children(item.generics, output);
        match item.kind {
            ItemKind::Function {
                abi,
                parameters,
                result,
                body,
                ..
            } => {
                output.extend(abi.literal());
                self.children(parameters, output);
                output.extend(result);
                self.children(item.where_clause, output);
                output.extend(body);
            }
            ItemKind::Struct { fields, .. } | ItemKind::Union { fields } => {
                self.children(item.where_clause, output);
                self.children(fields, output);
            }
            ItemKind::Enum { variants } => {
                self.children(item.where_clause, output);
                self.children(variants, output);
            }
            ItemKind::Trait { bounds, items, .. } => {
                self.children(bounds, output);
                self.children(item.where_clause, output);
                self.children(items, output);
            }
            ItemKind::Impl {
                trait_path,
                ty,
                items,
                ..
            } => {
                output.extend(trait_path);
                output.push(ty);
                self.children(item.where_clause, output);
                self.children(items, output);
            }
            ItemKind::Const { ty, value } | ItemKind::Static { ty, value, .. } => {
                output.push(ty);
                output.extend(value);
            }
            ItemKind::TypeAlias { bounds, ty } => {
                self.children(bounds, output);
                self.children(item.where_clause, output);
                output.extend(ty);
            }
            ItemKind::Module { items } => {
                if let Some(items) = items {
                    self.children(items, output);
                }
            }
            ItemKind::Use(value) | ItemKind::MacroInvocation(value) => output.push(value),
            ItemKind::ExternBlock { abi, items, .. } => {
                output.extend(abi);
                self.children(items, output);
            }
            ItemKind::ExternCrate { .. } | ItemKind::MacroRules { .. } => {}
        }
    }
}
