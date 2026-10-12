//! Pattern and type atoms needed by PA4 bodies. PA5 owns full item/type grammar.
use super::{Kind, LexError, NORMAL, NodeId, P, Parser, Span, TokenKind, error};
use crate::{
    literal,
    syntax::ast::{PatternFlavor, TypeFlavor},
};

use super::fragment::FragmentKind as F;

impl Parser<'_, '_> {
    pub(super) fn pattern(&mut self, alternatives: bool) -> Result<NodeId, LexError> {
        if self.depth >= self.nesting_limit() {
            return Err(self.error(if self.extended {
                "crate syntax nesting exceeds 128"
            } else {
                "syntax nesting exceeds 256"
            })?);
        }
        self.depth += 1;
        let result = self.pattern_inner(alternatives);
        self.depth -= 1;
        result
    }

    fn pattern_inner(&mut self, alternatives: bool) -> Result<NodeId, LexError> {
        let leading = alternatives && self.eat(P::Or)?;
        let first = self.pattern_piece()?;
        if !alternatives || (!leading && !self.cursor.at(P::Or)?) {
            return Ok(first);
        }
        let mark = self.scratch.len();
        self.scratch.push(first);
        while self.eat(P::Or)? {
            let item = self.pattern_piece()?;
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

    #[expect(
        clippy::too_many_lines,
        reason = "cohesive pattern prefix dispatch retains modifier and path ownership within the audit limit"
    )]
    fn pattern_atom(&mut self) -> Result<NodeId, LexError> {
        if let Some(node) = self.interpolated(&[F::Pattern, F::PatternParameter, F::Literal])? {
            return Ok(node);
        }
        if let Some(node) = self.interpolated(&[F::Expression, F::Expression2021])? {
            let literal = match self.ast.node(node).kind {
                Kind::Unary {
                    op: super::Unary::Negate,
                    operand,
                } => operand,
                _ => node,
            };
            if !matches!(
                self.ast.node(literal).kind,
                Kind::Literal(_) | Kind::DeferredLiteral(_)
            ) {
                return Err(error(
                    self.ast.node(node).span.start,
                    "expression fragment is not a literal pattern",
                ));
            }
            return Ok(node);
        }
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
        if self.cursor.at(P::Minus)? || literal::is_token(self.cursor.source, token) {
            return self.literal_pattern();
        }
        let by_ref = self.word("ref")?;
        let mutable = self.word("mut")?;
        let opaque_path = self
            .splice()?
            .is_some_and(|splice| splice.syntax_kind() == Some(F::Path));
        let path = self.path()?;
        if self.extended && self.eat(P::Bang)? {
            return self.macro_capture(path, crate::syntax::ast::MacroContext::Pattern);
        }
        if self.extended && self.cursor.at(P::OpenBrace)? {
            if by_ref || mutable {
                return Err(error(start, "binding modifier requires an identifier"));
            }
            return self.struct_pattern(path);
        }
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
        if opaque_path {
            if by_ref || mutable {
                return Err(error(start, "binding modifier requires an identifier"));
            }
            return Ok(path);
        }
        let simple = match self.ast.node(path).kind {
            Kind::Path {
                absolute: false,
                segments,
            } if self.ast.list(segments).len() == 1 => {
                let segment = self.ast.node(self.ast.list(segments)[0]);
                if matches!(segment.kind, Kind::Name(_)) {
                    Some(segment.span)
                } else {
                    None
                }
            }
            _ => None,
        };
        let Some(name) = simple else {
            if by_ref || mutable {
                return Err(error(start, "binding modifier requires an identifier"));
            }
            return Ok(path);
        };
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

    fn pattern_piece(&mut self) -> Result<NodeId, LexError> {
        if !self.extended {
            return self.pattern_atom();
        }
        let token = self
            .cursor
            .peek(0)?
            .ok_or_else(|| error(self.cursor.end(), "expected pattern"))?;
        let start = if matches!(
            token.kind,
            TokenKind::Punctuation(P::DotDot | P::DotDotEqual)
        ) {
            None
        } else {
            Some(self.pattern_atom()?)
        };
        if self.cursor.at(P::DotDotDot)? {
            return Err(self.error("obsolete range pattern is unavailable in edition 2024")?);
        }
        let inclusive = self.cursor.at(P::DotDotEqual)?;
        if inclusive || self.cursor.at(P::DotDot)? {
            self.required()?;
            let end = if self.cursor.peek(0)?.is_some_and(|t| {
                matches!(
                    t.kind,
                    TokenKind::Identifier
                        | TokenKind::RawIdentifier
                        | TokenKind::Integer
                        | TokenKind::Float
                        | TokenKind::Character
                        | TokenKind::Byte
                        | TokenKind::Interpolated(_)
                        | TokenKind::CrateRoot(_)
                        | TokenKind::Punctuation(P::Minus | P::ColonColon | P::Less)
                )
            }) && !self.cursor.word("if")?
            {
                Some(self.pattern_atom()?)
            } else {
                None
            };
            if inclusive && end.is_none() {
                return Err(error(
                    token.start,
                    "inclusive pattern range requires an end",
                ));
            }
            for bound in [start, end].into_iter().flatten() {
                if !matches!(
                    self.ast.node(bound).kind,
                    Kind::Literal(_)
                        | Kind::DeferredLiteral(_)
                        | Kind::Path { .. }
                        | Kind::Qualified { .. }
                        | Kind::Unary {
                            op: super::Unary::Negate,
                            ..
                        }
                        | Kind::Binding {
                            mutable: false,
                            by_ref: false,
                            subpattern: None,
                            ..
                        }
                ) {
                    return Err(error(
                        self.ast.node(bound).span.start,
                        "invalid range pattern bound",
                    ));
                }
            }
            if start.is_none() && end.is_none() {
                return self.node(token.start, Kind::RestPattern);
            }
            return self.node(
                token.start,
                Kind::PatternRange {
                    start,
                    end,
                    inclusive,
                },
            );
        }
        start.ok_or_else(|| error(token.start, "expected pattern"))
    }

    fn struct_pattern(&mut self, path: NodeId) -> Result<NodeId, LexError> {
        self.expect(P::OpenBrace)?;
        let mark = self.scratch.len();
        let mut rest = false;
        while !self.eat(P::CloseBrace)? {
            let attrs = self.attributes(false)?;
            if self.eat(P::DotDot)? {
                rest = true;
                self.eat(P::Comma)?;
                self.expect(P::CloseBrace)?;
                break;
            }
            let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
            let by_ref = self.word("ref")?;
            let mutable = self.word("mut")?;
            let token = self.required()?;
            if !self.field_name(token) {
                return Err(error(token.start, "expected pattern field"));
            }
            let name = Span {
                start: token.start,
                end: token.end,
            };
            let pattern = if self.eat(P::Colon)? {
                if by_ref || mutable {
                    return Err(error(start, "field binding modifiers require shorthand"));
                }
                self.pattern(true)?
            } else {
                if token.kind == TokenKind::Integer {
                    return Err(error(
                        token.start,
                        "numeric pattern field requires a pattern",
                    ));
                }
                self.node(
                    start,
                    Kind::Binding {
                        name,
                        by_ref,
                        mutable,
                        subpattern: None,
                    },
                )?
            };
            let field = self.node(start, Kind::PatternField { name, pattern })?;
            let field = self.attach(attrs, field)?;
            self.scratch.push(field);
            if self.eat(P::CloseBrace)? {
                break;
            }
            self.expect(P::Comma)?;
        }
        let fields = self.list(mark)?;
        self.node(
            self.ast.node(path).span.start,
            Kind::StructPattern { path, fields, rest },
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

    pub(super) fn literal_pattern(&mut self) -> Result<NodeId, LexError> {
        if let Some(node) = self.interpolated(&[F::Literal])? {
            return Ok(node);
        }
        let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
        let negative = self.eat(P::Minus)?;
        let token = self.required()?;
        if negative && !matches!(token.kind, TokenKind::Integer | TokenKind::Float) {
            return Err(error(start, "negative pattern requires a numeric literal"));
        }
        if self.extended {
            if !literal::is_token(self.cursor.source, token) {
                return Err(error(token.start, "expected literal pattern"));
            }
            let literal = self.node(token.start, Kind::DeferredLiteral(token))?;
            return if negative {
                self.node(
                    start,
                    Kind::Unary {
                        op: super::Unary::Negate,
                        operand: literal,
                    },
                )
            } else {
                Ok(literal)
            };
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
        if let Some(node) = self.interpolated(&[F::Type])? {
            return Ok(node);
        }
        if self.depth >= self.nesting_limit() {
            return Err(self.error(if self.extended {
                "crate syntax nesting exceeds 128"
            } else {
                "syntax nesting exceeds 256"
            })?);
        }
        self.depth += 1;
        let result = if self.extended {
            self.rich_type()
        } else {
            self.type_inner()
        };
        self.depth -= 1;
        result
    }

    pub(super) fn type_inner(&mut self) -> Result<NodeId, LexError> {
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
