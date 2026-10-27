//! Numeric expressions consume the same binding graph and active repeat stack.
use super::{Cardinalities, Count, LexError, Piece, Span, Writer, error};
use crate::expand::macros::model::Expression;
impl Writer<'_> {
    pub(super) fn metavariable(
        &mut self,
        expression: Expression,
        span: Span,
    ) -> Result<(), LexError> {
        let value = match expression {
            Expression::Ignore(name) => {
                if !self.variables.contains_key(&name) {
                    return Err(error(
                        span.start,
                        "ignore references an unknown metavariable",
                    ));
                }
                return Ok(());
            }
            Expression::Count { name, depth } => {
                let variable = *self
                    .variables
                    .get(&name)
                    .ok_or_else(|| error(span.start, "count references an unknown metavariable"))?;
                let query = Count {
                    definition: self.definition,
                    rule: self.rule,
                    variable,
                    path: &self.indices,
                    depth: depth.get(),
                };
                self.cardinalities
                    .get_or_insert_with(|| Box::new(Cardinalities::default()))
                    .count(&self.bindings, &query, &mut self.output.work)
                    .map_err(|failure| error(span.start, failure.message))?
            }
            Expression::Index(depth) | Expression::Length(depth) => {
                let frame = usize::try_from(depth.get())
                    .ok()
                    .and_then(|depth| depth.checked_add(1))
                    .and_then(|depth| self.indices.len().checked_sub(depth))
                    .and_then(|index| self.indices.get(index))
                    .ok_or_else(|| {
                        error(
                            span.start,
                            "metavariable index/len depth exceeds its actual repetition",
                        )
                    })?;
                if matches!(expression, Expression::Index(_)) {
                    frame.index
                } else {
                    frame.length
                }
            }
        };
        self.charge(span.start, value.checked_ilog10().unwrap_or(0) as usize + 1)?;
        self.output.pieces.push(Piece::Integer { value, span });
        Ok(())
    }
}
