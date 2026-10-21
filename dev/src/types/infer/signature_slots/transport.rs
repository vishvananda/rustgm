//! Transfer origin and monotone class loss without granting imported frame authority.
use super::{Context, Error, Owner, Slots, TermId};

impl Context {
    pub(in crate::types::infer) fn import_slot_facts(
        &mut self,
        source: &mut Self,
        original: TermId,
        copied: TermId,
        fresh: bool,
    ) -> Result<(), Error> {
        if source.signature_slots.is_none() {
            return Ok(());
        }
        self.step()?;
        let class = if self
            .terms
            .get(copied.0 as usize)
            .ok_or(Error::InvalidIdentity)?
            .was_variable
        {
            let class = source.slot_class(original)?;
            Some(if fresh {
                class
            } else {
                self.slot_class(copied)?.merge(class)
            })
        } else {
            None
        };
        let owner_frame = source.signature_frame(original);
        let owner_definition = source
            .signature_slots
            .as_ref()
            .and_then(|facts| facts.definitions.get(&original))
            .copied();
        if class.is_none() && owner_frame.is_none() && owner_definition.is_none() {
            return Ok(());
        }
        let facts = self
            .signature_slots
            .get_or_insert_with(|| Box::new(Slots::new()));
        let steps = if let Some(class) = class {
            if fresh {
                facts.classes.fresh(copied, class)?;
                1
            } else {
                facts.classes.merge_value(copied, class)?
            }
        } else {
            0
        };
        for (owner, after) in [
            (owner_frame, &mut facts.frames),
            (owner_definition, &mut facts.definitions),
        ] {
            if let Some(owner) = owner {
                // Existing original-context authority is neither granted nor replaced by import.
                after.entry(copied).or_insert(Owner {
                    origin: owner.origin,
                    closed: false,
                });
            }
        }
        facts.bounded()?;
        self.account_work(steps)
    }

    /// The actual binder-preserving fold supplies this edge, never an arity test.
    pub(in crate::types::infer) fn folded_slot_facts(
        &mut self,
        original: TermId,
        copied: TermId,
    ) -> Result<(), Error> {
        if self.signature_slots.is_none() || original == copied {
            return Ok(());
        }
        self.step()?;
        let frame = self.signature_frame(original);
        let facts = self
            .signature_slots
            .as_mut()
            .ok_or(Error::InvalidIdentity)?;
        if let Some(frame) = frame {
            let existing = facts.frames.entry(copied).or_insert(frame);
            if existing.origin != frame.origin || !frame.closed {
                existing.closed = false;
            }
        }
        facts.bounded()
    }

    /// A query can only revoke the source class. It cannot publish local authority.
    pub(in crate::types::infer) fn receive_slot_loss(
        &mut self,
        original: TermId,
        query: &mut Self,
        copied: TermId,
    ) -> Result<(), Error> {
        if self.signature_slots.is_none() || query.signature_slots.is_none() {
            return Ok(());
        }
        self.step()?;
        let class = query.slot_class(copied)?;
        let facts = self
            .signature_slots
            .as_mut()
            .ok_or(Error::InvalidIdentity)?;
        let steps = facts.classes.merge_value(original, class)?;
        facts.bounded()?;
        self.account_work(steps)
    }
}
