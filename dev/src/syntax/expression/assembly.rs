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
        let spelling = parser.cursor.spelling(token);
        let arg = if spelling == "options" {
            templates = false;
            parser.required()?;
            parser.expect(P::OpenParen)?;
            let mut options = 0;
            while !parser.eat(P::CloseParen)? {
                let token = parser.required()?;
                let bit = assembly::option(parser.cursor.spelling(token)).ok_or_else(|| {
                    crate::syntax::token_tree::error(token.start, "unsupported assembly option")
                })?;
                if options & bit != 0 {
                    return Err(crate::syntax::token_tree::error(
                        token.start,
                        "duplicate assembly option",
                    ));
                }
                options |= bit;
                if !parser.eat(P::Comma)? {
                    parser.expect(P::CloseParen)?;
                    break;
                }
            }
            parser.node(start, Kind::AssemblyOptions(options))?
        } else {
            let named = parser
                .cursor
                .peek(1)?
                .is_some_and(|t| t.kind == TokenKind::Punctuation(P::Equal));
            let name = if named {
                let t = parser.required()?;
                let name = parser.node(
                    t.start,
                    Kind::Name(
                        crate::syntax::token_tree::Span {
                            start: t.start,
                            end: t.end,
                        },
                        None,
                    ),
                )?;
                parser.expect(P::Equal)?;
                Some(name)
            } else {
                None
            };
            let next = parser.cursor.peek(0)?.ok_or_else(|| {
                crate::syntax::token_tree::error(start, "missing assembly operand")
            })?;
            let direction = match parser.cursor.spelling(next) {
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
            if let Some(direction) = direction {
                templates = false;
                parser.required()?;
                let register = if matches!(direction, Direction::Const | Direction::Sym) {
                    None
                } else {
                    Some(parser.assembly_register()?)
                };
                let mut input = None;
                let mut output = None;
                if matches!(direction, Direction::Out | Direction::LateOut) {
                    if !parser.eat_word(super::Word::Underscore)? {
                        output = Some(parser.expr(0, NORMAL)?);
                    }
                } else {
                    input = Some(parser.expr(0, NORMAL)?);
                    if matches!(direction, Direction::InOut | Direction::InLateOut) {
                        if parser.eat(P::FatArrow)? {
                            if !parser.eat_word(super::Word::Underscore)? {
                                output = Some(parser.expr(0, NORMAL)?);
                            }
                        } else {
                            output = input;
                        }
                    }
                }
                parser.node(
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
                if !templates || name.is_some() {
                    return Err(parser.error("assembly templates must precede operands")?);
                }
                let expression = parser.expr(0, NORMAL)?;
                parser.node(
                    start,
                    Kind::AssemblyTemplate {
                        value: expression,
                        pieces: super::List { start: 0, end: 0 },
                    },
                )?
            }
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
