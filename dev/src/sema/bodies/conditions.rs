//! Conditional patterns retain input identities without requiring exhaustiveness.
use super::{Checker, Domain, Error, Kind, NodeId, TermId, TypeId, Work};
use crate::sema::patterns::Conditional;
impl Checker<'_> {
    pub(super) fn enter_let_else(
        &mut self,
        node: NodeId,
        pattern: NodeId,
        annotation: Option<NodeId>,
        value: NodeId,
        otherwise: NodeId,
    ) -> Result<(), Error> {
        let expected = if let Some(annotation) = annotation {
            self.annotation(annotation)?
        } else {
            self.variable(node, Domain::Any)?
        };
        let never = self.known(otherwise, TypeId::NEVER)?;
        self.state.work.push(Work::Enter(otherwise, Some(never)));
        self.state.work.push(Work::Conditional {
            node,
            pattern,
            value,
            expected: Some(expected),
        });
        self.state.work.push(Work::Enter(value, Some(expected)));
        Ok(())
    }
    pub(super) fn finish_let(
        &mut self,
        node: NodeId,
        pattern: NodeId,
        value: Option<NodeId>,
        otherwise: Option<NodeId>,
    ) -> Result<TermId, Error> {
        if let Some(otherwise) = otherwise {
            let never = self.known(otherwise, TypeId::NEVER)?;
            self.coerce(otherwise, never)?;
        }
        let mut diverges = false;
        if let Some(value) = value {
            diverges = self.state.diverges[value.get() as usize];
            // Compound patterns were constrained after their initializer
            // by BindPattern; a literal's node type may now be adjusted.
            if otherwise.is_none() && self.simple_pattern(pattern) {
                let expected = self.term(pattern)?;
                let refresh = self.partial_nominal_initializer_call(value, expected)?;
                self.coerce(value, expected)?;
                if refresh {
                    self.state.work.push(Work::RefineCallResult(value));
                }
            }
        }
        self.known(
            node,
            if diverges {
                TypeId::NEVER
            } else {
                TypeId::UNIT
            },
        )
    }

    pub(super) fn conditional_pattern(
        &mut self,
        node: NodeId,
        pattern: NodeId,
        value: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let term = if let Some(expected) = expected {
            self.coerce(value, expected)?;
            expected
        } else {
            let actual = self.term(value)?;
            if self.head(value, actual)? == Some(super::Constructor::Never) {
                let expected = self.variable(node, Domain::Any)?;
                self.coerce(value, expected)?;
                expected
            } else {
                actual
            }
        };
        let pattern = self.selected_pattern(pattern, term)?;
        if matches!(self.kind(node), Kind::Let { .. }) {
            self.check_static_binding(node, pattern, Some(value))?;
        }
        self.state.alignment.patterns.push((pattern, Some(value)));
        let otherwise = match self.kind(node) {
            Kind::Let { otherwise, .. } => otherwise.map(|n| self.key(n)),
            Kind::LetCondition { .. } => {
                self.facts.patterns.chains.insert(self.key(node));
                None
            }
            _ => return Err(self.error(node, "missing conditional pattern syntax")),
        };
        self.facts.patterns.conditions.insert(
            self.key(node),
            Conditional {
                input: self.key(value),
                pattern,
                otherwise,
            },
        );
        Ok(())
    }
}
