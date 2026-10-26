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
#[path = "assumed_scopes.rs"]
mod assumption_environments;
pub(in crate::sema::crates::typed_headers) use assumption_environments::ScopeReader;
#[path = "formal_ancestry.rs"]
mod formal_path;
#[path = "recorded_regions.rs"]
mod numbering_records;
#[path = "regional_occurrences.rs"]
mod occurrence_work;
#[cfg(test)]
#[path = "../../../../tests/invariants/prepared_callables.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
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
    scopes: std::rc::Rc<Vec<Option<usize>>>,
    ready: Vec<Option<Ready>>,
    occurrences: Vec<Option<u32>>,
    environments: Option<Box<assumption_environments::Owner>>,
    recordings: Option<Box<numbering_records::Owner>>,
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
                .saturating_add(size_of::<Vec<Option<usize>>>() + 2 * size_of::<usize>())
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
            scopes: std::rc::Rc::new(scopes),
            ready: vec![None; rows.functions.len()],
            occurrences: vec![None; rows.types.len()],
            environments: None,
            recordings: None,
        })
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.templates.allocation().bytes
            + retained::vector(&self.scopes)
            + size_of::<Vec<Option<usize>>>()
            + 2 * size_of::<usize>()
            + retained::vector(&self.ready)
            + retained::vector(&self.occurrences)
            + self
                .environments
                .as_deref()
                .map_or(0, assumption_environments::Owner::retained_bytes)
            + self
                .recordings
                .as_deref()
                .map_or(0, numbering_records::Owner::retained_bytes)
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
    fn require_in(
        &self,
        templates: &Templates,
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
                || templates
                    .registration(item)
                    .is_none_or(|(template, parameters)| {
                        template.signature != pair.callable
                            || template.parameters != 0
                            || parameters.len() != ready.formals
                    }))
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
        let (header, item, ready) =
            self.require_in(&self.templates, &view, store, owner, pair, work)?;
        state.require_mapping(&view, store, ready.is_none(), work)?;
        if ready.is_some() {
            return Ok(());
        }
        state.require_signatures(&view, store, header, pair, work)?;
        let before_bytes = self.retained_bytes();
        let before_budget = work.retained;
        let rows = view.rows()?;
        let formals = formal_path::collect(view, &self.scopes, &state.maps, store, owner, work)?;
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
        let result = prepare(
            &mut self.recipes,
            &mut self.facts.types,
            None,
            owner,
            work,
            |work| {
                self.inputs
                    .recipe_view(&self.inventory, self.provider_index, work)
            },
        )?;
        Ok((self, result))
    }
}

pub(super) fn registered_pair(
    state: &State,
    view: &RecipeView<'_>,
    facts: &crate::sema::model::Declarations,
    owner: DeclarationRef,
    pair: SignaturePair,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    let prepared = state
        .callables
        .as_deref()
        .ok_or_else(|| error("selected method lost its actual callable registration owner"))?;
    if prepared
        .require_in(
            &facts.lifetimes.callables,
            view,
            &facts.types,
            owner,
            pair,
            work,
        )?
        .2
        .is_none()
    {
        return Err(error(
            "selected method requires its actual pending callable registration",
        ));
    }
    Ok(())
}

pub(super) fn scope_reader(
    state: &State,
    view: &RecipeView<'_>,
    store: &Store,
    work: &mut ReservationWork,
) -> Result<ScopeReader, Error> {
    let prepared = state
        .callables
        .as_deref()
        .ok_or_else(|| error("selected scope reader omitted its actual callable owner"))?;
    application_owner(state)?.scope_reader(prepared, view, store, work)
}
pub(super) fn require_scope_reader(
    state: &State,
    reader: &ScopeReader,
    view: &RecipeView<'_>,
    store: &Store,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    let prepared = state
        .callables
        .as_deref()
        .ok_or_else(|| error("selected scope reader lost its actual callable owner"))?;
    application_owner(state)?.require_scope_reader(prepared, reader, view, store, work)
}
fn application_owner(state: &State) -> Result<&assumption_environments::Owner, Error> {
    state
        .callables
        .as_deref()
        .and_then(|prepared| prepared.environments.as_deref())
        .ok_or_else(|| {
            error("selected applications require their actual declaration environment owner")
        })
}
pub(super) fn application_reader(
    state: &State,
    view: &RecipeView<'_>,
    store: &Store,
    work: &mut ReservationWork,
) -> Result<super::super::predicate_remap::translation::applications::Reader, Error> {
    application_owner(state)?.application_reader(view, store, work)
}
pub(super) fn require_application_reader(
    state: &State,
    reader: &super::super::predicate_remap::translation::applications::Reader,
    view: &RecipeView<'_>,
    store: &Store,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    application_owner(state)?.require_application_reader(reader, view, store, work)
}

