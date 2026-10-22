//! Early invariant inputs reuse the variance solver without assuming unfinished callees.
use super::{Definitions, Graph, ItemId, Kind, Store, TypeId, Variance};
use crate::support::hash::HashMap;
#[cfg(test)]
#[path = "../../../tests/invariants/invariant_inputs.rs"]
mod tests;

#[derive(Default)]
pub(crate) struct RigidInputs {
    completed: HashMap<ItemId, bool>,
    steps: usize,
}
impl RigidInputs {
    /// A true result proves every input of a projected-field owner invariant.
    /// False supplies no variance or coercion permission. Only completed raw
    /// templates are cached; external nominal edges remain unknown here.
    pub(crate) fn demand(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        item: ItemId,
        template: Option<TypeId>,
    ) -> Result<bool, &'static str> {
        self.charge(1)?;
        if let Some(&rigid) = self.completed.get(&item) {
            return Ok(rigid);
        }
        let Some(definition) = definitions.get(item) else {
            return Ok(false);
        };
        let variants = definitions.variants(definition);
        let mut projected = false;
        for variant in variants {
            let fields = types
                .list(variant.fields)
                .ok_or("invalid invariant field template")?;
            self.charge(fields.len() + 1)?;
            projected |= fields
                .iter()
                .any(|&ty| types.contains_projections(ty) == Some(true));
        }
        if !projected {
            self.completed.insert(item, false);
            return Ok(false);
        }
        let Some(Kind::Adt(owner, arguments)) = template.and_then(|ty| types.kind(ty)) else {
            return Err("missing invariant nominal schema");
        };
        if owner != item {
            return Err("invariant nominal owner differs from schema");
        }
        let mut graph = Graph {
            skip_external_nominals: true,
            ..Graph::default()
        };
        graph.declaration(types, item, arguments)?;
        for variant in variants {
            for &field in types
                .list(variant.fields)
                .ok_or("invalid invariant fields")?
            {
                graph.visit(types, item, field)?;
            }
        }
        graph.solver.solve()?;
        self.charge(graph.solver.steps)?;
        let rigid = graph
            .variables
            .iter()
            .all(|&slot| graph.solver.value(slot) == Variance::Invariant);
        self.completed.insert(item, rigid);
        Ok(rigid)
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|steps| *steps <= 4_000_000)
            .ok_or("invariant input work budget exceeded")?;
        Ok(())
    }
}
