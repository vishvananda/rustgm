//! Stable flat syntax view. IDs and child ranges are the in-process interface.
use super::{Ast, Kind, List, NodeId, P, Span, parse};
use crate::syntax::ast::Syntax;
use crate::{lex::render::RenderError, literal, support::quoted};
use std::{
    fmt,
    io::{self, Write},
};

mod item;

struct Optional(Option<NodeId>);
impl fmt::Display for Optional {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = self.0 {
            write!(output, "{}", id.get())
        } else {
            output.write_str("-")
        }
    }
}
struct CallableAbi(crate::syntax::item::Abi);
impl fmt::Display for CallableAbi {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            crate::syntax::item::Abi::Rust => output.write_str("-"),
            crate::syntax::item::Abi::Extern(None) => output.write_str("extern"),
            crate::syntax::item::Abi::Extern(Some(node)) => write!(output, "{}", node.get()),
        }
    }
}
struct Children<'a, L>(&'a Syntax<L>, List);
impl<L> fmt::Display for Children<'_, L> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str("[")?;
        for (index, id) in self.0.list(self.1).iter().enumerate() {
            if index != 0 {
                output.write_str(",")?;
            }
            write!(output, "{}", id.get())?;
        }
        output.write_str("]")
    }
}

fn spelling(source: &str, span: Span, output: &mut impl Write) -> io::Result<()> {
    quoted::bytes(
        output,
        &source.as_bytes()[span.start as usize..span.end as usize],
        false,
    )
}

fn label(source: &str, span: Option<Span>, output: &mut impl Write) -> io::Result<()> {
    if let Some(span) = span {
        spelling(source, span, output)
    } else {
        write!(output, "-")
    }
}

