//! Declaration property queries preserve projection proofs in their existing caller scope.
use super::State;
use crate::{
    sema::{model::Declarations, projections::Session},
    types::{TypeId, lifetime_proof::ScopeId},
};

impl State {
    pub(in crate::sema::nominals) fn normalize(
        &mut self,
        facts: &mut Declarations,
        original: TypeId,
    ) -> Result<TypeId, &'static str> {
        if let Some(&ty) = self.normalized.get(&original) {
            return Ok(ty);
        }
        let opened = if let Some(fold) = &mut self.fold {
            let before = fold.steps;
            let result = fold.apply(&mut facts.types, original)?;
            self.context
                .account_work(fold.steps.saturating_sub(before))
                .map_err(|_| "declaration normalization input budget exceeded")?;
            result
        } else {
            original
        };
        let mut session = Session::new(
            self.environment.clone(),
            &mut self.obligations,
            ScopeId::ROOT,
            self.availability,
            |_, _| Ok(None),
        )?;
        session.assume_placeholders(self.placeholders.clone())?;
        session.nest_at(self.nesting);
        let (normalized, steps) = session.normalize(facts, opened)?;
        self.context
            .account_nested(session.nested_work())
            .map_err(|_| "declaration normalization work budget exceeded")?;
        self.context
            .account_work(steps)
            .map_err(|_| "declaration normalization work budget exceeded")?;
        let mut restore = self.obligations.restoration()?;
        let result = restore.apply(&mut facts.types, normalized)?;
        self.context
            .account_work(restore.steps)
            .map_err(|_| "declaration normalization restoration budget exceeded")?;
        self.normalized.insert(original, result);
        Ok(result)
    }
}
