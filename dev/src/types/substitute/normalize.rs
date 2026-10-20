//! Suspend the common type fold at projections so semantic selection can borrow the store.
mod aliases;
mod scopes;
use super::{Action, Bound, Key};
use crate::types::{Kind, Store, TypeId, lifetime_proof::ScopeId};
pub use aliases::Alias;
use scopes::{Frame, Suspended};
use std::collections::{HashMap, HashSet};

/// Inputs have already been normalized in the indicated surrounding binder depth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub alias: Alias,
    pub depth: u16,
    /// Identity in this normalizer's callable binder tree; siblings are distinct.
    pub scope: ScopeId,
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
    frame: Frame,
    suspended: Vec<Suspended>,
    binders: Vec<(ScopeId, u32)>,
    suspended_work: usize,
    root: Option<Key>,
    pending: Option<Request>,
    failed: bool,
}
impl Default for Normalizer {
    fn default() -> Self {
        Self {
            frame: Frame::new(ScopeId::ROOT, 0),
            suspended: Vec::new(),
            binders: Vec::new(),
            suspended_work: 0,
            root: None,
            pending: None,
            failed: false,
        }
    }
}
impl Normalizer {
    #[must_use]
    pub fn steps(&self) -> usize {
        self.frame.fold.steps
    }

    /// Begin a root in the same semantic scope, reusing completed graph work.
    /// # Errors
    /// Rejects an unfinished/failed query, malformed types, cycles or exhausted work.
    pub fn start(&mut self, store: &mut Store, ty: TypeId) -> Result<Step, &'static str> {
        if self.root.is_some() || self.failed {
            return Err("normalization scope has an unfinished or failed query");
        }
        self.root = Some((ty, 0));
        self.frame.fold.queue(ty, 0)?;
        self.advance(store)
    }

    /// Begin a root whose child is already a normal form in this exact scope.
    /// The caller retains the child's formation evidence, including erased debts.
    /// Only this root-scope child is reused; new arguments, replacement types and
    /// nested callable scopes still demand their own normalization.
    /// # Errors
    /// Rejects an unfinished query, invalid identities or a conflicting normal form.
    pub fn start_with_input(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        input: TypeId,
    ) -> Result<Step, &'static str> {
        if self.root.is_some() || self.failed {
            return Err("normalization scope has an unfinished or failed query");
        }
        if ty == input || store.kind(input).is_none() || store.kind(ty).is_none() {
            return Err("invalid completed normalization input");
        }
        if self
            .frame
            .fold
            .completed
            .get(&(input, 0))
            .is_some_and(|&value| value != input)
        {
            return Err("completed normalization input conflicts with its previous result");
        }
        self.frame.fold.account_arguments(1)?;
        self.frame.fold.completed.insert((input, 0), input);
        self.start(store, ty)
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
        let key = (request.source, 0);
        match resolution {
            Resolution::Opaque | Resolution::Deferred => {
                let ty = store.intern(request.alias.kind())?;
                self.frame.fold.completed.insert(key, ty);
                self.frame.active.remove(&key);
            }
            Resolution::Type(ty) => {
                store
                    .kind(ty)
                    .ok_or("invalid normalized projection result")?;
                self.frame.fold.account_arguments(1)?;
                self.frame.redirects.insert(key, (ty, 0));
                self.frame.fold.work.push((key, true));
                self.frame.fold.queue(ty, 0)?;
            }
        }
        self.pending = None;
        self.advance(store)
    }

    fn advance(&mut self, store: &mut Store) -> Result<Step, &'static str> {
        let result = self.drain(store);
        if result.is_err() {
            self.failed = true;
            self.frame.fold.work.clear();
            self.suspended.clear();
            self.suspended_work = 0;
            self.pending = None;
        }
        result
    }

    fn drain(&mut self, store: &mut Store) -> Result<Step, &'static str> {
        loop {
            self.check_work()?;
            let Some((key, finish)) = self.frame.fold.work.pop() else {
                if self.finish_binder(store)? {
                    continue;
                }
                break;
            };
            self.frame.fold.account_arguments(1)?;
            if self.frame.fold.result(store, key)?.is_some() {
                continue;
            }
            if finish {
                if let Some(&(ty, depth)) = self.frame.redirects.get(&key) {
                    let ty = self.frame.fold.child(store, ty, depth)?;
                    self.complete(key, ty);
                    continue;
                }
                let kind = store.kind(key.0).ok_or("invalid normalization type")?;
                let kind = self.frame.fold.rebuild(store, kind, key.1)?;
                let alias = match kind {
                    Kind::Projection(application) => Some(Alias::Associated(application)),
                    Kind::Opaque(id, arguments)
                        if store.opaque_declaration(id).is_some_and(|d| d.trait_result) =>
                    {
                        Some(Alias::TraitResult(id, arguments))
                    }
                    _ => None,
                };
                if let Some(alias) = alias {
                    let request = Request {
                        alias,
                        depth: u16::try_from(self.suspended.len())
                            .map_err(|_| "normalization binder depth exhausted")?,
                        scope: self.frame.scope,
                        source: key.0,
                    };
                    self.pending = Some(request);
                    return Ok(Step::Resolve(request));
                }
                self.complete(key, store.intern(kind)?);
            } else {
                if !self.frame.active.insert(key) {
                    return Err("cycle in associated type normalization");
                }
                let kind = store
                    .kind(key.0)
                    .ok_or("invalid normalization dependency")?;
                if matches!(kind, Kind::Function { .. } | Kind::Dynamic { .. }) {
                    self.enter_binder(store, key, kind)?;
                } else {
                    self.frame.fold.work.push((key, true));
                    self.frame.fold.dependencies(store, kind, key.1)?;
                }
            }
        }
        let root = self.root.ok_or("missing normalization root")?;
        let ty = self.frame.fold.child(store, root.0, root.1)?;
        self.root = None;
        Ok(Step::Complete(ty))
    }

    fn complete(&mut self, key: Key, ty: TypeId) {
        self.frame.fold.completed.insert(key, ty);
        self.frame.redirects.remove(&key);
        self.frame.active.remove(&key);
    }
}