/// The consumer lends its one real table exclusively; buffer handles move and
/// are restored on every outcome. No second registration or table clone occurs.
pub(super) use assumption_environments::Declaration;
pub(super) fn declaration_ready(
    recipes: Option<&State>,
    view: &RecipeView<'_>,
    facts: &super::super::Declarations,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<bool, Error> {
    let Some(state) = recipes else {
        return Ok(false);
    };
    assumption_environments::declaration_ready(view, state, facts, owner, work)
}
pub(super) fn declarations<'a>(
    recipes: &mut Option<Box<State>>,
    facts: &mut super::super::Declarations,
    owner: DeclarationRef,
    work: &mut ReservationWork,
    mut view: impl FnMut(&mut ReservationWork) -> Result<RecipeView<'a>, Error>,
) -> Result<Vec<Declaration>, Error> {
    let first = view(work)?;
    if recipes.is_none() {
        *recipes = Some(Box::new(State::new(&first, &mut facts.types, work)?));
    }
    let state = recipes
        .as_mut()
        .ok_or_else(|| error("declaration lost its persistent recipe state"))?;
    state.require_mapping(&first, &facts.types, true, work)?;
    if state.callables.is_none() {
        state.callables = Some(Box::new(Prepared::new(&first, work)?));
    }
    let mut prepared = state
        .callables
        .take()
        .ok_or_else(|| error("declaration lost its callable scope index"))?;
    let result =
        assumption_environments::declarations(first, state, facts, &mut prepared, owner, work);
    state.callables = Some(prepared);
    result
}
pub(super) fn mark_declarations(
    recipes: &mut Option<Box<State>>,
    rows: &[Declaration],
) -> Result<(), Error> {
    let prepared = recipes
        .as_mut()
        .and_then(|state| state.callables.as_mut())
        .ok_or_else(|| error("installed declaration lost its actual prepared index"))?;
    assumption_environments::mark_installed(prepared, rows)
}
pub(super) fn prepare<'a>(
    recipes: &mut Option<Box<State>>,
    store: &mut Store,
    target: Option<&mut Templates>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
    mut view: impl FnMut(&mut ReservationWork) -> Result<RecipeView<'a>, Error>,
) -> Result<Demand<()>, Error> {
    let first = view(work)?;
    if recipes.is_none() {
        *recipes = Some(Box::new(State::new(&first, store, work)?));
    }
    let state = recipes
        .as_mut()
        .ok_or_else(|| error("private callable recipe state disappeared"))?;
    let pair = match state.pair(first, store, owner, work)? {
        Demand::Ready(pair) => pair,
        Demand::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
    };
    let second = view(work)?;
    if state.callables.is_none() {
        state.callables = Some(Box::new(Prepared::new(&second, work)?));
    }
    let mut prepared = state
        .callables
        .take()
        .ok_or_else(|| error("private callable preparation disappeared"))?;
    let result = if let Some(target) = target {
        work.charge(4)?;
        if !prepared.templates.is_empty() {
            state.callables = Some(prepared);
            return Err(error(
                "selected callable retained a second registration table",
            ));
        }
        std::mem::swap(target, &mut prepared.templates);
        let result = prepared.register(second, state, store, owner, pair, work);
        std::mem::swap(target, &mut prepared.templates);
        result
    } else {
        prepared.register(second, state, store, owner, pair, work)
    };
    state.callables = Some(prepared);
    result?;
    Ok(Demand::Ready(()))
}
