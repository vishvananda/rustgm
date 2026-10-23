//! Ordinary lifetime formation remains debt, independent of physical pointee shape.
use super::super::{Declarations, Node, Search, inference};
use crate::types::{
    Kind, RegionId, TypeId,
    infer::Context,
    lifetime_proof::ScopeId,
    outlives::{Pair, ParameterPair, ProjectionPair, Queries},
    parameters::predicates::{Bound, Predicate},
};
struct Debts {
    regions: Vec<Pair>,
    parameters: Vec<ParameterPair>,
    projections: Vec<ProjectionPair>,
}
fn collect(facts: &Declarations, context: &mut Context, ty: TypeId) -> Result<Debts, String> {
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let result = (|| {
        let debts = queries.symbolic_requirements(ty)?;
        context
            .account_work(
                debts
                    .scalars
                    .regions
                    .len()
                    .saturating_add(debts.scalars.parameters.len())
                    .saturating_add(debts.projections.len()),
            )
            .map_err(inference)?;
        Ok(Debts {
            regions: debts.scalars.regions.to_vec(),
            parameters: debts.scalars.parameters.to_vec(),
            projections: debts.projections.to_vec(),
        })
    })();
    context.account_work(queries.steps()).map_err(inference)?;
    result
}
impl Search<'_> {
    pub(super) fn unsizing_input_debts(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        goal: Predicate,
        scope: ScopeId,
        first: u32,
        end: u32,
    ) -> Result<Option<usize>, String> {
        context.account_work(1).map_err(inference)?;
        if facts.types.contains_regions(goal.ty) == Some(false)
            && facts.types.contains_parameters(goal.ty) == Some(false)
            && facts.types.contains_projections(goal.ty) == Some(false)
        {
            return Ok(None);
        }
        let debts = collect(facts, context, goal.ty)?;
        if debts.regions.is_empty() && debts.parameters.is_empty() && debts.projections.is_empty() {
            return Ok(None);
        }
        let count = debts
            .parameters
            .len()
            .checked_add(debts.projections.len())
            .ok_or("Unsize lifetime capacity overflow")?;
        context.account_work(count).map_err(inference)?;
        let mut types = Vec::<(TypeId, RegionId)>::with_capacity(count);
        for (parameter, shorter) in debts.parameters {
            types.push((facts.types.intern(Kind::Parameter(parameter))?, shorter));
        }
        let mut lifting = crate::sema::nominals::Requirements::default();
        for (projection, shorter) in debts.projections {
            let before = lifting.steps();
            let result = lifting.outlives(&mut facts.types, projection, shorter);
            context
                .account_work(lifting.steps().saturating_sub(before))
                .map_err(inference)?;
            if let Some(predicate) = result? {
                let Bound::Outlives(shorter) = predicate.bound else {
                    return Err("Unsize lifetime debt changed kind".into());
                };
                types.push((predicate.ty, shorter));
            }
        }
        if debts.regions.is_empty() && types.is_empty() {
            return Ok(None);
        }
        Ok(Some(self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: None,
                children: Vec::new(),
                first,
                end,
                equalities: Vec::new(),
                outlives: debts.regions,
                type_outlives: types,
                sized: Vec::new(),
            },
            scope,
            first..first,
        )))
    }
}
