//! Parent roles follow installed typed edges, not a fragment's parsing context.
use crate::syntax::{
    ast::{Kind, NodeId, Syntax, TypeFlavor},
    item::{GenericFlavor, ItemKind},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Position {
    Other,
    CrateRoot,
    FunctionResult,
    FunctionBody,
    Expression,
    Statement,
    MethodReceiver,
    IfBody,
}

/// Each edge is classified in constant time; no sibling or ancestor scan.
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive typed parent-edge dispatch retains source feature positions"
)]
pub(super) fn child<L>(
    ast: &Syntax<L>,
    parent: NodeId,
    position: Position,
    child: NodeId,
) -> Position {
    match ast.node(parent).kind {
        Kind::Attributed { .. } | Kind::Loop { .. } | Kind::Label { .. } => position,
        Kind::Statement(_) | Kind::Block { .. } => Position::Statement,
        Kind::Expansion { tail, .. } => {
            if tail == Some(child) {
                position
            } else {
                Position::Other
            }
        }
        Kind::Item(index) => item(ast.items[index as usize].kind, child),
        Kind::FunctionType(index) => {
            if ast.function_types[index as usize].result == Some(child) {
                Position::FunctionResult
            } else {
                Position::Other
            }
        }
        Kind::Closure { result, body, .. } => {
            if result == Some(child) {
                Position::FunctionResult
            } else if body == child {
                Position::Expression
            } else {
                Position::Other
            }
        }
        Kind::MethodCall { receiver, .. } => {
            if receiver == child {
                Position::MethodReceiver
            } else {
                Position::Expression
            }
        }
        Kind::GenericMethodCall {
            receiver, method, ..
        } => {
            if receiver == child {
                Position::MethodReceiver
            } else if method == child {
                Position::Other
            } else {
                Position::Expression
            }
        }
        Kind::Type {
            flavor: TypeFlavor::Array,
            elements,
        } => {
            if ast.list(elements).get(1) == Some(&child) {
                Position::Expression
            } else {
                Position::Other
            }
        }
        Kind::Let {
            value, otherwise, ..
        } => {
            if value == Some(child) || otherwise == Some(child) {
                Position::Expression
            } else {
                Position::Other
            }
        }
        Kind::LetCondition { value, .. } | Kind::Cast { value, .. } | Kind::Match { value, .. } => {
            if value == child {
                Position::Expression
            } else {
                Position::Other
            }
        }
        Kind::Arm { value, guard, .. } => {
            if value == child || guard == Some(child) {
                Position::Expression
            } else {
                Position::Other
            }
        }
        Kind::Struct { base, .. } => {
            if base == Some(child) {
                Position::Expression
            } else {
                Position::Other
            }
        }
        Kind::For { iterator, body, .. } => {
            if iterator == child {
                Position::Expression
            } else if body == child {
                position
            } else {
                Position::Other
            }
        }
        Kind::While { condition, .. } => {
            if condition == child {
                Position::Expression
            } else {
                position
            }
        }
        Kind::If { condition, .. } => {
            if condition == child {
                Position::Expression
            } else {
                Position::IfBody
            }
        }
        Kind::Generic(index) => {
            let parameter = ast.generics[index as usize];
            if parameter.flavor == GenericFlavor::Const && parameter.default == Some(child) {
                Position::Expression
            } else {
                Position::Other
            }
        }
        Kind::Variant(index) => {
            if ast.variants[index as usize].discriminant == Some(child) {
                Position::Expression
            } else {
                Position::Other
            }
        }
        Kind::LibraryValue { .. }
        | Kind::Group(_)
        | Kind::Tuple(_)
        | Kind::Array(_)
        | Kind::Repeat { .. }
        | Kind::Unary { .. }
        | Kind::Binary { .. }
        | Kind::Range { .. }
        | Kind::Call { .. }
        | Kind::Index { .. }
        | Kind::Field { .. }
        | Kind::FieldInit { .. }
        | Kind::Return(_)
        | Kind::Break { .. } => Position::Expression,
        Kind::Receiver { .. }
        | Kind::DeferredLiteral(_)
        | Kind::Variadic
        | Kind::Crate(_)
        | Kind::Applied { .. }
        | Kind::Qualified { .. }
        | Kind::Constraint { .. }
        | Kind::Bound { .. }
        | Kind::Where { .. }
        | Kind::TraitType { .. }
        | Kind::PreciseCapture(_)
        | Kind::FieldDefinition { .. }
        | Kind::UseTree(_)
        | Kind::StructPattern { .. }
        | Kind::PatternField { .. }
        | Kind::PatternRange { .. }
        | Kind::Literal(_)
        | Kind::Name(..)
        | Kind::CrateRoot(_)
        | Kind::Path { .. }
        | Kind::EmptyStatement
        | Kind::Continue(_)
        | Kind::Parameter { .. }
        | Kind::Macro { .. }
        | Kind::Binding { .. }
        | Kind::Wildcard
        | Kind::RestPattern
        | Kind::Pattern { .. }
        | Kind::ReferencePattern { .. }
        | Kind::Type { .. }
        | Kind::InferType
        | Kind::NeverType => Position::Other,
    }
}

fn item(kind: ItemKind, child: NodeId) -> Position {
    match kind {
        ItemKind::Function { result, body, .. } => {
            if result == Some(child) {
                Position::FunctionResult
            } else if body == Some(child) {
                Position::FunctionBody
            } else {
                Position::Other
            }
        }
        ItemKind::Const { value, .. } | ItemKind::Static { value, .. } => {
            if value == Some(child) {
                Position::Expression
            } else {
                Position::Other
            }
        }
        ItemKind::Struct { .. }
        | ItemKind::Enum { .. }
        | ItemKind::Union { .. }
        | ItemKind::Trait { .. }
        | ItemKind::Impl { .. }
        | ItemKind::TypeAlias { .. }
        | ItemKind::Module { .. }
        | ItemKind::Use(_)
        | ItemKind::ExternCrate { .. }
        | ItemKind::ExternBlock { .. }
        | ItemKind::MacroRules { .. }
        | ItemKind::DeclarativeMacro { .. }
        | ItemKind::MacroInvocation(_) => Position::Other,
    }
}
