//! Cold nominal dependencies share the ordinary solver and completed variance rows.
use super::{
    Argument, Definitions, GenericArgs, Graph, ItemId, Kind, ListId, Store, TypeId, Variance,
    Variances, solver::Term,
};
use crate::support::retained::{Growth, Payment};
#[cfg(test)]
#[path = "../../../tests/invariants/reachable_variance.rs"]
mod tests;

impl Variances {
    /// Complete only the new nominal dependency closure. Previously solved rows
    /// are immutable constants, so shared dependencies are not solved again.
    pub(crate) fn append_reached(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        declarations: &[Option<TypeId>],
        item: ItemId,
        mut fields: impl FnMut(ItemId, u32, ListId) -> ListId,
        pay: Payment<'_>,
    ) -> Result<(), &'static str> {
        pay(1, 0)?;
        if self.get(item).is_some() {
            return Ok(());
        }
        let mut graph = Graph::paid(self, declarations, pay)?;
        graph.reached_fields(types, definitions, item, &mut fields)?;
        graph.solver.pay(graph.variables.len(), 0)?;
        graph
            .solver
            .vector(&mut graph.result.values, graph.variables.len())?;
        graph
            .result
            .values
            .extend(graph.variables.iter().map(|&id| graph.solver.value(id)));
        let inferred = graph.result;
        let steps = self
            .steps
            .checked_add(graph.solver.steps)
            .filter(|steps| *steps <= 1_000_000)
            .ok_or("reached variance work budget exceeded")?;
        let mut growth = Growth::map::<ItemId, (usize, usize)>(
            self.ranges.len(),
            self.ranges.capacity(),
            inferred.ranges.len(),
        );
        growth.include(Growth::vector::<Variance>(
            self.values.len(),
            self.values.capacity(),
            inferred.values.len(),
        ));
        pay(
            growth
                .work
                .saturating_add(inferred.ranges.len())
                .saturating_add(inferred.values.len()),
            growth.bytes,
        )?;
        let base = self.values.len();
        self.values.extend(inferred.values);
        for (owner, (start, end)) in inferred.ranges {
            self.ranges.insert(owner, (base + start, base + end));
        }
        self.steps = steps;
        Ok(())
    }
}

impl Graph<'_> {
    fn reached_fields(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        item: ItemId,
        fields: &mut impl FnMut(ItemId, u32, ListId) -> ListId,
    ) -> Result<(), &'static str> {
        self.reserve_reached(types, item)?;
        while let Some(owner) = self.pending.pop() {
            let definition = definitions
                .get(owner)
                .ok_or("reached variance requires completed nominal fields")?;
            self.solver.pay(self.seen.capacity(), 0)?;
            self.seen.clear();
            if definition.kind == crate::types::aggregate::Kind::PhantomData {
                let arguments = declaration(
                    types,
                    self.declarations.ok_or("missing reached declarations")?,
                    owner,
                )?;
                let target = definitions.phantom_argument_from(types, owner, arguments)?;
                self.visit(types, owner, target)?;
            }
            for (ordinal, variant) in (0..).zip(definitions.variants(definition)) {
                self.solver.charge()?;
                let selected = fields(owner, ordinal, variant.fields);
                let selected = types
                    .list(selected)
                    .ok_or("missing reached variance fields")?;
                if selected.len()
                    != types
                        .list(variant.fields)
                        .ok_or("missing reached variance storage")?
                        .len()
                {
                    return Err("reached variance field count differs from storage");
                }
                for &ty in selected {
                    self.visit(types, owner, ty)?;
                }
            }
        }
        self.solver.solve()
    }
    pub(super) fn reserve_reached(
        &mut self,
        types: &Store,
        item: ItemId,
    ) -> Result<(), &'static str> {
        if self.result.ranges.contains_key(&item) {
            return Ok(());
        }
        let declarations = self
            .declarations
            .ok_or("missing reached nominal declarations")?;
        let arguments = declaration(types, declarations, item)?;
        self.declaration(types, item, arguments)?;
        self.solver.vector(&mut self.pending, 1)?;
        self.pending.push(item);
        Ok(())
    }

    pub(super) fn completed_nominal(
        &mut self,
        types: &Store,
        owner: ItemId,
        nominal: (ItemId, GenericArgs),
        position: Term,
        depth: u16,
        variances: &[Variance],
    ) -> Result<(), &'static str> {
        let actual = types
            .arguments(nominal.1)
            .ok_or("invalid completed variance arguments")?;
        let declarations = self
            .declarations
            .ok_or("missing completed variance declarations")?;
        let formal = declaration(types, declarations, nominal.0)?;
        let formal = types
            .arguments(formal)
            .ok_or("invalid completed variance formals")?;
        if actual.len() != variances.len() || formal.len() != variances.len() {
            return Err("incorrect completed variance arity");
        }
        for ((actual, formal), &variance) in actual.zip(formal).zip(variances) {
            self.solver.charge()?;
            let value = self.solver.compose(position, Term::constant(variance))?;
            match (actual, formal) {
                (Argument::Lifetime(region), Argument::Lifetime(_)) => {
                    if let Some(target) = self.region(owner, region, depth)? {
                        self.solver.constrain(target, value)?;
                    }
                }
                (Argument::Type(ty), Argument::Type(_)) => self.queue_type(ty, value, depth)?,
                (Argument::Const(id), Argument::Const(expected))
                    if types.const_type(id) == types.const_type(expected) =>
                {
                    self.constant(types, owner, id)?;
                }
                _ => return Err("incorrect completed variance argument kind"),
            }
        }
        Ok(())
    }
}

fn declaration(
    types: &Store,
    declarations: &[Option<TypeId>],
    item: ItemId,
) -> Result<GenericArgs, &'static str> {
    let ty = declarations
        .get(item.0 as usize)
        .copied()
        .flatten()
        .ok_or("missing canonical reached variance declaration")?;
    let Some(Kind::Adt(owner, arguments)) = types.kind(ty) else {
        return Err("reached variance declaration is not nominal");
    };
    if owner != item {
        return Err("reached variance declaration changed its nominal identity");
    }
    Ok(arguments)
}
