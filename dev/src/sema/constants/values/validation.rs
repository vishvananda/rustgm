//! The outer completion point requires each ordinary closed initializer once.
use super::{Error, Evaluator, InitializerKind};

impl Evaluator<'_> {
    pub(crate) fn validate_ordinary_initializers(&mut self) -> Result<(), Error> {
        // Declaration/body readiness is complete. No query appends initializers;
        // keep their original order and reuse this batch's memoized values.
        let count = self.queries.typing.facts.initializers.len();
        for index in 0..count {
            let facts = &mut *self.queries.typing.facts;
            facts.constant_work.initializer_visits += 1;
            let initializer = facts.initializers[index];
            if facts.associated.owner(initializer.definition).is_some()
                || facts.generics.has_parameters(initializer.definition)
            {
                continue;
            }
            facts.constant_work.initializer_validations += 1;
            match initializer.kind {
                InitializerKind::Const => self.definition(initializer.definition)?,
                InitializerKind::Static { .. } => self.initializer(initializer.definition)?,
            };
        }
        Ok(())
    }
}
