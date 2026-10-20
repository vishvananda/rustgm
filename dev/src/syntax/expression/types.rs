//! Type paths, binders and generic argument syntax; no name/type resolution.
use super::{Kind, LexError, List, NORMAL, NodeId, P, Parser, Span, TokenKind, error, keyword};
use crate::syntax::{ast::TypeFlavor, item::GenericFlavor};

use super::fragment::FragmentKind as F;

impl Parser<'_, '_> {
    pub(super) fn empty_list() -> List {
        List { start: 0, end: 0 }
    }

    pub(super) fn name(&mut self) -> Result<Span, LexError> {
        let token = self.required()?;
        let text = self.cursor.spelling(token);
        if !keyword::identifier(token.kind, text.strip_prefix("r#").unwrap_or(text)) {
            return Err(error(token.start, "expected identifier"));
        }
        Ok(Span {
            start: token.start,
            end: token.end,
        })
    }

    pub(super) fn at_angle_end(&mut self) -> Result<bool, LexError> {
        Ok(self.cursor.at(P::Greater)?
            || self.cursor.at(P::ShiftRight)?
            || self.cursor.at(P::GreaterEqual)?
            || self.cursor.at(P::ShiftRightEqual)?)
    }
    pub(super) fn angle_end(&mut self) -> Result<(), LexError> {
        if self.eat(P::Greater)? {
            return Ok(());
        }
        let rest = if self.cursor.at(P::ShiftRight)? {
            P::Greater
        } else if self.cursor.at(P::GreaterEqual)? {
            P::Equal
        } else if self.cursor.at(P::ShiftRightEqual)? {
            P::GreaterEqual
        } else {
            return Err(self.error("expected closing generic angle")?);
        };
        self.cursor.split(P::Greater, rest)?;
        Ok(())
    }
    fn angle_start(&mut self) -> Result<(), LexError> {
        if self.cursor.at(P::LeftArrow)? {
            self.cursor.split(P::Less, P::Minus)?;
        } else if self.cursor.at(P::ShiftLeft)? {
            self.cursor.split(P::Less, P::Less)?;
        } else {
            self.expect(P::Less)?;
        }
        Ok(())
    }

    pub(super) fn rich_path(&mut self, type_position: bool) -> Result<NodeId, LexError> {
        if let Some(node) = self.interpolated(&[F::Path])? {
            return Ok(node);
        }
        if type_position && let Some(node) = self.interpolated(&[F::Type])? {
            if let Kind::Type {
                flavor: TypeFlavor::Path,
                elements,
            } = self.ast.node(node).kind
                && let [path] = self.ast.list(elements)
            {
                return Ok(*path);
            }
            return Err(error(
                self.ast.node(node).span.start,
                "type fragment is not a path",
            ));
        }
        let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
        if self.cursor.at(P::Less)? || self.cursor.at(P::ShiftLeft)? {
            self.angle_start()?;
            let ty = self.ty()?;
            let trait_path = if self.word("as")? {
                Some(self.rich_path(true)?)
            } else {
                None
            };
            self.angle_end()?;
            self.expect(P::ColonColon)?;
            let path = self.rich_path(type_position)?;
            return self.node(
                start,
                Kind::Qualified {
                    ty,
                    trait_path,
                    path,
                },
            );
        }
        let absolute = self.eat(P::ColonColon)?;
        let mark = self.scratch.len();
        loop {
            let token = self.required()?;
            let mut segment = if let TokenKind::CrateRoot(crate_id) = token.kind {
                if absolute || self.scratch.len() != mark {
                    return Err(error(token.start, "crate marker must begin a path"));
                }
                self.node(token.start, Kind::CrateRoot(crate_id.get()))?
            } else {
                let text = self.cursor.spelling(token);
                let name = text.strip_prefix("r#").unwrap_or(text);
                if !(keyword::identifier(token.kind, name)
                    || (token.kind == TokenKind::Identifier
                        && matches!(name, "Self" | "self" | "super" | "crate")))
                {
                    return Err(error(token.start, "expected path segment"));
                }
                self.node(
                    token.start,
                    Kind::Name(Span {
                        start: token.start,
                        end: token.end,
                    }),
                )?
            };
            let separator = self.cursor.at(P::ColonColon)?
                && self.cursor.peek(1)?.is_some_and(|next| {
                    matches!(
                        next.kind,
                        TokenKind::Punctuation(
                            P::Less | P::LeftArrow | P::ShiftLeft | P::OpenParen
                        )
                    )
                });
            if separator {
                self.required()?;
            }
            if (type_position || separator)
                && (self.cursor.at(P::Less)?
                    || self.cursor.at(P::LeftArrow)?
                    || self.cursor.at(P::ShiftLeft)?
                    || self.cursor.at(P::OpenParen)?)
            {
                let (arguments, output, parenthesized) = self.generic_arguments()?;
                segment = self.node(
                    token.start,
                    Kind::Applied {
                        base: segment,
                        arguments,
                        output,
                        parenthesized,
                    },
                )?;
            }
            self.scratch.push(segment);
            if !self.eat(P::ColonColon)? {
                break;
            }
        }
        let segments = self.list(mark)?;
        self.node(start, Kind::Path { absolute, segments })
    }

