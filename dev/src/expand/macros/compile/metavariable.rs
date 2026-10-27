//! Concatenation consumes the retained template forest, never rendered syntax.
use super::super::model::RepetitionDepth;
use super::{
    Atom, Compiler, Delimiter, Kind, LexError, List, Mode, NodeId, P, Span, Token, TokenKind,
    TreeKind, error,
};

impl Compiler<'_> {
    pub(super) fn escaped_dollar(&mut self, token: Token, mode: Mode) -> Result<NodeId, LexError> {
        if mode == Mode::Matcher {
            return Err(error(token.start, "escaped dollar in a macro matcher"));
        }
        self.node(
            Span {
                start: token.start,
                end: token.end,
            },
            Kind::EscapedDollar(token),
        )
    }

    pub(super) fn expression(
        &mut self,
        dollar: Token,
        group: u32,
    ) -> Result<(NodeId, u32), LexError> {
        use super::super::model::Expression;
        let outer = self
            .forest
            .children(group)
            .ok_or_else(|| error(dollar.start, "metavariable expression requires braces"))?;
        let name = self
            .forest
            .name(self.source, outer.start)
            .ok_or_else(|| error(dollar.start, "metavariable expression requires a name"))?;
        if name == "concat" {
            return self.concat(dollar, group);
        }
        let arguments = outer.start + 1;
        if !matches!(
            self.forest.tree(arguments).map(|tree| tree.kind),
            Some(TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            })
        ) || self.forest.after(arguments) != outer.end
        {
            return Err(error(
                dollar.start,
                "metavariable expression requires one parenthesized argument list",
            ));
        }
        let range = self
            .forest
            .children(arguments)
            .ok_or_else(|| error(dollar.start, "metavariable expression lost its arguments"))?;
        let value = match name {
            "ignore" | "count" => {
                let ignore = name == "ignore";
                let (variable, mut next) = self.expression_variable(range.start, range.end)?;
                if ignore {
                    if next != range.end {
                        return Err(error(dollar.start, "ignore requires one metavariable"));
                    }
                    Expression::Ignore(variable)
                } else {
                    let depth = if next == range.end {
                        RepetitionDepth::new(0)
                    } else {
                        if !self.forest.punctuation(next, P::Comma) || next + 1 == range.end {
                            return Err(error(
                                dollar.start,
                                "count requires a metavariable and optional depth",
                            ));
                        }
                        next += 1;
                        self.expression_depth(next, range.end)?
                    };
                    Expression::Count {
                        name: variable,
                        depth,
                    }
                }
            }
            "index" => Expression::Index(self.expression_depth(range.start, range.end)?),
            "len" => Expression::Length(self.expression_depth(range.start, range.end)?),
            _ => return Err(error(dollar.start, "unsupported metavariable expression")),
        };
        let span = self
            .forest
            .tree(group)
            .ok_or_else(|| error(dollar.start, "missing metavariable expression group"))?
            .span;
        Ok((
            self.node(span, Kind::Expression(value))?,
            self.forest.after(group),
        ))
    }

    fn expression_variable(
        &mut self,
        index: u32,
        end: u32,
    ) -> Result<(crate::support::names::NameId, u32), LexError> {
        if index + 1 >= end || !self.forest.punctuation(index, P::Dollar) {
            return Err(error(
                self.forest.offset(index),
                "metavariable expression requires a dollar-prefixed variable",
            ));
        }
        let token = self.token(index + 1)?;
        if token.kind != TokenKind::Identifier {
            return Err(error(
                token.start,
                "metavariable expression requires a nonraw variable",
            ));
        }
        Ok((self.name(index + 1)?, index + 2))
    }

    fn expression_depth(&self, index: u32, end: u32) -> Result<RepetitionDepth, LexError> {
        if index == end {
            return Ok(RepetitionDepth::new(0));
        }
        let token = self.token(index)?;
        if index + 1 != end || token.kind != TokenKind::Integer {
            return Err(error(
                token.start,
                "metavariable expression depth requires one unsuffixed integer",
            ));
        }
        let Some(crate::literal::Value::Integer(value)) =
            crate::literal::interpret(self.source, token)?
        else {
            return Err(error(
                token.start,
                "metavariable expression depth is not an integer",
            ));
        };
        if value.suffix.is_some() {
            return Err(error(
                token.start,
                "metavariable expression depth must be unsuffixed",
            ));
        }
        u64::try_from(value.magnitude)
            .map(RepetitionDepth::new)
            .map_err(|_| error(token.start, "metavariable expression depth exceeds usize"))
    }

    pub(super) fn concat(&mut self, dollar: Token, group: u32) -> Result<(NodeId, u32), LexError> {
        let outer = self.forest.children(group).ok_or_else(|| {
            error(
                dollar.start,
                "metavariable expression requires a brace group",
            )
        })?;
        if self.forest.name(self.source, outer.start) != Some("concat") {
            return Err(error(dollar.start, "unsupported metavariable expression"));
        }
        let arguments = outer.start + 1;
        if !matches!(
            self.forest.tree(arguments).map(|tree| tree.kind),
            Some(TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            })
        ) || self.forest.after(arguments) != outer.end
        {
            return Err(error(
                dollar.start,
                "concat requires one parenthesized operand list",
            ));
        }
        let range = self
            .forest
            .children(arguments)
            .ok_or_else(|| error(dollar.start, "concat requires parenthesized operands"))?;
        let mark = self.scratch.len();
        let mut next = range.start;
        while next < range.end {
            let (node, after) = self.concat_operand(next, range.end)?;
            self.scratch.push(node);
            next = after;
            if next < range.end {
                if !self.forest.punctuation(next, P::Comma) || next + 1 == range.end {
                    return Err(error(
                        self.forest.offset(next),
                        "concat operands require commas without a trailing comma",
                    ));
                }
                next += 1;
            }
        }
        if self.scratch.len() - mark < 2 {
            return Err(error(dollar.start, "concat requires at least two operands"));
        }
        let start = u32::try_from(self.definition.edges.len())
            .map_err(|_| error(dollar.start, "too many macro edges"))?;
        self.definition.edges.extend(self.scratch.drain(mark..));
        let end = u32::try_from(self.definition.edges.len())
            .map_err(|_| error(dollar.start, "too many macro edges"))?;
        let span = self
            .forest
            .tree(group)
            .ok_or_else(|| error(dollar.start, "missing concat group"))?
            .span;
        Ok((
            self.node(span, Kind::Concat(List { start, end }))?,
            self.forest.after(group),
        ))
    }

    fn concat_operand(&mut self, index: u32, end: u32) -> Result<(NodeId, u32), LexError> {
        let token = self.token(index)?;
        if token.kind == TokenKind::Punctuation(P::Dollar) {
            if index + 1 >= end {
                return Err(error(
                    token.start,
                    "concat requires an actual metavariable name",
                ));
            }
            let identifier = self.token(index + 1)?;
            if identifier.kind != TokenKind::Identifier {
                return Err(error(
                    identifier.start,
                    "concat does not support raw metavariables",
                ));
            }
            let name = self.name(index + 1)?;
            return Ok((
                self.node(
                    Span {
                        start: token.start,
                        end: identifier.end,
                    },
                    Kind::Substitute {
                        name,
                        dollar: token,
                        identifier,
                    },
                )?,
                index + 2,
            ));
        }
        let spelling = self
            .source
            .get(token.start as usize..token.end as usize)
            .ok_or_else(|| error(token.start, "concat operand is outside its source"))?;
        let symbol = match token.kind {
            TokenKind::Identifier => spelling,
            TokenKind::String => spelling
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .ok_or_else(|| error(token.start, "concat string operands must not be suffixed"))?,
            _ => {
                return Err(error(
                    token.start,
                    "concat requires nonraw identifiers, cooked strings or metavariables",
                ));
            }
        };
        let name = self
            .names
            .intern_plain(symbol)
            .map_err(|e| error(token.start, e))?;
        Ok((
            self.node(
                Span {
                    start: token.start,
                    end: token.end,
                },
                Kind::Atom(Atom {
                    token,
                    name: Some(name),
                }),
            )?,
            index + 1,
        ))
    }
}
