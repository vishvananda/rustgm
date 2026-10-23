//! One selected constructor condition serves ordinary and guarded retry decisions.
use super::{Builder, Error, Place, Rvalue, ScalarId, ValueId};
use crate::{
    lex::token::Punctuation,
    sema::{
        model::Scalar,
        patterns::{Kind, PatternId},
    },
    types::{Primitive, TypeId},
};

impl Builder<'_> {
    pub(super) fn pattern_condition(
        &mut self,
        id: PatternId,
        place: Place,
    ) -> Result<Option<ValueId>, Error> {
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let boolean = TypeId::primitive(Primitive::Bool);
        let value = match pattern.kind {
            Kind::Range(range) => {
                return self.interval_test(
                    node,
                    pattern
                        .ty
                        .ok_or_else(|| self.error(node, "missing pattern input type"))?,
                    place,
                    self.facts.patterns.ranges[range.0 as usize],
                );
            }
            Kind::Text(text) => self.evaluate(node, boolean, Rvalue::TextEquals { place, text })?,
            Kind::Slice { minimum, exact, .. } => {
                let ty = TypeId::primitive(Primitive::Usize);
                let length = self.evaluate(node, ty, Rvalue::Length(place))?;
                let scalar = ScalarId(self.index(node, self.body.scalars.len())?);
                self.body.scalars.push(Scalar::Integer(u128::from(minimum)));
                let minimum = self.evaluate(node, ty, Rvalue::Scalar(scalar))?;
                self.evaluate(
                    node,
                    boolean,
                    Rvalue::Binary {
                        operands: crate::body::ir::BinaryOperands::Neither,
                        operator: if exact {
                            Punctuation::EqualEqual
                        } else {
                            Punctuation::GreaterEqual
                        },
                        left: length,
                        right: minimum,
                    },
                )?
            }
            Kind::Product {
                variant: Some(variant),
                ..
            } if self
                .facts
                .members
                .variants(self.facts.members.variants[variant.0 as usize].owner)
                .len()
                > 1 =>
            {
                self.evaluate(node, boolean, Rvalue::IsVariant { place, variant })?
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
