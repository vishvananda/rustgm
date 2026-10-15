//! Suspend the common type fold at projections so semantic selection can borrow the store.
use super::{Action, Bound, Key};
use crate::types::{Kind, Store, TypeId, projection::Application};
use std::collections::{HashMap, HashSet};

/// Inputs have already been normalized in the indicated surrounding binder depth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub application: Application,
    pub depth: u16,
    source: TypeId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Complete(TypeId),
    Resolve(Request),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    /// The resolver has proved this result in the request's exact binder context.
    Type(TypeId),
    /// The resolver has proved the trait obligation but its result remains symbolic.
    Opaque,
    /// An alias contract retains this unproved type-formation requirement for its uses.
    /// The resolver must store the requirement before returning this response.
    Deferred,
}

/// One immutable semantic environment owns this cache. Keep availability, caller
/// assumptions and binder universes fixed; a different proof scope needs a fresh fold.
pub struct Normalizer {
    fold: Bound,
    active: HashSet<Key>,
    redirects: HashMap<Key, Key>,
    root: Option<Key>,
    pending: Option<Request>,
    failed: bool,
}
impl Default for Normalizer {
    fn default() -> Self {
        let mut fold = Bound::new(Vec::new());
        fold.action = Action::Normalize;
        Self {
            fold,
            active: HashSet::new(),
            redirects: HashMap::new(),
            root: None,
            pending: None,
            failed: false,
        }
    }
}
impl Normalizer {
    #[must_use]
    pub fn steps(&self) -> usize {
        self.fold.steps
    }

    /// Begin a root in the same semantic scope, reusing completed graph work.
    /// # Errors
    /// Rejects an unfinished/failed query, malformed types, cycles or exhausted work.
    pub fn start(&mut self, store: &mut Store, ty: TypeId) -> Result<Step, &'static str> {
        if self.root.is_some() || self.failed {
            return Err("normalization scope has an unfinished or failed query");
        }
        self.root = Some((ty, 0));
        self.fold.queue(ty, 0)?;
        self.advance(store)
    }

    /// Continue after semantic selection; replacement types use the request's context.
    /// The replacement itself is normalized before publishing its parent result.
    /// # Errors
    /// Rejects stale requests, malformed types, recursive normalization or excess work.
    pub fn resume(
        &mut self,
        store: &mut Store,
        request: Request,
        resolution: Resolution,
    ) -> Result<Step, &'static str> {
        if self.pending != Some(request) || self.failed {
            return Err("normalization response does not match its pending request");
        }
        let key = (request.source, request.depth);
        match resolution {
            Resolution::Opaque | Resolution::Deferred => {
                let ty = store.intern(Kind::Projection(request.application))?;
                self.fold.completed.insert(key, ty);
                self.active.remove(&key);
            }
            Resolution::Type(ty) => {
                store
                    .kind(ty)
                    .ok_or("invalid normalized projection result")?;
                self.fold.account_arguments(1)?;
                self.redirects.insert(key, (ty, request.depth));
                self.fold.work.push((key, true));
                self.fold.queue(ty, request.depth)?;
            }
        }
        self.pending = None;
        self.advance(store)
    }

    fn advance(&mut self, store: &mut Store) -> Result<Step, &'static str> {
        let result = self.drain(store);
        if result.is_err() {
            self.failed = true;
            self.fold.work.clear();
            self.pending = None;
        }
        result
    }

    fn drain(&mut self, store: &mut Store) -> Result<Step, &'static str> {
        while let Some((key, finish)) = self.fold.work.pop() {
            self.fold.account_arguments(1)?;
            if self.fold.result(store, key)?.is_some() {
                continue;
            }
            if finish {
                if let Some(&(ty, depth)) = self.redirects.get(&key) {
                    let ty = self.fold.child(store, ty, depth)?;
                    self.complete(key, ty);
                    continue;
                }
                let kind = store.kind(key.0).ok_or("invalid normalization type")?;
                let kind = self.fold.rebuild(store, kind, key.1)?;
                if let Kind::Projection(application) = kind {
                    let request = Request {
                        application,
                        depth: key.1,
                        source: key.0,
                    };
                    self.pending = Some(request);
                    return Ok(Step::Resolve(request));
                }
                self.complete(key, store.intern(kind)?);
            } else {
                if !self.active.insert(key) {
                    return Err("cycle in associated type normalization");
                }
                let kind = store
                    .kind(key.0)
                    .ok_or("invalid normalization dependency")?;
                self.fold.work.push((key, true));
                self.fold.dependencies(store, kind, key.1)?;
            }
        }
        let root = self.root.ok_or("missing normalization root")?;
        let ty = self.fold.child(store, root.0, root.1)?;
        self.root = None;
        Ok(Step::Complete(ty))
    }

    fn complete(&mut self, key: Key, ty: TypeId) {
        self.fold.completed.insert(key, ty);
        self.redirects.remove(&key);
        self.active.remove(&key);
    }
}
