//! Pattern and type atoms needed by PA4 bodies. PA5 owns full item/type grammar.
use super::{Kind, LexError, NORMAL, NodeId, P, Parser, Span, TokenKind, error};
use crate::{
    literal,
    syntax::ast::{PatternFlavor, TypeFlavor},
};

impl Parser<'_> {
    pub(super) fn pattern(&mut self, alternatives: bool) -> Result<NodeId, LexError> {
        if self.depth >= 256 {
            return Err(self.error("syntax nesting exceeds 256")?);
        }
        self.depth += 1;
        let result = self.pattern_inner(alternatives);
        self.depth -= 1;
        result
    }

    fn pattern_inner(&mut self, alternatives: bool) -> Result<NodeId, LexError> {
        let leading = alternatives && self.eat(P::Or)?;
        let first = self.pattern_atom()?;
        if !alternatives || (!leading && !self.cursor.at(P::Or)?) {
            return Ok(first);
        }
        let mark = self.scratch.len();
        self.scratch.push(first);
        while self.eat(P::Or)? {
            let item = self.pattern_atom()?;
            self.scratch.push(item);
        }
        let elements = self.list(mark)?;
        self.node(
            self.ast.node(first).span.start,
            Kind::Pattern {
                flavor: PatternFlavor::Or,
                path: None,
                elements,
            },
        )
    }

    fn pattern_atom(&mut self) -> Result<NodeId, LexError> {
        let token = self
            .cursor
            .peek(0)?
            .ok_or_else(|| error(self.cursor.end(), "expected pattern"))?;
        let start = token.start;
        if self.word("_")? {
            return self.node(start, Kind::Wildcard);
        }
        if self.eat(P::DotDot)? {
            return self.node(start, Kind::RestPattern);
        }
        if self.cursor.at(P::And)? || self.cursor.at(P::AndAnd)? {
            if self.cursor.at(P::AndAnd)? {
                self.cursor.split(P::And, P::And)?;
            } else {
                self.required()?;
            }
            let mutable = self.word("mut")?;
            let pattern = self.pattern(false)?;
            return self.node(start, Kind::ReferencePattern { mutable, pattern });
        }
        if self.cursor.at(P::OpenParen)? || self.cursor.at(P::OpenBracket)? {
            let tuple = self.eat(P::OpenParen)?;
            if !tuple {
                self.expect(P::OpenBracket)?;
            }
            let close = if tuple {
                P::CloseParen
            } else {
                P::CloseBracket
            };
            let (elements, comma) = self.pattern_list(close)?;
            if tuple && !comma && self.ast.list(elements).len() == 1 {
                return self.node(start, Kind::Group(self.ast.list(elements)[0]));
            }
            return self.node(
                start,
                Kind::Pattern {
                    flavor: if tuple {
                        PatternFlavor::Tuple
                    } else {
                        PatternFlavor::Slice
                    },
                    path: None,
                    elements,
                },
            );
        }
        if self.cursor.at(P::Minus)? || literal::interpret(self.cursor.source, token)?.is_some() {
            return self.literal_pattern();
        }
        let by_ref = self.word("ref")?;
        let mutable = self.word("mut")?;
        let path = self.path()?;
        let Kind::Path { absolute, segments } = self.ast.node(path).kind else {
            return Err(error(start, "expected pattern path"));
        };
        if self.eat(P::OpenParen)? {
            if by_ref || mutable {
                return Err(error(start, "binding modifier requires an identifier"));
            }
            let (elements, _) = self.pattern_list(P::CloseParen)?;
            return self.node(
                start,
                Kind::Pattern {
                    flavor: PatternFlavor::Variant,
                    path: Some(path),
                    elements,
                },
            );
        }
        if absolute || self.ast.list(segments).len() != 1 {
            if by_ref || mutable {
                return Err(error(start, "binding modifier requires an identifier"));
            }
            return Ok(path);
        }
        let name = self.ast.node(self.ast.list(segments)[0]).span;
        let subpattern = if self.eat(P::At)? {
            Some(self.pattern(false)?)
        } else {
            None
        };
        self.node(
            start,
            Kind::Binding {
                name,
                mutable,
                by_ref,
                subpattern,
            },
        )
    }

    fn pattern_list(&mut self, close: P) -> Result<(super::List, bool), LexError> {
        let mark = self.scratch.len();
        let mut comma = false;
        while !self.eat(close)? {
            let pattern = self.pattern(true)?;
            self.scratch.push(pattern);
            if self.eat(close)? {
                break;
            }
            self.expect(P::Comma)?;
            comma = true;
        }
        Ok((self.list(mark)?, comma))
    }

    fn literal_pattern(&mut self) -> Result<NodeId, LexError> {
        let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
        let negative = self.eat(P::Minus)?;
        let token = self.required()?;
        if negative && !matches!(token.kind, TokenKind::Integer | TokenKind::Float) {
            return Err(error(start, "negative pattern requires a numeric literal"));
        }
        let value = literal::interpret(self.cursor.source, token)?
            .ok_or_else(|| error(token.start, "expected literal pattern"))?;
        let id = u32::try_from(self.ast.literals.len())
            .map_err(|_| error(start, "too many literals"))?;
        self.ast.literals.push(value);
        let literal = self.node(token.start, Kind::Literal(id))?;
        if negative {
            self.node(
                start,
                Kind::Unary {
                    op: super::super::ast::Unary::Negate,
                    operand: literal,
                },
            )
        } else {
            Ok(literal)
        }
    }

    pub(super) fn ty(&mut self) -> Result<NodeId, LexError> {
        if self.depth >= 256 {
            return Err(self.error("syntax nesting exceeds 256")?);
        }
        self.depth += 1;
        let result = self.type_inner();
        self.depth -= 1;
        result
    }

    fn type_inner(&mut self) -> Result<NodeId, LexError> {
        let token = self
            .cursor
            .peek(0)?
            .ok_or_else(|| error(self.cursor.end(), "expected type"))?;
        let start = token.start;
        if self.word("_")? {
            return self.node(start, Kind::InferType);
        }
        if self.eat(P::Bang)? {
            return self.node(start, Kind::NeverType);
        }
        let mark = self.scratch.len();
        let flavor = if self.cursor.at(P::And)? || self.cursor.at(P::AndAnd)? {
            if self.cursor.at(P::AndAnd)? {
                self.cursor.split(P::And, P::And)?;
            } else {
                self.required()?;
            }
            if self
                .cursor
                .peek(0)?
                .is_some_and(|t| matches!(t.kind, TokenKind::Lifetime | TokenKind::RawLifetime))
            {
                let lifetime = self.required()?;
                let node = self.node(
                    lifetime.start,
                    Kind::Name(Span {
                        start: lifetime.start,
                        end: lifetime.end,
                    }),
                )?;
                self.scratch.push(node);
            }
            let mutable = self.word("mut")?;
            let ty = self.ty()?;
            self.scratch.push(ty);
            if mutable {
                TypeFlavor::RefMut
            } else {
                TypeFlavor::Ref
            }
        } else if self.eat(P::Star)? {
            let flavor = if self.word("const")? {
                TypeFlavor::PointerConst
            } else if self.word("mut")? {
                TypeFlavor::PointerMut
            } else {
                return Err(self.error("pointer type requires const or mut")?);
            };
            let ty = self.ty()?;
            self.scratch.push(ty);
            flavor
        } else if self.eat(P::OpenParen)? {
            let mut comma = false;
            while !self.eat(P::CloseParen)? {
                let ty = self.ty()?;
                self.scratch.push(ty);
                if self.eat(P::CloseParen)? {
                    break;
                }
                self.expect(P::Comma)?;
                comma = true;
            }
            if !comma && self.scratch.len() == mark + 1 {
                let child = self.scratch[mark];
                self.scratch.truncate(mark);
                return self.node(start, Kind::Group(child));
            }
            TypeFlavor::Tuple
        } else if self.eat(P::OpenBracket)? {
            let ty = self.ty()?;
            self.scratch.push(ty);
            let flavor = if self.eat(P::Semicolon)? {
                let count = self.expr(0, NORMAL)?;
                self.scratch.push(count);
                TypeFlavor::Array
            } else {
                TypeFlavor::Slice
            };
            self.expect(P::CloseBracket)?;
            flavor
        } else {
            let path = self.path()?;
            self.scratch.push(path);
            TypeFlavor::Path
        };
        let elements = self.list(mark)?;
        self.node(start, Kind::Type { flavor, elements })
    }
}
