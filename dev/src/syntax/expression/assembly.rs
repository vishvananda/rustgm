//! Inline assembly arguments use the ordinary expression parser and its opaque inputs once.
use super::{Ast, Cursor, Kind, LexError, NORMAL, NodeId, P, Parser, Token, TokenKind};
use crate::syntax::{
    assembly::{self, Direction},
    interpolation::Inputs,
};
pub(crate) fn parse<'s>(
    source: &'s str,
    tokens: &'s [Token],
    inputs: &dyn Inputs,
) -> Result<Ast<'s>, LexError> {
    let mut parser = Parser::with_cursor(Cursor::from_tokens(source, tokens), true);
    parser.inputs = inputs;
    parser.has_opaque = !inputs.is_empty();
    let mut arguments = Vec::new();
    let mut templates = true;
    while let Some(token) = parser.cursor.peek(0)? {
        let start = token.start;
        let arg = if parser.cursor.spelling(token) == "options" {
            templates = false;
            parser.assembly_options(start)?
        } else {
            parser.assembly_argument(start, &mut templates)?
        };
        arguments.push(arg);
        if !parser.eat(P::Comma)? && parser.cursor.peek(0)?.is_some() {
            return Err(parser.error("expected comma between assembly arguments")?);
        }
    }
    if !arguments
        .first()
        .is_some_and(|n| matches!(parser.ast.node(*n).kind, Kind::AssemblyTemplate { .. }))
    {
        return Err(parser.error("assembly requires a template")?);
    }
    let start = tokens.first().map_or(0, |t| t.start);
    let mark = parser.scratch.len();
    parser.scratch.extend(arguments);
    let args = parser.list(mark)?;
    parser.ast.root = parser.node(start, Kind::InlineAssembly(args))?;
    Ok(parser.ast)
}
impl Parser<'_, '_> {
    fn assembly_options(&mut self, start: u32) -> Result<NodeId, LexError> {
        self.required()?;
        self.expect(P::OpenParen)?;
        let mut options = 0;
        while !self.eat(P::CloseParen)? {
            let token = self.required()?;
            let bit = assembly::option(self.cursor.spelling(token)).ok_or_else(|| {
                crate::syntax::token_tree::error(token.start, "unsupported assembly option")
            })?;
            if options & bit != 0 {
                return Err(crate::syntax::token_tree::error(
                    token.start,
                    "duplicate assembly option",
                ));
            }
            options |= bit;
            if !self.eat(P::Comma)? {
                self.expect(P::CloseParen)?;
                break;
            }
        }
        self.node(start, Kind::AssemblyOptions(options))
    }
    fn assembly_argument(&mut self, start: u32, templates: &mut bool) -> Result<NodeId, LexError> {
        let named = self
            .cursor
            .peek(1)?
            .is_some_and(|t| t.kind == TokenKind::Punctuation(P::Equal));
        let name = if named {
            let t = self.required()?;
            let name = self.node(
                t.start,
                Kind::Name(
                    crate::syntax::token_tree::Span {
                        start: t.start,
                        end: t.end,
                    },
                    None,
                ),
            )?;
            self.expect(P::Equal)?;
            Some(name)
        } else {
            None
        };
        let next = self
            .cursor
            .peek(0)?
            .ok_or_else(|| crate::syntax::token_tree::error(start, "missing assembly operand"))?;
        let direction = match self.cursor.spelling(next) {
            "in" => Some(Direction::In),
            "out" => Some(Direction::Out),
            "lateout" => Some(Direction::LateOut),
            "inout" => Some(Direction::InOut),
            "inlateout" => Some(Direction::InLateOut),
            "const" => Some(Direction::Const),
            "sym" => Some(Direction::Sym),
            "label" | "clobber_abi" => {
                return Err(crate::syntax::token_tree::error(
                    next.start,
                    "assembly labels and clobber ABI require their complete control contract",
                ));
            }
            _ => None,
        };
        let argument = if let Some(direction) = direction {
            *templates = false;
            self.required()?;
            let register = if matches!(direction, Direction::Const | Direction::Sym) {
                None
            } else {
                Some(self.assembly_register()?)
            };
            let mut input = None;
            let mut output = None;
            if matches!(direction, Direction::Out | Direction::LateOut) {
                if !self.eat_word(super::Word::Underscore)? {
                    output = Some(self.expr(0, NORMAL)?);
                }
            } else {
                input = Some(self.expr(0, NORMAL)?);
                if matches!(direction, Direction::InOut | Direction::InLateOut) {
                    if self.eat(P::FatArrow)? {
                        if !self.eat_word(super::Word::Underscore)? {
                            output = Some(self.expr(0, NORMAL)?);
                        }
                    } else {
                        output = input;
                    }
                }
            }
            self.node(
                start,
                Kind::AssemblyOperand {
                    direction,
                    register,
                    input,
                    output,
                    name,
                },
            )?
        } else {
            if !*templates || name.is_some() {
                return Err(self.error("assembly templates must precede operands")?);
            }
            let expression = self.expr(0, NORMAL)?;
            self.node(
                start,
                Kind::AssemblyTemplate {
                    value: expression,
                    pieces: super::List { start: 0, end: 0 },
                },
            )?
        };
        Ok(argument)
    }

    fn assembly_register(&mut self) -> Result<NodeId, LexError> {
        self.expect(P::OpenParen)?;
        let token = self.required()?;
        let explicit = matches!(token.kind, TokenKind::String | TokenKind::RawString);
        let value = if explicit {
            let value = crate::literal::interpret(self.cursor.source, token)?.ok_or_else(|| {
                crate::syntax::token_tree::error(token.start, "invalid explicit assembly register")
            })?;
            let id = u32::try_from(self.ast.literals.len()).map_err(|_| {
                crate::syntax::token_tree::error(token.start, "too many assembly register literals")
            })?;
            self.ast.literals.push(value);
            self.node(token.start, Kind::Literal(id))?
        } else {
            self.node(
                token.start,
                Kind::Name(
                    crate::syntax::token_tree::Span {
                        start: token.start,
                        end: token.end,
                    },
                    None,
                ),
            )?
        };
        self.expect(P::CloseParen)?;
        self.node(token.start, Kind::AssemblyRegister { explicit, value })
    }
}
