//! Compiler expressions retain actual type/field roots and the name's gate site.
use super::{F, Kind, LexError, NodeId, P, Parser, Span, TokenKind, error};

impl Parser<'_, '_> {
    pub(super) fn builtin_expression(&mut self) -> Result<NodeId, LexError> {
        let start = self.required()?.start;
        self.expect(P::Pound)?;
        let name = self.required()?;
        if name.kind != TokenKind::Identifier || self.cursor.spelling(name) != "offset_of" {
            return Err(error(name.start, "unknown compiler builtin expression"));
        }
        self.expect(P::OpenParen)?;
        let container = self.ty()?;
        self.expect(P::Comma)?;
        let expression = self.offset_fields()?;
        let fields = self.captured_offset_fields(expression)?;
        self.eat(P::Comma)?;
        self.expect(P::CloseParen)?;
        let gate = self.ast.next_expression_gate()?;
        let node = self.node(
            start,
            Kind::OffsetOf {
                container,
                fields,
                expression,
                gate,
            },
        )?;
        self.ast.record_expression_gate(
            node,
            Span {
                start: name.start,
                end: name.end,
            },
        )?;
        Ok(node)
    }

    fn offset_fields(&mut self) -> Result<NodeId, LexError> {
        let mut fields = if let Some(root) =
            self.interpolated(&[F::Expression, F::Expression2021, F::Literal])?
        {
            root
        } else {
            let token = self.cursor.projection()?;
            if !self.field_name(token, self.cursor.last_word())? {
                return Err(error(
                    token.start,
                    "expected offset field name or tuple index",
                ));
            }
            self.name_span(Span {
                start: token.start,
                end: token.end,
            })?
        };
        while self.eat(P::Dot)? {
            let token = self.cursor.projection()?;
            if !self.field_name(token, self.cursor.last_word())? {
                return Err(error(
                    token.start,
                    "expected offset field name or tuple index",
                ));
            }
            fields = self.node(
                self.ast.node(fields).span.start,
                Kind::Field {
                    value: fields,
                    name: Span {
                        start: token.start,
                        end: token.end,
                    },
                },
            )?;
        }
        Ok(fields)
    }

    /// A literal-shaped tuple path has already been parsed in an opaque expr.
    /// Split only its decimal projection spelling; retain that original root.
    fn captured_offset_fields(&mut self, root: NodeId) -> Result<NodeId, LexError> {
        let mark = self.scratch.len();
        let mut base = root;
        while let Kind::Field { value, .. } = self.ast.node(base).kind {
            if value.get() >= base.get() {
                return Err(error(
                    self.ast.node(base).span.start,
                    "offset field edge changed",
                ));
            }
            self.scratch.push(base);
            base = value;
        }
        let literal = self.ast.node(base);
        if matches!(literal.kind, Kind::Literal(_) | Kind::DeferredLiteral(_)) {
            let mut start = literal.span.start;
            let spelling = self
                .cursor
                .source
                .get(start as usize..literal.span.end as usize)
                .ok_or_else(|| error(start, "offset literal source disappeared"))?;
            let mut normalized = None;
            for part in spelling.split('.') {
                if part.is_empty()
                    || !part.bytes().all(|byte| byte.is_ascii_digit())
                    || (part.len() > 1 && part.starts_with('0'))
                {
                    return Err(error(
                        start,
                        "offset tuple field requires an unsuffixed decimal index",
                    ));
                }
                let end = start
                    .checked_add(
                        u32::try_from(part.len())
                            .map_err(|_| error(start, "offset tuple field too long"))?,
                    )
                    .ok_or_else(|| error(start, "offset tuple field span overflow"))?;
                let span = Span { start, end };
                let kind = normalized.map_or(Kind::Name(span, None), |value| Kind::Field {
                    value,
                    name: span,
                });
                let node = self.node(literal.span.start, kind)?;
                self.ast.nodes[node.get() as usize].span.end = end;
                normalized = Some(node);
                start = end.saturating_add(1);
            }
            base = normalized.ok_or_else(|| error(start, "offset tuple field is empty"))?;
            while self.scratch.len() > mark {
                let old = self
                    .scratch
                    .pop()
                    .ok_or_else(|| error(start, "offset field suffix disappeared"))?;
                let source = self.ast.node(old);
                let Kind::Field { name, .. } = source.kind else {
                    return Err(error(start, "offset field suffix changed"));
                };
                base = self.node(source.span.start, Kind::Field { value: base, name })?;
                self.ast.nodes[base.get() as usize].span = source.span;
            }
        } else {
            self.scratch.truncate(mark);
            base = root;
        }
        super::super::offset_fields::visit(&self.ast, base, |_, _| Ok(()))?;
        Ok(base)
    }
}
