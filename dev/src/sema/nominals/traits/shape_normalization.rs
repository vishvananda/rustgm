//! Nominal size proofs share a normalizer per actual callable scope.
use super::{Declarations, State};
use crate::support::hash::HashMap;
use crate::{
    sema::{
        nominals::sizes,
        projections::{Cursor, Progress},
    },
    types::{
        lifetime_proof::{Obligations, ScopeId},
        parameters::shape::{Request, Shapes},
        substitute::Bound,
    },
};
use std::sync::Arc;

struct Scope {
    sizes: sizes::Query,
    cursor: Cursor,
    owner: sizes::Owner,
}
impl Scope {
    fn physical(
        owner: &sizes::Owner,
        facts: &Declarations,
        shapes: &mut Shapes,
        request: Request,
        normalized: bool,
        context: &mut crate::types::infer::Context,
    ) -> Result<Option<Box<sizes::Physical>>, &'static str> {
        if request.depth != 0 {
            return Ok(None);
        }
        let (physical, work) =
            sizes::Physical::capture(owner, &facts.types, shapes, request.ty, normalized)?;
        transfer_work(context, work, "size normalization work budget exceeded")?;
        Ok(physical)
    }
    fn new(
        facts: &mut Declarations,
        mut owner: sizes::Owner,
        nesting: crate::sema::traits::selection::Nesting,
        anchor: (&Obligations, ScopeId),
        depth: u16,
        context: &mut crate::types::infer::Context,
    ) -> Result<Self, &'static str> {
        let mut cursor = Cursor::new(
            owner.environment.clone(),
            anchor.0,
            anchor.1,
            owner.availability,
        )?;
        cursor.assume_placeholders(owner.placeholders.clone())?;
        cursor.nest_at(nesting);
        owner.environment = scoped_environment(facts, owner.environment.as_ref(), depth, context)?;
        Ok(Self {
            sizes: sizes::Query::with_placeholders(
                owner.environment.clone(),
                owner.placeholders.clone(),
            ),
            cursor,
            owner,
        })
    }
}

fn transfer_work(
    context: &mut crate::types::infer::Context,
    work: usize,
    cause: &'static str,
) -> Result<(), &'static str> {
    context.account_nested(work).map_err(|_| cause)?;
    #[cfg(test)]
    crate::sema::traits::selection::record_nested_work(work);
    Ok(())
}

fn scoped_environment(
    facts: &mut Declarations,
    environment: Option<&Arc<crate::types::parameters::environment::Environment>>,
    depth: u16,
    context: &mut crate::types::infer::Context,
) -> Result<Option<Arc<crate::types::parameters::environment::Environment>>, &'static str> {
    environment
        .map(|environment| {
            if depth == 0 {
                return Ok(Arc::clone(environment));
            }
            let mut shift = Bound::shift(depth);
            let shifted = environment.substitute_regions(&mut facts.types, &mut shift)?;
            context
                .account_work(shift.steps)
                .map_err(|_| "size normalization environment budget exceeded")?;
            Ok(Arc::new(shifted))
        })
        .transpose()
}
#[derive(Default)]
pub(super) struct Sizes {
    mappings: HashMap<(Option<u32>, bool), Mapping>,
    queries: HashMap<ScopeId, Scope>,
    learned: Option<Box<sizes::Learned>>,
}

#[derive(Default)]
struct Mapping {
    scopes: HashMap<ScopeId, ScopeId>,
    pending: Vec<(ScopeId, u32)>,
}
impl Mapping {
    fn demand(
        &mut self,
        proof: &mut Obligations,
        shapes: &mut Shapes,
        normalized: bool,
        mut needed: ScopeId,
        count: Option<u32>,
    ) -> Result<ScopeId, &'static str> {
        if self.scopes.is_empty() {
            let root = if let Some(count) = count {
                proof.scope(ScopeId::ROOT, count)?
            } else {
                ScopeId::ROOT
            };
            self.scopes.insert(ScopeId::ROOT, root);
        }
        // A resumed body may already have unrelated completed shape scopes.
        // Map only this request's ancestors, never their earlier siblings.
        while !self.scopes.contains_key(&needed) {
            let (parent, count) = shapes.size_binder(normalized, needed)?;
            self.pending.push((needed, count));
            needed = parent;
        }
        let mut target = *self.scopes.get(&needed).ok_or("missing size proof root")?;
        for (source, count) in self.pending.drain(..).rev() {
            target = proof.scope(target, count)?;
            self.scopes.insert(source, target);
        }
        Ok(target)
    }
}

