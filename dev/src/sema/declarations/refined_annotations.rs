//! Refined annotation endpoints use the ordinary typed constant continuation.
use super::{Checker, Error, Kind, NodeId, NodeKey, Type, TypeId, regions};
use crate::{
    syntax::ast::TypeConstraint,
    types::{Constraint, Primitive, ScalarRange},
};
#[cfg(test)]
#[path = "../../../tests/invariants/constrained_scalars.rs"]
mod tests;

impl Checker<'_> {
    pub(super) fn scalar_pattern_inputs(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        let Kind::TypePattern {
            base,
            constraint: TypeConstraint::Pattern(pattern),
        } = self.kind(key)
        else {
            return Err(self.error(key, "scalar pattern continuation changed its annotation"));
        };
        self.pattern_charge(key, 1, 0)?;
        let base = self.ty(NodeKey { node: base, ..key })?;
        self.scalar_base(key, base)?;
        let (start, end) = self.scalar_endpoints(NodeKey {
            node: pattern,
            ..key
        })?;
        self.schedule_typed_constant(NodeKey { node: end, ..key }, base, context)?;
        self.schedule_typed_constant(NodeKey { node: start, ..key }, base, context)
    }
    pub(super) fn pattern_type(
        &mut self,
        key: NodeKey,
        base: NodeId,
        constraint: TypeConstraint,
    ) -> Result<TypeId, Error> {
        let base = self.ty(NodeKey { node: base, ..key })?;
        let constraint = match constraint {
            TypeConstraint::NonNull => {
                if !matches!(self.facts.types.kind(base), Some(Type::Pointer { .. })) {
                    return Err(self.error(
                        key,
                        "non-pointer non-null patterns require their wider profile",
                    ));
                }
                Constraint::NonNull
            }
            TypeConstraint::Pattern(pattern) => {
                let (start, end) = self.scalar_endpoints(NodeKey {
                    node: pattern,
                    ..key
                })?;
                let start = self.pattern_endpoint(NodeKey { node: start, ..key }, base)?;
                let end = self.pattern_endpoint(NodeKey { node: end, ..key }, base)?;
                let producer = self
                    .report
                    .lint_producer()
                    .ok_or_else(|| self.error(key, "scalar pattern lost its source payer"))?;
                let pay = |steps, bytes| {
                    producer
                        .diagnostic_payload(key, steps, bytes)
                        .map_err(|e| e.message)
                };
                let id = self
                    .facts
                    .types
                    .intern_scalar_range(ScalarRange { base, start, end }, &pay)
                    .map_err(|e| self.error(key, e))?;
                return self
                    .facts
                    .types
                    .intern_observed::<true>(
                        Type::Pattern {
                            base,
                            constraint: Constraint::ScalarRange(id),
                        },
                        &pay,
                    )
                    .map_err(|e| self.error(key, e));
            }
        };
        self.intern(key, Type::Pattern { base, constraint })
    }
    fn scalar_base(&self, key: NodeKey, ty: TypeId) -> Result<(), Error> {
        if matches!(self.facts.types.kind(ty), Some(Type::Primitive(p)) if p == Primitive::Char || p.integer().is_some())
        {
            return Ok(());
        }
        Err(self.error(
            key,
            "scalar pattern requires a concrete integer or char base",
        ))
    }
    fn scalar_endpoints(&self, key: NodeKey) -> Result<(NodeId, NodeId), Error> {
        let Kind::PatternRange {
            start: Some(start),
            end: Some(end),
            inclusive: true,
        } = self.kind(key)
        else {
            return Err(self.error(
                key,
                "only closed inclusive scalar patterns have a complete consumer",
            ));
        };
        Ok((start, end))
    }
    fn pattern_endpoint(&self, key: NodeKey, ty: TypeId) -> Result<u128, Error> {
        let id = self.generic_constant(key, ty)?;
        self.facts
            .types
            .constant(id)
            .map(|value| value.bits)
            .ok_or_else(|| {
                self.error(
                    key,
                    "generic scalar pattern endpoints require their wider profile",
                )
            })
    }
    fn pattern_charge(&self, key: NodeKey, steps: usize, bytes: usize) -> Result<(), Error> {
        self.report
            .lint_producer()
            .ok_or_else(|| self.error(key, "scalar pattern lost its source payer"))?
            .diagnostic_payload(key, steps, bytes)
            .map_err(Error::from)
    }
}