fn operator(op: P) -> &'static str {
    match op {
        P::Plus => "+",
        P::Minus => "-",
        P::Star => "*",
        P::Slash => "/",
        P::Percent => "%",
        P::Caret => "^",
        P::And => "&",
        P::Or => "|",
        P::Equal => "=",
        P::Less => "<",
        P::Greater => ">",
        P::AndAnd => "&&",
        P::OrOr => "||",
        P::ShiftLeft => "<<",
        P::ShiftRight => ">>",
        P::PlusEqual => "+=",
        P::MinusEqual => "-=",
        P::StarEqual => "*=",
        P::SlashEqual => "/=",
        P::PercentEqual => "%=",
        P::CaretEqual => "^=",
        P::AndEqual => "&=",
        P::OrEqual => "|=",
        P::ShiftLeftEqual => "<<=",
        P::ShiftRightEqual => ">>=",
        P::EqualEqual => "==",
        P::BangEqual => "!=",
        P::LessEqual => "<=",
        P::GreaterEqual => ">=",
        _ => "invalid-binary-operator",
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive syntax vocabulary dispatch, checked against the file audit dispatch limit"
)]
fn record<L>(source: &str, ast: &Syntax<L>, kind: Kind, output: &mut impl Write) -> io::Result<()> {
    match kind {
        Kind::LibraryValue { kind, value } => write!(
            output,
            "library-value {} value={}",
            kind.name(),
            Optional(value)
        ),
        Kind::Expansion { children, tail } => write!(
            output,
            "expansion {} tail={}",
            Children(ast, children),
            Optional(tail)
        ),
        Kind::DeferredLiteral(token) => {
            write!(output, "deferred-literal {} ", token.kind.name())?;
            spelling(
                source,
                Span {
                    start: token.start,
                    end: token.end,
                },
                output,
            )
        }
        Kind::Item(id) => write!(output, "item {id}"),
        Kind::PreciseCapture(_)
        | Kind::Receiver { .. }
        | Kind::GenericMethodCall { .. }
        | Kind::Variadic
        | Kind::Crate(_)
        | Kind::Attributed { .. }
        | Kind::Applied { .. }
        | Kind::Qualified { .. }
        | Kind::Constraint { .. }
        | Kind::Bound { .. }
        | Kind::Generic(_)
        | Kind::Where { .. }
        | Kind::FunctionType(_)
        | Kind::TraitType { .. }
        | Kind::FieldDefinition { .. }
        | Kind::Variant(_)
        | Kind::UseTree(_)
        | Kind::StructPattern { .. }
        | Kind::PatternField { .. }
        | Kind::PatternRange { .. } => item::node(source, ast, kind, output),
        Kind::Literal(id) => write!(output, "literal {id}"),
        Kind::CrateRoot(id) => write!(output, "crate-root {id}"),
        Kind::Name(span) => {
            write!(output, "name ")?;
            spelling(source, span, output)
        }
        Kind::Path { absolute, segments } => {
            write!(output, "path {absolute} {}", Children(ast, segments))
        }
        Kind::Group(value) => write!(output, "group {}", value.get()),
        Kind::Tuple(list) => write!(output, "tuple {}", Children(ast, list)),
        Kind::Array(list) => write!(output, "array {}", Children(ast, list)),
        Kind::Repeat { value, count } => write!(output, "repeat {} {}", value.get(), count.get()),
        Kind::Unary { op, operand } => write!(output, "unary {} {}", op.name(), operand.get()),
        Kind::Binary { op, left, right } => {
            write!(
                output,
                "binary {} {} {}",
                operator(op),
                left.get(),
                right.get()
            )
        }
        Kind::Range {
            inclusive,
            start,
            end,
        } => write!(
            output,
            "range {inclusive} {} {}",
            Optional(start),
            Optional(end)
        ),
        Kind::Cast { value, ty } => write!(output, "cast {} {}", value.get(), ty.get()),
        Kind::Call { callee, arguments } => {
            write!(output, "call {} {}", callee.get(), Children(ast, arguments))
        }
        Kind::MethodCall {
            receiver,
            name,
            arguments,
        } => {
            write!(output, "method-call {} ", receiver.get())?;
            spelling(source, name, output)?;
            write!(output, " {}", Children(ast, arguments))
        }
        Kind::Index { value, index } => write!(output, "index {} {}", value.get(), index.get()),
        Kind::Field { value, name } => {
            write!(output, "field {} ", value.get())?;
            spelling(source, name, output)
        }
        Kind::Struct { path, fields, base } => write!(
            output,
            "struct {} {} {}",
            path.get(),
            Children(ast, fields),
            Optional(base)
        ),
        Kind::FieldInit { name, value } => {
            write!(output, "field-init ")?;
            spelling(source, name, output)?;
            write!(output, " {}", value.get())
        }
        Kind::Block {
            flavor,
            statements,
            tail,
        } => write!(
            output,
            "block {flavor:?} {} {}",
            Children(ast, statements),
            Optional(tail)
        ),
        Kind::EmptyStatement => write!(output, "empty-statement"),
        Kind::Statement(value) => write!(output, "statement {}", value.get()),
        Kind::Let {
            pattern,
            ty,
            value,
            otherwise,
        } => write!(
            output,
            "let {} {} {} {}",
            pattern.get(),
            Optional(ty),
            Optional(value),
            Optional(otherwise)
        ),
        Kind::LetCondition { pattern, value } => {
            write!(output, "let-condition {} {}", pattern.get(), value.get())
        }
        Kind::If {
            condition,
            then,
            otherwise,
        } => write!(
            output,
            "if {} {} {}",
            condition.get(),
            then.get(),
            Optional(otherwise)
        ),
        Kind::Loop { body } => write!(output, "loop {}", body.get()),
        Kind::While { condition, body } => {
            write!(output, "while {} {}", condition.get(), body.get())
        }
        Kind::For {
            pattern,
            iterator,
            body,
        } => write!(
            output,
            "for {} {} {}",
            pattern.get(),
            iterator.get(),
            body.get()
        ),
        Kind::Label { name, value } => {
            write!(output, "label ")?;
            spelling(source, name, output)?;
            write!(output, " {}", value.get())
        }
        Kind::Return(value) => write!(output, "return {}", Optional(value)),
        Kind::Break { label: name, value } => {
            write!(output, "break ")?;
            label(source, name, output)?;
            write!(output, " {}", Optional(value))
        }
        Kind::Continue(name) => {
            write!(output, "continue ")?;
            label(source, name, output)
        }
        Kind::Match { value, arms } => {
            write!(output, "match {} {}", value.get(), Children(ast, arms))
        }
        Kind::Arm {
            pattern,
            guard,
            value,
        } => write!(
            output,
            "arm {} {} {}",
            pattern.get(),
            Optional(guard),
            value.get()
        ),
        Kind::Closure {
            parameters,
            result,
            body,
            moving,
            asynchronous,
        } => write!(
            output,
            "closure {moving} {asynchronous} {} {} {}",
            Children(ast, parameters),
            Optional(result),
            body.get()
        ),
        Kind::Parameter { pattern, ty } => {
            write!(output, "parameter {} {}", pattern.get(), Optional(ty))
        }
        Kind::Macro { path, capture, .. } => write!(output, "macro {} {capture}", path.get()),
        Kind::Binding {
            name,
            mutable,
            by_ref,
            subpattern,
        } => {
            write!(output, "binding {mutable} {by_ref} ")?;
            spelling(source, name, output)?;
            write!(output, " {}", Optional(subpattern))
        }
        Kind::Wildcard => write!(output, "wildcard"),
        Kind::RestPattern => write!(output, "rest-pattern"),
        Kind::Pattern {
            flavor,
            path,
            elements,
        } => write!(
            output,
            "pattern {flavor:?} {} {}",
            Optional(path),
            Children(ast, elements)
        ),
        Kind::ReferencePattern { mutable, pattern } => {
            write!(output, "reference-pattern {mutable} {}", pattern.get())
        }
        Kind::Type { flavor, elements } => {
            write!(output, "type {flavor:?} {}", Children(ast, elements))
        }
        Kind::InferType => write!(output, "infer-type"),
        Kind::NeverType => write!(output, "never-type"),
    }
}