impl State {
    pub(in crate::sema::nominals) fn shape_size(
        &mut self,
        facts: &mut Declarations,
        shapes: &mut Shapes,
        request: Request,
        count: Option<u32>,
        normalized: bool,
    ) -> Result<bool, &'static str> {
        let mut ty = request.ty;
        if let Some(fold) = &mut self.fold {
            let before = fold.steps;
            ty = fold.apply_inside(&mut facts.types, ty, request.depth)?;
            self.context
                .account_work(fold.steps.saturating_sub(before))
                .map_err(|_| "size normalization input budget exceeded")?;
        }
        let sizes = self.shapes.get_or_insert_with(Default::default);
        let mapping = sizes.mappings.entry((count, normalized)).or_default();
        let scope = mapping.demand(
            &mut self.obligations,
            shapes,
            normalized,
            request.scope,
            count,
        )?;
        if self.obligations.scope_depth(scope)? != request.depth {
            return Err("nominal size request changed its binder depth");
        }
        if let std::collections::hash_map::Entry::Vacant(entry) = sizes.queries.entry(scope) {
            let owner = sizes::Owner {
                environment: self.environment.clone(),
                placeholders: self.placeholders.clone(),
                availability: self.availability,
            };
            entry.insert(Scope::new(
                facts,
                owner,
                self.nesting,
                (&self.obligations, scope),
                request.depth,
                &mut self.context,
            )?);
        }
        let query = sizes
            .queries
            .get_mut(&scope)
            .ok_or("missing size proof scope")?;
        let before = query.sizes.steps;
        #[cfg(test)]
        let premised = self.placeholders.is_some();
        let owner = query.owner.clone();
        let result = query.sizes.demand_with_sizes(facts, ty, |facts, ty| {
            #[cfg(test)]
            super::count_size_normalization(premised);
            let physical = Scope::physical(
                &owner,
                facts,
                shapes,
                Request { ty, ..request },
                normalized,
                &mut self.context,
            )?;
            query.cursor.start_sizing(facts, ty, physical)?;
            match query
                .cursor
                .advance(facts, &mut self.obligations, &mut |_, _| Ok(None))?
            {
                Progress::Complete { ty, steps, nested } => {
                    self.context
                        .account_nested(nested as usize)
                        .map_err(|_| "size normalization work budget exceeded")?;
                    self.context
                        .account_work(steps)
                        .map_err(|_| "size normalization work budget exceeded")?;
                    Ok(sizes::Normalized::Sizes {
                        ty,
                        learned: query.cursor.take_sizes(),
                        availability: self.availability,
                    })
                }
                Progress::Definition(_) | Progress::Dependency(_) => {
                    Err("final nominal size proof requires completed declarations")
                }
            }
        });
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                // A placeholder proof keeps its shapes when a size proof
                // fails, so its next size request in this scope starts afresh.
                if self.placeholders.is_some() {
                    sizes.queries.remove(&scope);
                }
                return Err(error);
            }
        };
        self.context
            .account_work(query.sizes.steps.saturating_sub(before))
            .map_err(|_| "nominal size proof budget exceeded")?;
        result.final_sized()
    }
}

impl State {
    #[cfg(test)]
    pub(in crate::sema::nominals) fn preload_nested(
        &mut self,
        work: usize,
    ) -> Result<(), &'static str> {
        self.context
            .account_nested(work)
            .map_err(|_| "size normalization work budget exceeded")
    }
    pub(in crate::sema::nominals) fn record_physical_size(
        &mut self,
        facts: &Declarations,
        ty: crate::types::TypeId,
        sized: bool,
    ) -> Result<(), &'static str> {
        if self.provisional_size
            || facts.types.contains_projected_sizes(ty) != Some(true)
            || facts.types.has_free_regions(ty, 0) != Some(false)
        {
            return Ok(());
        }
        let owner = sizes::Owner {
            environment: self.environment.clone(),
            placeholders: self.placeholders.clone(),
            availability: self.availability,
        };
        transfer_work(
            &mut self.context,
            1,
            "nominal size result work budget exceeded",
        )?;
        self.shapes
            .get_or_insert_with(Default::default)
            .learned
            .get_or_insert_with(|| Box::new(sizes::Learned::new(owner)))
            .insert(ty, sized)
    }
    pub(in crate::sema::nominals) fn take_sizes(&mut self) -> Option<Box<sizes::Learned>> {
        let learned = self.shapes.as_mut().and_then(|sizes| sizes.learned.take());
        if self.provisional_size { None } else { learned }
    }
    pub(super) fn retain_sizes(
        &mut self,
        learned: Option<Box<sizes::Learned>>,
    ) -> Result<(), &'static str> {
        let Some(learned) = learned else {
            return Ok(());
        };
        let owner = sizes::Owner {
            environment: self.environment.clone(),
            placeholders: self.placeholders.clone(),
            availability: self.availability,
        };
        if !learned.belongs_to(&owner) {
            return Ok(());
        }
        let stored = &mut self.shapes.get_or_insert_with(Default::default).learned;
        if let Some(stored) = stored {
            transfer_work(
                &mut self.context,
                learned.len(),
                "nominal size result work budget exceeded",
            )?;
            stored.extend(*learned)?;
        } else {
            *stored = Some(learned);
        }
        Ok(())
    }
}
