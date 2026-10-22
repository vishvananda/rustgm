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
}
#[derive(Default)]
pub(super) struct Sizes {
    mappings: HashMap<(Option<u32>, bool), Mapping>,
    queries: HashMap<ScopeId, Scope>,
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
            let environment = self
                .environment
                .as_ref()
                .map(|environment| {
                    if request.depth == 0 {
                        return Ok(Arc::clone(environment));
                    }
                    let mut shift = Bound::shift(request.depth);
                    let shifted = environment.substitute_regions(&mut facts.types, &mut shift)?;
                    self.context
                        .account_work(shift.steps)
                        .map_err(|_| "size normalization environment budget exceeded")?;
                    Ok::<_, &'static str>(Arc::new(shifted))
                })
                .transpose()?;
            entry.insert(Scope {
                sizes: sizes::Query::new(environment),
                cursor: Cursor::new(
                    self.environment.clone(),
                    &self.obligations,
                    scope,
                    self.availability,
                )?,
            });
        }
        let query = sizes
            .queries
            .get_mut(&scope)
            .ok_or("missing size proof scope")?;
        let before = query.sizes.steps;
        let result = query.sizes.demand(facts, ty, |facts, ty| {
            query.cursor.start(facts, ty)?;
            match query
                .cursor
                .advance(facts, &mut self.obligations, &mut |_, _| Ok(None))?
            {
                Progress::Complete { ty, steps } => {
                    self.context
                        .account_work(steps)
                        .map_err(|_| "size normalization work budget exceeded")?;
                    Ok(ty)
                }
                Progress::Definition(_) | Progress::Dependency(_) => {
                    Err("final nominal size proof requires completed declarations")
                }
            }
        })?;
        self.context
            .account_work(query.sizes.steps.saturating_sub(before))
            .map_err(|_| "nominal size proof budget exceeded")?;
        match result {
            sizes::Progress::Ready {
                sized,
                provisional: false,
            } => Ok(sized),
            sizes::Progress::Ready {
                provisional: true, ..
            }
            | sizes::Progress::Fields => {
                Err("final nominal size proof cannot retain provisional fields")
            }
        }
    }
}
