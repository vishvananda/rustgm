//! Bounded Rust-syntax spelling from actual parsed expressions, never diagnostics.
mod arena;
mod embedded_items;
mod flow;
mod layout;
mod operands;
mod structural_text;
mod token_forest;
use crate::{
    lex::token::LexError,
    syntax::{
        ast::{Kind, List, Node, NodeId, Syntax},
        interpolation::Inputs,
        token_tree::{Span, error},
    },
};
use arena::Arena;
use std::cell::Cell;

const MAX_FRAMES: usize = 4096;
const MAX_WORK: usize = 1_000_000;
const MAX_STORAGE: usize = 64 * 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Expression,
    Type,
    Pattern,
    Statement,
    LeftmostOperand,
}

#[derive(Clone, Copy)]
struct View<'a> {
    mode: Mode,
    source: &'a str,
    arena: &'a dyn Arena,
    inputs: &'a dyn Inputs,
}
impl<'a> View<'a> {
    fn expr(self) -> Self {
        Self {
            mode: Mode::Expression,
            ..self
        }
    }
    fn leftmost(self) -> Self {
        Self {
            mode: Mode::LeftmostOperand,
            ..self
        }
    }
    fn statement(self) -> Self {
        Self {
            mode: Mode::Statement,
            ..self
        }
    }
    fn ty(self) -> Self {
        Self {
            mode: Mode::Type,
            ..self
        }
    }
    fn pattern(self) -> Self {
        Self {
            mode: Mode::Pattern,
            ..self
        }
    }
    fn node(self, id: NodeId) -> Result<&'a Node, LexError> {
        self.arena
            .nodes()
            .get(id.get() as usize)
            .ok_or_else(|| error(0, "expression writer received an invalid node"))
    }
    fn list(self, list: List) -> Result<&'a [NodeId], LexError> {
        self.arena
            .edges()
            .get(list.start as usize..list.end as usize)
            .ok_or_else(|| error(0, "expression writer received an invalid list"))
    }
    fn n(self, id: NodeId) -> Action<'a> {
        Action::Node {
            view: self,
            id,
            power: 0,
        }
    }
    fn child(self, id: NodeId, power: u8) -> Action<'a> {
        Action::Node {
            view: self,
            id,
            power,
        }
    }
    fn span(self, span: Span) -> Action<'a> {
        Action::Span(self, span)
    }
    fn sequence(self, list: List, separator: &'static str) -> Action<'a> {
        Action::Sequence {
            view: self,
            list,
            separator,
            initial: true,
        }
    }
}
#[derive(Clone, Copy)]
enum Action<'a> {
    Node {
        view: View<'a>,
        id: NodeId,
        power: u8,
    },
    Span(View<'a>, Span),
    Word(&'static str),
    Separator(&'static str),
    Space {
        offset: i16,
    },
    HardLine,
    TrailingComma {
        offset: i16,
    },
    BeginBox {
        indent: u16,
        consistent: bool,
    },
    EndBox,
    Forest {
        view: View<'a>,
        forest: &'a crate::syntax::token_tree::Forest,
        cursor: usize,
        end: usize,
        previous: Option<crate::lex::token::Kind>,
    },
    Sequence {
        view: View<'a>,
        list: List,
        separator: &'static str,
        initial: bool,
    },
}
struct Writer<'a> {
    document: Vec<layout::Token<'a>>,
    work: Cell<usize>,
    storage: Cell<usize>,
    pending: Vec<Action<'a>>,
    charge: &'a dyn Fn(usize, usize) -> Result<(), LexError>,
}
impl<'a> Writer<'a> {
    fn pay(&self, work: usize, storage: usize) -> Result<(), LexError> {
        (self.charge)(work, storage)?;
        self.work.set(self.work.get().saturating_add(work));
        self.storage.set(self.storage.get().saturating_add(storage));
        if self.work.get() > MAX_WORK || self.storage.get() > MAX_STORAGE {
            return Err(error(
                0,
                "expression writer work or storage budget exceeded",
            ));
        }
        Ok(())
    }

    fn enqueue(&mut self, actions: &[Action<'a>]) -> Result<(), LexError> {
        self.pay(
            actions.len().saturating_add(1),
            actions.len().saturating_mul(4 * size_of::<Action<'a>>()),
        )?;
        if self.pending.len().saturating_add(actions.len()) > MAX_FRAMES {
            return Err(error(0, "expression writer frame budget exceeded"));
        }
        self.pending
            .try_reserve(actions.len())
            .map_err(|_| error(0, "expression writer frame allocation failed"))?;
        self.pending.extend(actions.iter().rev().copied());
        Ok(())
    }
    fn document(&mut self, token: layout::Token<'a>) -> Result<(), LexError> {
        self.pay(1, 4 * size_of::<layout::Token<'a>>())?;
        self.document
            .try_reserve(1)
            .map_err(|_| error(0, "expression document allocation failed"))?;
        self.document.push(token);
        Ok(())
    }
    fn text(&mut self, text: &'a str) -> Result<(), LexError> {
        self.pay(text.len().saturating_add(1), 0)?;
        if !text.is_empty() {
            self.document(layout::Token::Text(text))?;
        }
        Ok(())
    }
    fn separator(&mut self, separator: &'static str) -> Result<(), LexError> {
        self.pay(separator.len().saturating_add(1), 0)?;
        let mut first = true;
        for word in separator.split(' ') {
            if !first {
                self.document(layout::Token::Break {
                    blank: 1,
                    offset: 0,
                    hard: false,
                    comma: false,
                })?;
            }
            first = false;
            self.text(word)?;
        }
        Ok(())
    }
    fn sequence(
        &mut self,
        view: View<'a>,
        list: List,
        separator: &'static str,
        initial: bool,
    ) -> Result<(), LexError> {
        let values = view.list(list)?;
        let Some(&first) = values.first() else {
            return Ok(());
        };
        if initial {
            self.document(layout::Token::Begin {
                indent: 0,
                consistent: false,
            })?;
            self.enqueue(&[Action::EndBox])?;
        }
        let tail = List {
            start: list.start + 1,
            end: list.end,
        };
        if tail.start == tail.end {
            self.enqueue(&[view.n(first)])
        } else {
            self.enqueue(&[
                view.n(first),
                Action::Separator(separator),
                Action::Sequence {
                    view,
                    list: tail,
                    separator,
                    initial: false,
                },
            ])
        }
    }
    fn run(&mut self) -> Result<(), LexError> {
        while let Some(action) = self.pending.pop() {
            self.pay(1, 0)?;
            match action {
                Action::Word(text) => self.text(text)?,
                Action::Separator(text) => self.separator(text)?,
                Action::Space { offset } => self.document(layout::Token::Break {
                    blank: 1,
                    offset,
                    hard: false,
                    comma: false,
                })?,
                Action::HardLine => self.document(layout::Token::Break {
                    blank: 0,
                    offset: 0,
                    hard: true,
                    comma: false,
                })?,
                Action::TrailingComma { offset } => self.document(layout::Token::Break {
                    blank: 1,
                    hard: false,
                    comma: true,
                    offset,
                })?,
                Action::BeginBox { indent, consistent } => {
                    self.document(layout::Token::Begin { indent, consistent })?;
                }
                Action::EndBox => self.document(layout::Token::End)?,
                Action::Forest {
                    view,
                    forest,
                    cursor,
                    end,
                    previous,
                } => self.forest(view, forest, cursor, end, previous)?,
                Action::Span(view, span) => self.text(
                    view.source
                        .get(span.start as usize..span.end as usize)
                        .ok_or_else(|| {
                            error(
                                span.start,
                                "expression writer span changed its source image",
                            )
                        })?,
                )?,
                Action::Sequence {
                    view,
                    list,
                    separator,
                    initial,
                } => self.sequence(view, list, separator, initial)?,
                Action::Node { view, id, power } => {
                    let node = view.node(id)?;
                    let boundary = view.mode == Mode::LeftmostOperand
                        && matches!(
                            node.kind,
                            Kind::Block { .. }
                                | Kind::TryBlock { .. }
                                | Kind::If { .. }
                                | Kind::Match { .. }
                                | Kind::Loop { .. }
                                | Kind::While { .. }
                                | Kind::For { .. }
                        );
                    if boundary || operands::power(node.kind) < power {
                        self.enqueue(&[Action::Word("("), view.expr().n(id), Action::Word(")")])?;
                    } else {
                        self.document(layout::Token::Begin {
                            indent: if view.mode == Mode::Type { 0 } else { 4 },
                            consistent: false,
                        })?;
                        self.enqueue(&[Action::EndBox])?;
                        self.node(view, *node)?;
                    }
                }
            }
        }
        Ok(())
    }
    fn node(&mut self, view: View<'a>, node: Node) -> Result<(), LexError> {
        match node.kind {
            Kind::Name(..)
            | Kind::Literal(_)
            | Kind::DeferredLiteral(_)
            | Kind::CrateRoot(_)
            | Kind::Group(_)
            | Kind::TryBlock { .. }
            | Kind::OffsetOf { .. }
            | Kind::ConstClosure { .. }
            | Kind::Tuple(_)
            | Kind::Array(_)
            | Kind::Repeat { .. }
            | Kind::Path { .. }
            | Kind::Unary { .. }
            | Kind::Binary { .. }
            | Kind::Cast { .. }
            | Kind::Call { .. }
            | Kind::MethodCall { .. }
            | Kind::GenericMethodCall { .. }
            | Kind::Index { .. }
            | Kind::Field { .. }
            | Kind::Block { .. }
            | Kind::EmptyStatement
            | Kind::Statement(_) => self.operand(view, node),
            Kind::If { .. }
            | Kind::Loop { .. }
            | Kind::While { .. }
            | Kind::For { .. }
            | Kind::LetCondition { .. }
            | Kind::Let { .. }
            | Kind::Return(_)
            | Kind::Break { .. }
            | Kind::Continue(_)
            | Kind::Label { .. }
            | Kind::Match { .. }
            | Kind::Arm { .. }
            | Kind::Closure { .. }
            | Kind::Parameter { .. } => self.flow(view, node),
            Kind::Wildcard
            | Kind::InferType
            | Kind::NeverType
            | Kind::RestPattern
            | Kind::Variadic
            | Kind::Binding { .. }
            | Kind::ReferencePattern { .. }
            | Kind::Pattern { .. }
            | Kind::PatternRange { .. }
            | Kind::Range { .. }
            | Kind::StructPattern { .. }
            | Kind::PatternField { .. }
            | Kind::Struct { .. }
            | Kind::FieldInit { .. }
            | Kind::TypePattern { .. }
            | Kind::Type { .. }
            | Kind::Applied { .. }
            | Kind::Qualified { .. }
            | Kind::TraitType { .. }
            | Kind::PreciseCapture(_)
            | Kind::Constraint { .. }
            | Kind::Bound { .. }
            | Kind::Where { .. }
            | Kind::ImplRestriction { .. } => self.pattern_type(view, node),
            Kind::Generic(_)
            | Kind::FunctionType(_)
            | Kind::FieldDefinition { .. }
            | Kind::Receiver { .. }
            | Kind::Item(_)
            | Kind::Crate(_)
            | Kind::Expansion { .. }
            | Kind::Variant(_)
            | Kind::UseTree(_)
            | Kind::Macro { .. }
            | Kind::Attributed { .. }
            | Kind::InlineAssembly(_)
            | Kind::AssemblyTemplate { .. }
            | Kind::AssemblyOperand { .. }
            | Kind::AssemblyRegister { .. }
            | Kind::AssemblyOrigin(_)
            | Kind::AssemblyOptions(_)
            | Kind::AssemblyPlaceholder { .. }
            | Kind::LibraryValue { .. } => self.declaration_syntax(view, node),
        }
    }
}

/// The caller supplies only an actual parsed arena and its owned source image.
/// This produces text, not source-binding permission or an executable expression.
pub(crate) fn expression<L>(
    source: &str,
    ast: &Syntax<L>,
    root: NodeId,
    inputs: &dyn Inputs,
    charge: &dyn Fn(usize, usize) -> Result<(), LexError>,
) -> Result<String, LexError> {
    charge(1, 0)?;
    let view = View {
        mode: Mode::Expression,
        source,
        arena: ast,
        inputs,
    };
    let mut writer = Writer {
        document: Vec::new(),
        work: Cell::new(0),
        storage: Cell::new(0),
        pending: Vec::new(),
        charge,
    };
    view.node(root)?;
    writer.enqueue(&[view.n(root)])?;
    writer.run()?;
    layout::render(&writer.document, &|work, storage| writer.pay(work, storage))
}

#[cfg(test)]
#[path = "../../../tests/invariants/assert_printer.rs"]
mod tests;