    pub(super) fn generic_arguments(&mut self) -> Result<(List, Option<NodeId>, bool), LexError> {
        let mark = self.scratch.len();
        if self.eat(P::OpenParen)? {
            while !self.eat(P::CloseParen)? {
                let ty = self.ty()?;
                self.scratch.push(ty);
                if self.eat(P::CloseParen)? {
                    break;
                }
                self.expect(P::Comma)?;
            }
            let arguments = self.list(mark)?;
            let output = if self.eat(P::Arrow)? {
                Some(self.ty()?)
            } else {
                None
            };
            return Ok((arguments, output, true));
        }
        self.angle_start()?;
        while !self.at_angle_end()? {
            let token = self
                .cursor
                .peek(0)?
                .ok_or_else(|| error(self.cursor.end(), "expected generic argument"))?;
            let argument = if matches!(token.kind, TokenKind::Lifetime | TokenKind::RawLifetime) {
                self.required()?;
                self.node(
                    token.start,
                    Kind::Name(Span {
                        start: token.start,
                        end: token.end,
                    }),
                )?
            } else if self.cursor.at(P::OpenBrace)?
                || self.cursor.at(P::Minus)?
                || crate::literal::is_token(self.cursor.source, token)
            {
                self.expr(131, NORMAL)?
            } else {
                let target = self.ty()?;
                if self.eat(P::Equal)? {
                    let value = self.ty()?;
                    self.node(
                        token.start,
                        Kind::Constraint {
                            target,
                            value: Some(value),
                            bounds: Self::empty_list(),
                        },
                    )?
                } else if self.eat(P::Colon)? {
                    let bounds = self.bounds()?;
                    self.node(
                        token.start,
                        Kind::Constraint {
                            target,
                            value: None,
                            bounds,
                        },
                    )?
                } else {
                    target
                }
            };
            self.scratch.push(argument);
            if self.at_angle_end()? {
                break;
            }
            self.expect(P::Comma)?;
        }
        self.angle_end()?;
        Ok((self.list(mark)?, None, false))
    }

    pub(super) fn rich_type(&mut self) -> Result<NodeId, LexError> {
        let token = self
            .cursor
            .peek(0)?
            .ok_or_else(|| error(self.cursor.end(), "expected type"))?;
        let start = token.start;
        if self.cursor.word("dyn")? || self.cursor.word("impl")? {
            let opaque = self.word("impl")?;
            if !opaque {
                self.required()?;
            }
            let bounds = self.bounds()?;
            if bounds.start == bounds.end {
                return Err(error(start, "trait type requires a bound"));
            }
            return self.node(start, Kind::TraitType { opaque, bounds });
        }
        if self.cursor.word("for")?
            || self.cursor.word("unsafe")?
            || self.cursor.word("extern")?
            || self.cursor.word("fn")?
        {
            return self.function_type();
        }
        if matches!(
            token.kind,
            TokenKind::Punctuation(
                P::And | P::AndAnd | P::Star | P::OpenParen | P::OpenBracket | P::Bang
            )
        ) || self.cursor.word("_")?
        {
            return self.type_inner();
        }
        let path = self.rich_path(true)?;
        if self.eat(P::Bang)? {
            return self.macro_capture(path, crate::syntax::ast::MacroContext::Type);
        }
        let mark = self.scratch.len();
        self.scratch.push(path);
        let elements = self.list(mark)?;
        self.node(
            start,
            Kind::Type {
                flavor: TypeFlavor::Path,
                elements,
            },
        )
    }

