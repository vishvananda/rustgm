//! Concatenation consumes the retained template forest, never rendered syntax.
use super::{
    Atom, Compiler, Delimiter, Kind, LexError, List, NodeId, P, Span, Token, TokenKind, TreeKind,
    error,
};

impl Compiler<'_> {
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
            if !self.by_name.contains_key(&name) {
                return Err(error(
                    identifier.start,
                    "concat references an unknown metavariable",
                ));
            }
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
