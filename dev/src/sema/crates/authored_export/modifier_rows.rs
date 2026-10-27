//! The same selected-source walk certifies explicit absence of written modifiers.
use super::{DefId, Stage, s};
use crate::{resolve::package::Error, support::retained::Growth};

pub(super) struct Draft {
    completed: Vec<Option<s::HeaderModifierRecord>>,
}
impl Draft {
    pub(super) fn new(stage: &mut Stage) -> Result<Self, Error> {
        let count = stage.rows.declarations.len();
        let growth = Growth::vector::<Option<s::HeaderModifierRecord>>(0, 0, count);
        stage
            .charge(count.saturating_add(growth.work))
            .map_err(Error::unlocated)?;
        stage
            .reserve(size_of::<Self>().saturating_add(growth.bytes))
            .map_err(Error::unlocated)?;
        Ok(Self {
            completed: vec![None; count],
        })
    }
    /// Called only after the actual item's typed default/final fence succeeded.
    pub(super) fn record(
        &mut self,
        stage: &mut Stage,
        definition: DefId,
        source: crate::resolve::unit::NodeKey,
    ) -> Result<(), Error> {
        stage.charge(2).map_err(Error::unlocated)?;
        let declaration = stage.declaration(definition).map_err(Error::unlocated)?;
        let site = stage.site(declaration, source).map_err(Error::unlocated)?;
        let row = self
            .completed
            .get_mut(declaration.0 as usize)
            .ok_or_else(|| Error::unlocated("modifier completion lost its actual declaration"))?;
        if row
            .replace(s::HeaderModifierRecord {
                declaration,
                site,
                policy: s::HeaderModifierPolicy::NoWrittenDefaultOrFinalOrCallerTracking,
            })
            .is_some()
        {
            return Err(Error::unlocated(
                "modifier completion repeated its actual source item",
            ));
        }
        Ok(())
    }
    pub(super) fn finish(mut self, stage: &mut Stage) -> Result<s::HeaderModifierColumns, Error> {
        let rows = &stage.rows;
        let mut growth = Growth::vector::<s::HeaderModifierRecord>(0, 0, rows.functions.len());
        growth.include(Growth::vector::<s::HeaderModifierRecord>(
            0,
            0,
            rows.traits.len(),
        ));
        growth.include(Growth::vector::<s::HeaderModifierRecord>(
            0,
            0,
            rows.implementations.len(),
        ));
        stage
            .charge(self.completed.len().saturating_add(growth.work))
            .map_err(Error::unlocated)?;
        stage
            .reserve(size_of::<s::HeaderModifierColumns>().saturating_add(growth.bytes))
            .map_err(Error::unlocated)?;
        let mut take = |declaration: crate::support::crates::owner_catalog::DeclarationRef| {
            self.completed
                .get_mut(declaration.0 as usize)
                .and_then(Option::take)
                .ok_or_else(|| {
                    Error::unlocated("modifier completion omitted an actual supported header")
                })
        };
        let functions = stage
            .rows
            .functions
            .iter()
            .map(|row| take(row.declaration))
            .collect::<Result<_, _>>()?;
        let traits = stage
            .rows
            .traits
            .iter()
            .map(|row| take(row.declaration))
            .collect::<Result<_, _>>()?;
        let implementations = stage
            .rows
            .implementations
            .iter()
            .map(|row| take(row.self_definition))
            .collect::<Result<_, _>>()?;
        if self.completed.iter().any(Option::is_some) {
            return Err(Error::unlocated(
                "modifier completion has no matching actual header",
            ));
        }
        Ok(s::HeaderModifierColumns {
            functions,
            traits,
            implementations,
        })
    }
}