    pub(super) fn generics(&mut self) -> Result<List, LexError> {
        if self.depth >= self.nesting_limit() {
            return Err(self.error(if self.extended {
                "crate syntax nesting exceeds 128"
            } else {
                "syntax nesting exceeds 256"
            })?);
        }
        self.depth += 1;
        let result = self.generic_parameters();
        self.depth -= 1;
        result
    }
    fn generic_parameters(&mut self) -> Result<List, LexError> {
        let mark = self.scratch.len();
        if !self.eat(P::Less)? {
            return self.list(mark);
        }
        while !self.at_angle_end()? {
            let attrs = self.attributes(false)?;
            let token = self
                .cursor
                .peek(0)?
                .ok_or_else(|| error(self.cursor.end(), "expected generic parameter"))?;
            let (flavor, name, ty) =
                if matches!(token.kind, TokenKind::Lifetime | TokenKind::RawLifetime) {
                    self.required()?;
                    (
                        GenericFlavor::Lifetime,
                        Span {
                            start: token.start,
                            end: token.end,
                        },
                        None,
                    )
                } else if self.word("const")? {
                    let name = self.name()?;
                    self.expect(P::Colon)?;
                    (GenericFlavor::Const, name, Some(self.ty()?))
                } else {
                    (GenericFlavor::Type, self.name()?, None)
                };
            let bounds = if self.eat(P::Colon)? {
                self.bounds()?
            } else {
                Self::empty_list()
            };
            let default = if self.eat(P::Equal)? {
                Some(if flavor == GenericFlavor::Const {
                    self.expr(131, NORMAL)?
                } else {
                    self.ty()?
                })
            } else {
                None
            };
            let index = u32::try_from(self.ast.generics.len())
                .map_err(|_| error(token.start, "too many generic parameters"))?;
            self.ast
                .generics
                .push(crate::syntax::item::GenericParameter {
                    flavor,
                    name,
                    ty,
                    bounds,
                    default,
                });
            let parameter = self.node(token.start, Kind::Generic(index))?;
            let parameter = self.attach(attrs, parameter)?;
            self.scratch.push(parameter);
            if self.at_angle_end()? {
                break;
            }
            self.expect(P::Comma)?;
        }
        self.angle_end()?;
        self.list(mark)
    }

    pub(super) fn binders(&mut self) -> Result<List, LexError> {
        if !self.word("for")? {
            return Ok(Self::empty_list());
        }
        if !self.cursor.at(P::Less)? {
            return Err(self.error("for binder requires lifetime parameters")?);
        }
        let binders = self.generics()?;
        for id in self.ast.list(binders) {
            if !matches!(self.ast.node(*id).kind, Kind::Generic(index) if self.ast.generics[index as usize].flavor == GenericFlavor::Lifetime)
            {
                return Err(error(
                    self.ast.node(*id).span.start,
                    "for binder requires lifetime parameters",
                ));
            }
        }
        Ok(binders)
    }

    pub(super) fn bounds(&mut self) -> Result<List, LexError> {
        let mark = self.scratch.len();
        while let Some(token) = self.cursor.peek(0)? {
            if matches!(
                token.kind,
                TokenKind::Punctuation(
                    P::Comma
                        | P::Greater
                        | P::ShiftRight
                        | P::GreaterEqual
                        | P::ShiftRightEqual
                        | P::Equal
                        | P::Semicolon
                        | P::OpenBrace
                        | P::CloseParen
                )
            ) || self.cursor.word("where")?
            {
                break;
            }
            let grouped = self.eat(P::OpenParen)?;
            if self.word("use")? {
                if grouped {
                    return Err(error(
                        token.start,
                        "precise capture bound cannot be parenthesized",
                    ));
                }
                let captures = self.precise_captures(token.start)?;
                self.scratch.push(captures);
                if !self.eat(P::Plus)? {
                    break;
                }
                continue;
            }
            let optional = self.eat(P::Question)?;
            let quantified = self.cursor.word("for")?;
            let binders = self.binders()?;
            if quantified && (optional || self.cursor.at(P::Question)?) {
                return Err(self.error("higher-ranked bounds cannot relax trait requirements")?);
            }
            if quantified
                && self.cursor.peek(0)?.is_some_and(|token| {
                    matches!(token.kind, TokenKind::Lifetime | TokenKind::RawLifetime)
                })
            {
                return Err(self.error("higher-ranked bound requires a trait")?);
            }
            let value = if self
                .cursor
                .peek(0)?
                .is_some_and(|t| matches!(t.kind, TokenKind::Lifetime | TokenKind::RawLifetime))
            {
                let t = self.required()?;
                self.node(
                    t.start,
                    Kind::Name(Span {
                        start: t.start,
                        end: t.end,
                    }),
                )?
            } else {
                self.rich_path(true)?
            };
            if grouped {
                self.expect(P::CloseParen)?;
            }
            let bound = self.node(
                token.start,
                Kind::Bound {
                    optional,
                    quantified,
                    binders,
                    value,
                },
            )?;
            self.scratch.push(bound);
            if !self.eat(P::Plus)? {
                break;
            }
        }
        self.list(mark)
    }

