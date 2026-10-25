//! Private registration is structural readiness, never a published header or proof.
use super::super::{
    error,
    recipe_cursor::{Cursor, Demand},
    staged_headers,
};
use super::{
    Session,
    header_demands::{Progress, SignaturePair, State},
};
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::RecipeView},
        package::Error,
    },
    support::{
        crates::{generics::source as s, owner_catalog::DeclarationRef},
        retained,
    },
    types::{ItemId, Kind, Store, callable::Templates},
};
#[path = "formal_ancestry.rs"]
mod formal_path;
#[path = "regional_occurrences.rs"]
mod occurrence_work;
#[cfg(test)]
#[path = "../../../../tests/invariants/prepared_callables.rs"]
mod tests;

#[derive(Clone, Copy)]
struct Ready {
    pair: SignaturePair,
    header: s::FunctionHeader,
    original: Kind,
    callable: Kind,
    item: ItemId,
    formals: usize,
    scope: (usize, s::ParameterScope),
}
pub(super) struct Prepared {
    templates: Templates,
    scopes: Vec<Option<usize>>,
    ready: Vec<Option<Ready>>,
    occurrences: Vec<Option<u32>>,
}
impl Prepared {
    fn new(view: &RecipeView<'_>, work: &mut ReservationWork) -> Result<Self, Error> {
        let rows = view.rows()?;
        work.charge(
            rows.scopes
                .len()
                .saturating_add(rows.declarations.len())
                .saturating_add(rows.functions.len())
                .saturating_add(rows.types.len()),
        )?;
        work.reserve(
            size_of::<Self>()
                .saturating_add(
                    rows.declarations
                        .len()
                        .saturating_mul(size_of::<Option<usize>>()),
                )
                .saturating_add(
                    rows.functions
                        .len()
                        .saturating_mul(size_of::<Option<Ready>>()),
                )
                .saturating_add(rows.types.len().saturating_mul(size_of::<Option<u32>>())),
        )?;
        let mut scopes = vec![None; rows.declarations.len()];
        for (index, scope) in rows.scopes.iter().enumerate() {
            let cell = scopes
                .get_mut(scope.owner.0 as usize)
                .ok_or_else(|| error("private callable scope lost its actual declaration"))?;
            if cell.replace(index).is_some() {
                return Err(error("private callable repeats its actual formal scope"));
            }
        }
        Ok(Self {
            templates: Templates::default(),
            scopes,
            ready: vec![None; rows.functions.len()],
            occurrences: vec![None; rows.types.len()],
        })
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.templates.allocation().bytes
            + retained::vector(&self.scopes)
            + retained::vector(&self.ready)
            + retained::vector(&self.occurrences)
    }
    fn scope(
        &self,
        rows: s::ValidatedView<'_>,
        owner: DeclarationRef,
    ) -> Result<(usize, s::ParameterScope), Error> {
        let index = self
            .scopes
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("private callable lost its actual formal-scope index"))?;
        let scope = *rows.scopes.get(index).ok_or_else(|| {
            error("private callable formal-scope index is outside its actual rows")
        })?;
        if scope.owner != owner {
            return Err(error(
                "private callable formal-scope index changed its actual owner",
            ));
        }
        Ok((index, scope))
    }
    fn require(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        owner: DeclarationRef,
        pair: SignaturePair,
        work: &mut ReservationWork,
    ) -> Result<(s::FunctionHeader, ItemId, Option<Ready>), Error> {
        work.charge(10)?;
        let rows = view.rows()?;
        let header = *rows
            .functions
            .get(pair.function_row as usize)
            .ok_or_else(|| error("private callable lost its actual signature row"))?;
        if rows.header(owner).map_err(error)? != Some(s::HeaderRef::Function(pair.function_row))
            || header.declaration != owner
            || view.declaration(owner, work)? != pair.declaration
        {
            return Err(error(
                "private callable signature changed its actual catalog owner",
            ));
        }
        let body = match header.body {
            s::FunctionBody::Original(body) => Some(view.body(body, work)?),
            s::FunctionBody::Absent => None,
        };
        if body != pair.body {
            return Err(error(
                "private callable signature changed its actual body identity",
            ));
        }
        let item = ItemId(view.definition(owner, work)?.0);
        let scope = self.scope(rows, header.scope)?;
        let memo = self
            .ready
            .get(pair.function_row as usize)
            .copied()
            .ok_or_else(|| error("private callable readiness row is missing"))?;
        if let Some(ready) = memo
            && (ready.pair != pair
                || ready.header != header
                || ready.scope != scope
                || ready.item != item
                || store.kind(pair.original) != Some(ready.original)
                || store.kind(pair.callable) != Some(ready.callable)
                || self
                    .templates
                    .get(item)
                    .is_none_or(|t| t.signature != pair.callable || t.parameters != 0)
                || self
                    .templates
                    .generic_parameters(item)
                    .is_none_or(|p| p.len() != ready.formals))
        {
            return Err(error(
                "private callable readiness changed its sealed owner or signature",
            ));
        }
        Ok((header, item, memo))
    }
    fn register(
        &mut self,
        view: RecipeView<'_>,
        state: &mut State,
        store: &mut Store,
        owner: DeclarationRef,
        pair: SignaturePair,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        let (header, item, ready) = self.require(&view, store, owner, pair, work)?;
        if ready.is_some() {
            return Ok(());
        }
        let before_bytes = self.retained_bytes();
        let before_budget = work.retained;
        let rows = view.rows()?;
        let formals =
            formal_path::collect(view, &self.scopes, &mut state.maps, store, owner, work)?;
        if formals.owners.last() != Some(&item) || store.escaping_binders(pair.callable) != Some(0)
        {
            return Err(error(
                "private callable lacks its complete actual owner or closed binder",
            ));
        }
        let occurrence = if formals.parameters.is_empty() {
            occurrence_work::prepare(
                rows,
                &state.cursor,
                store,
                header.callable,
                pair.function_row,
                &mut self.occurrences,
                work,
            )?
        } else {
            0
        };
        let growth = self
            .templates
            .registration_allocation_bound(formals.parameters.len(), formals.owners.len());
        work.charge(
            formals
                .parameters
                .len()
                .saturating_add(formals.owners.len())
                .saturating_add(self.templates.allocation().visits)
                .saturating_add(1),
        )?;
        work.reserve(growth.saturating_add(occurrence))?;
        staged_headers::register_callable(
            store,
            &mut self.templates,
            &formals.owners,
            &formals.parameters,
            pair.callable,
        )?;
        let original = store
            .kind(pair.original)
            .ok_or_else(|| error("private callable original signature vanished"))?;
        let callable = store
            .kind(pair.callable)
            .ok_or_else(|| error("private callable public signature vanished"))?;
        work.charge(2)?;
        let scope = self.scope(rows, header.scope)?;
        self.ready[pair.function_row as usize] = Some(Ready {
            pair,
            header,
            original,
            callable,
            item,
            formals: formals.parameters.len(),
            scope,
        });
        let actual = self.retained_bytes().saturating_sub(before_bytes);
        if actual > work.retained.saturating_sub(before_budget) {
            return Err(error(
                "private callable exceeded its actual registration preflight",
            ));
        }
        // Successful transient buffers retire; peak and failed attempts survive.
        work.retained = before_budget.saturating_add(actual);
        Ok(())
    }
}
impl Session {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private callable readiness awaits defining-environment production; all Provided publication and execution guards remain closed"
        )
    )]
    pub(super) fn callable_template(
        mut self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Progress, Error> {
        self.require_owner(work)?;
        let view = self
            .inputs
            .recipe_view(&self.inventory, self.provider_index, work)?;
        if self.recipes.is_none() {
            self.recipes = Some(Box::new(State::new(&view, &mut self.facts.types, work)?));
        }
        let state = self
            .recipes
            .as_mut()
            .ok_or_else(|| error("private callable recipe state disappeared"))?;
        let pair = match state.pair(view, &mut self.facts.types, owner, work)? {
            Demand::Ready(pair) => pair,
            Demand::Blocked(dependency) => return Ok((self, Demand::Blocked(dependency))),
        };
        let view = self
            .inputs
            .recipe_view(&self.inventory, self.provider_index, work)?;
        if state.callables.is_none() {
            state.callables = Some(Box::new(Prepared::new(&view, work)?));
        }
        let mut prepared = state
            .callables
            .take()
            .ok_or_else(|| error("private callable preparation disappeared"))?;
        prepared.register(view, state, &mut self.facts.types, owner, pair, work)?;
        state.callables = Some(prepared);
        Ok((self, Demand::Ready(())))
    }
}