/// # Errors
/// Parses before emitting any output; rejects malformed fragments atomically.
pub fn expression(source: &str, output: &mut impl Write) -> Result<(), RenderError> {
    let ast = parse(source).map_err(RenderError::Lex)?;
    write_ast(source, &ast, output).map_err(RenderError::Io)
}

/// Render existing typed syntax without reparsing it.
///
/// # Errors
/// Reports output failures.
pub fn write_ast(source: &str, ast: &Ast<'_>, output: &mut impl Write) -> io::Result<()> {
    write_syntax(source, ast, output, |index, span, out| {
        literal::render::value(out, &ast.literals[index as usize], span.start, span.end)
    })
}

/// Render a borrowed syntax arena with caller-owned literal storage. The view
/// never clones an arena or changes any retained semantic facts.
/// # Errors
/// Reports output failures and invalid literal storage supplied by the caller.
pub fn write_syntax<L, W: Write>(
    source: &str,
    ast: &Syntax<L>,
    output: &mut W,
    mut value: impl FnMut(u32, Span, &mut W) -> io::Result<()>,
) -> io::Result<()> {
    for (id, node) in ast.nodes.iter().enumerate() {
        write!(output, "node {id} {} {} ", node.span.start, node.span.end)?;
        record(source, ast, node.kind, output)?;
        writeln!(output)?;
        if let Kind::Literal(index) = node.kind {
            write!(output, "value {index} ")?;
            value(index, node.span, output)?;
        }
        if let Kind::Item(index) = node.kind {
            write!(output, "item {index} ")?;
            item::item(source, ast, &ast.items[index as usize], output)?;
            writeln!(output)?;
        }
    }
    for (id, capture) in ast.attributes.iter().enumerate() {
        let attribute = capture.attribute;
        write!(
            output,
            "attribute-capture {id} {} {} {} ",
            attribute.style.name(),
            attribute.span.start,
            attribute.span.end
        )?;
        spelling(source, attribute.span, output)?;
        writeln!(output)?;
    }
    for (id, capture) in ast.captures.iter().enumerate() {
        if let Some(tree) = capture.tree(0) {
            write!(
                output,
                "capture {id} {} {} {} ",
                tree.span.start,
                tree.span.end,
                capture.nodes().len()
            )?;
            spelling(source, tree.span, output)?;
            writeln!(output)?;
        }
    }
    writeln!(output, "root {}", ast.root.get())?;
    writeln!(output, "eof {}", source.len())
}
