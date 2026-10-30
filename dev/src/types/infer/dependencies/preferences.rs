//! Selected child preferences lend a type to an existing real numeric frontier.
use super::{Context, Dependency, Error, Fallback, State, Visit, WatchId};
use crate::types::infer::{Domain, TermId};

impl Context {
    pub(crate) fn clear_float_preference(&mut self, id: WatchId) -> Result<(), Error> {
        let fallback = self
            .dependencies
            .as_ref()
            .and_then(|storage| storage.watches.get(id.0 as usize))
            .ok_or(Error::InvalidIdentity)?
            .fallback;
        if matches!(fallback, Fallback::Float32(_)) {
            self.step()?;
            self.dependencies
                .as_mut()
                .and_then(|storage| storage.watches.get_mut(id.0 as usize))
                .ok_or(Error::InvalidIdentity)?
                .fallback = Fallback::Predicate;
        }
        Ok(())
    }
    /// Attach a preference to this live predicate, never to a numeric root globally.
    /// The semantic caller authenticates the source language contract first.
    pub(crate) fn prefer_f32(&mut self, id: WatchId, argument: TermId) -> Result<(), Error> {
        self.step()?;
        if self.unresolved_domain(argument)? != Some(Domain::Float) {
            return Err(Error::InvalidIdentity);
        }
        let watch = self
            .dependencies
            .as_mut()
            .and_then(|storage| storage.watches.get_mut(id.0 as usize))
            .ok_or(Error::InvalidIdentity)?;
        if !matches!(watch.fallback, Fallback::Predicate | Fallback::Float32(_))
            || watch.state == State::Cancelled
        {
            return Err(Error::InvalidIdentity);
        }
        watch.fallback = Fallback::Float32(argument);
        Ok(())
    }

    /// Read only this live condition's authenticated, still-floating preference.
    pub(crate) fn preferred_float(&mut self, id: WatchId) -> Result<Option<TermId>, Error> {
        self.step()?;
        let watch = self
            .dependencies
            .as_ref()
            .and_then(|storage| storage.watches.get(id.0 as usize))
            .ok_or(Error::InvalidIdentity)?;
        let Fallback::Float32(term) = watch.fallback else {
            return Ok(None);
        };
        if watch.state == State::Cancelled {
            return Ok(None);
        }
        Ok((self.unresolved_domain(term)? == Some(Domain::Float)).then_some(term))
    }

    /// A retained child may prefer only an actual visible parent input. This
    /// silent ticket cannot seed fallback or issue a resolution notification.
    /// Its semantic owner must cancel it before resuming or completing the query.
    pub(crate) fn watch_float_hint(
        &mut self,
        parent: WatchId,
        argument: TermId,
    ) -> Result<Option<WatchId>, Error> {
        self.step()?;
        let argument = self.root(argument)?;
        if self.unresolved_domain(argument)? != Some(Domain::Float) {
            return Ok(None);
        }
        let owner = self
            .dependencies
            .as_ref()
            .and_then(|storage| storage.watches.get(parent.0 as usize))
            .ok_or(Error::InvalidIdentity)?;
        if owner.state == State::Cancelled
            || !owner.fallback.seeds()
            || owner.seen.get(&Dependency::Type(argument)) != Some(&Visit::Numeric)
        {
            return Ok(None);
        }
        self.watch_inputs(&[argument], &[], false, Fallback::Hint32(argument))
            .map(Some)
    }
}
