//! Unexpanded macro payloads retain typed forests and real opaque arena identities.
use super::{Action, LexError, Mode, View, Writer, error};
use crate::{
    lex::token::{Kind as TokenKind, Punctuation as P, Token},
    syntax::{
        expression::fragment::FragmentKind,
        interpolation::Nonterminal,
        item::AttributeRange,
        token_tree::{Delimiter, Forest, TreeKind},
        transfer,
    },
};
impl<'a> Writer<'a> {
    pub(super) fn capture(&mut self, view: View<'a>, index: u32) -> Result<(), LexError> {
        let forest = view
            .arena
            .captures()
            .get(index as usize)
            .ok_or_else(|| error(0, "expression writer capture row is missing"))?;
        self.enqueue(&[Action::Forest {
            view,
            forest,
            cursor: 0,
            end: forest.nodes().len(),
            previous: None,
        }])
    }
    pub(super) fn attributes(
        &mut self,
        view: View<'a>,
        attributes: AttributeRange,
    ) -> Result<(), LexError> {
        if attributes.start > attributes.end
            || attributes.end as usize > view.arena.attributes().len()
        {
            return Err(error(0, "expression writer attribute range is invalid"));
        }
        for capture in view.arena.attributes()[attributes.start as usize..attributes.end as usize]
            .iter()
            .rev()
        {
            self.enqueue(&[
                Action::Forest {
                    view,
                    forest: &capture.forest,
                    cursor: 0,
                    end: capture.forest.nodes().len(),
                    previous: None,
                },
                Action::Word(" "),
            ])?;
        }
        Ok(())
    }
    pub(super) fn forest(
        &mut self,
        view: View<'a>,
        forest: &'a Forest,
        cursor: usize,
        end: usize,
        previous: Option<TokenKind>,
    ) -> Result<(), LexError> {
        if cursor == end {
            return Ok(());
        }
        let tree = *forest
            .nodes()
            .get(cursor)
            .filter(|_| cursor < end && end <= forest.nodes().len())
            .ok_or_else(|| error(0, "expression writer token forest range changed"))?;
        let (next, first, last) = match tree.kind {
            TreeKind::Group { delimiter, after } => {
                let (open, close) = delimiters(delimiter);
                (
                    after as usize,
                    TokenKind::Punctuation(open),
                    TokenKind::Punctuation(close),
                )
            }
            TreeKind::Token(kind) => (cursor + 1, kind, kind),
            TreeKind::Doc { .. } => (cursor + 1, TokenKind::String, TokenKind::String),
        };
        if next <= cursor || next > end {
            return Err(error(
                tree.span.start,
                "expression writer token group changed",
            ));
        }
        self.enqueue(&[Action::Forest {
            view,
            forest,
            cursor: next,
            end,
            previous: Some(last),
        }])?;
        match tree.kind {
            TreeKind::Group { delimiter, .. } => {
                let (open, close) = match delimiter {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::Brace => ("{", "}"),
                };
                let inner = cursor + 1 < next;
                self.enqueue(&[
                    Action::Word(open),
                    Action::Word(if inner && delimiter == Delimiter::Brace {
                        " "
                    } else {
                        ""
                    }),
                    Action::Forest {
                        view,
                        forest,
                        cursor: cursor + 1,
                        end: next,
                        previous: None,
                    },
                    Action::Word(if inner && delimiter == Delimiter::Brace {
                        " "
                    } else {
                        ""
                    }),
                    Action::Word(close),
                ])?;
            }
            TreeKind::Token(TokenKind::Interpolated(index)) => {
                self.opaque(
                    view,
                    Token {
                        kind: TokenKind::Interpolated(index),
                        start: tree.span.start,
                        end: tree.span.end,
                    },
                )?;
            }
            TreeKind::Token(_) => self.enqueue(&[view.span(tree.span)])?,
            TreeKind::Doc { .. } => {
                // Preserve the typed comment kind; no metadata is decoded from text.
                self.enqueue(&[view.span(tree.span), Action::HardLine])?;
            }
        }
        if space(previous, first) {
            self.enqueue(&[Action::Word(" ")])?;
        }
        Ok(())
    }
    fn opaque(&mut self, view: View<'a>, token: Token) -> Result<(), LexError> {
        self.pay(1, 0)?;
        let splice = view
            .inputs
            .get(token)?
            .ok_or_else(|| error(token.start, "expression writer opaque input is missing"))?;
        match splice.fragment {
            Nonterminal::Syntax { parsed, kind } => {
                self.pay(
                    parsed
                        .ast
                        .nodes
                        .len()
                        .saturating_add(parsed.ast.edges.len()),
                    0,
                )?;
                transfer::validate_spans(&parsed.ast, |span| {
                    self.pay(1, 0)?;
                    splice.span(span)?;
                    splice
                        .original_source
                        .get(span.start as usize..span.end as usize)
                        .ok_or_else(|| {
                            error(span.start, "opaque writer source incarnation/range changed")
                        })?;
                    Ok(span)
                })?;
                let mode = match kind {
                    FragmentKind::Type => Mode::Type,
                    FragmentKind::Pattern | FragmentKind::PatternParameter => Mode::Pattern,
                    _ => Mode::Expression,
                };
                let actual = View {
                    source: splice.original_source,
                    arena: &parsed.ast,
                    inputs: view.inputs,
                    mode,
                };
                self.enqueue(&[actual.n(parsed.ast.root)])
            }
            Nonterminal::Metadata { forest, .. } => {
                let actual = View {
                    source: splice.original_source,
                    ..view
                };
                self.enqueue(&[Action::Forest {
                    view: actual,
                    forest,
                    cursor: 0,
                    end: forest.nodes().len(),
                    previous: None,
                }])
            }
            Nonterminal::Visibility { value, path } => {
                let actual = if let Some(path) = path {
                    View {
                        source: splice.original_source,
                        arena: &path.ast,
                        ..view
                    }
                } else {
                    view
                };
                self.visibility(actual, value)
            }
        }
    }
}
fn delimiters(delimiter: Delimiter) -> (P, P) {
    match delimiter {
        Delimiter::Parenthesis => (P::OpenParen, P::CloseParen),
        Delimiter::Bracket => (P::OpenBracket, P::CloseBracket),
        Delimiter::Brace => (P::OpenBrace, P::CloseBrace),
    }
}
fn space(previous: Option<TokenKind>, current: TokenKind) -> bool {
    use TokenKind::Punctuation as Q;
    let Some(previous) = previous else {
        return false;
    };
    if matches!(
        current,
        Q(P::Comma
            | P::Semicolon
            | P::Dot
            | P::ColonColon
            | P::Colon
            | P::CloseParen
            | P::CloseBracket)
    ) || matches!(
        previous,
        Q(P::Dot | P::ColonColon | P::OpenParen | P::OpenBracket | P::Bang | P::Pound)
    ) {
        return false;
    }
    if matches!(current, Q(P::OpenParen | P::OpenBracket)) {
        return false;
    }
    true
}