    fn precise_captures(&mut self, start: u32) -> Result<NodeId, LexError> {
        self.expect(P::Less)?;
        let mark = self.scratch.len();
        while !self.at_angle_end()? {
            let token = self.required()?;
            let spelling = self.cursor.spelling(token);
            if !(matches!(token.kind, TokenKind::Lifetime | TokenKind::RawLifetime)
                || keyword::identifier(token.kind, spelling.strip_prefix("r#").unwrap_or(spelling))
                || (token.kind == TokenKind::Identifier && spelling == "Self"))
            {
                return Err(error(token.start, "expected captured generic parameter"));
            }
            let parameter = self.node(
                token.start,
                Kind::Name(Span {
                    start: token.start,
                    end: token.end,
                }),
            )?;
            self.scratch.push(parameter);
            if self.at_angle_end()? {
                break;
            }
            self.expect(P::Comma)?;
        }
        self.angle_end()?;
        let captures = self.list(mark)?;
        self.node(start, Kind::PreciseCapture(captures))
    }

    pub(super) fn where_clause(&mut self) -> Result<List, LexError> {
        let mark = self.scratch.len();
        if !self.word("where")? {
            return self.list(mark);
        }
        while !self.cursor.at(P::OpenBrace)?
            && !self.cursor.at(P::Semicolon)?
            && !self.cursor.at(P::Equal)?
        {
            let quantified = self.cursor.word("for")?;
            let binders = self.binders()?;
            let token = self
                .cursor
                .peek(0)?
                .ok_or_else(|| error(self.cursor.end(), "expected where predicate"))?;
            let ty = if matches!(token.kind, TokenKind::Lifetime | TokenKind::RawLifetime) {
                self.required()?;
                self.node(
                    token.start,
                    Kind::Name(Span {
                        start: token.start,
                        end: token.end,
                    }),
                )?
            } else {
                self.ty()?
            };
            self.expect(P::Colon)?;
            let bounds = self.bounds()?;
            let predicate = self.node(
                token.start,
                Kind::Where {
                    quantified,
                    binders,
                    ty,
                    bounds,
                },
            )?;
            self.scratch.push(predicate);
            if !self.eat(P::Comma)? {
                break;
            }
        }
        self.list(mark)
    }

    fn function_type(&mut self) -> Result<NodeId, LexError> {
        let start = self.cursor.peek(0)?.map_or(self.cursor.end(), |t| t.start);
        let binders = self.binders()?;
        let unsafe_ = self.word("unsafe")?;
        let abi = if self.word("extern")? {
            self.abi()?
        } else {
            None
        };
        if !self.word("fn")? {
            return Err(self.error("expected fn in function pointer type")?);
        }
        let parameters = self.function_parameters(true)?;
        let result = if self.eat(P::Arrow)? {
            Some(self.ty()?)
        } else {
            None
        };
        let index = u32::try_from(self.ast.function_types.len())
            .map_err(|_| error(start, "too many function pointer types"))?;
        self.ast
            .function_types
            .push(crate::syntax::item::FunctionType {
                unsafe_,
                abi,
                binders,
                parameters,
                result,
            });
        self.node(start, Kind::FunctionType(index))
    }
}
