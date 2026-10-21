//! Root-keyed metadata follows ordered links without rescanning its old entries.
use super::{Context, Error, TermId};

impl Context {
    /// Enable one caller-owned metadata index. Existing roots must be obtained
    /// with `root`; the journal starts here and records no earlier history.
    pub(crate) fn track_root_changes(&mut self) {
        self.dependencies
            .get_or_insert_with(Default::default)
            .root_changes
            .get_or_insert_with(Vec::new);
    }

    /// Move new links into an empty reusable consumer buffer. Every link is
    /// observed once, in order. Path compression changes no equivalence class.
    pub(crate) fn exchange_root_changes(
        &mut self,
        changes: &mut Vec<(TermId, TermId)>,
    ) -> Result<(), Error> {
        if !changes.is_empty() {
            return Err(Error::InvalidIdentity);
        }
        let Some(journal) = self
            .dependencies
            .as_mut()
            .and_then(|storage| storage.root_changes.as_mut())
        else {
            return Ok(());
        };
        std::mem::swap(journal, changes);
        self.account_work(changes.len())
    }

    pub(super) fn link_root(&mut self, source: TermId, target: TermId) -> Result<(), Error> {
        self.link_slot_classes(source, target)?;
        self.terms[source.0 as usize].parent = target;
        if let Some(journal) = self
            .dependencies
            .as_mut()
            .and_then(|storage| storage.root_changes.as_mut())
        {
            journal.push((source, target));
            self.step()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Store, TypeId, infer::Domain};

    #[test]
    fn root_journal_starts_at_subscription_and_reports_each_link_once() -> Result<(), Error> {
        let store = Store::default();
        let mut context = Context::default();
        let first = context.variable(Domain::Any)?;
        let second = context.variable(Domain::Any)?;
        context.equate(&store, first, second)?;
        context.track_root_changes();
        let origin = context.root(second)?;
        let next = context.variable(Domain::Any)?;
        context.equate(&store, next, origin)?;
        let ty = context.known(&store, TypeId::UNIT)?;
        context.equate(&store, second, ty)?;
        let mut links = Vec::new();
        context.exchange_root_changes(&mut links)?;
        assert_eq!(links.len(), 2);
        let mut roots = std::collections::HashMap::from([(origin, 7), (next, 9)]);
        for (source, target) in links.drain(..) {
            if let Some(value) = roots.remove(&source) {
                let current = roots.entry(target).or_insert(value);
                *current = (*current).min(value);
            }
        }
        assert_eq!(roots, std::collections::HashMap::from([(ty, 7)]));
        assert_eq!(context.root(first)?, ty);
        assert_eq!(context.root(next)?, ty);
        context.exchange_root_changes(&mut links)?;
        assert!(links.is_empty());
        Ok(())
    }
}
