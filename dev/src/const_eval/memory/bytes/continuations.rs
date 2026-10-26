//! Typed byte reads retain unpaid reference duties across the ordinary prepare retry.
use super::{Constant, Failure, Id, Machine, Read, TypeId};
use crate::const_eval::{
    ReadOperation,
    static_reads::{EvaluationContext, ReadPurpose},
};
use crate::support::hash::HashMap;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Source {
    Backing(Id),
    Image(Constant),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Key {
    pub source: Source,
    pub offset: u64,
    pub ty: TypeId,
    pub length: Option<u64>,
    pub purpose: ReadPurpose,
    pub context: EvaluationContext,
}
#[derive(Clone, Copy)]
pub(super) struct Duty {
    pub address: Constant,
    pub ty: TypeId,
    pub mutable: bool,
}
struct Pending {
    key: Key,
    value: Constant,
    duties: Vec<Duty>,
    next: usize,
}
impl Pending {
    fn retained_slots(&self) -> usize {
        self.duties
            .capacity()
            .saturating_mul(size_of::<Duty>())
            .saturating_add(4 * size_of::<Self>())
            .div_ceil(size_of::<Constant>())
    }
}
#[derive(Default)]
pub(super) struct Reads {
    entries: HashMap<ReadOperation, Pending>,
    retained: usize,
}
impl Reads {
    pub(super) fn retained_slots(&self) -> usize {
        self.retained.saturating_add(
            self.entries
                .capacity()
                .saturating_mul(4 * size_of::<(ReadOperation, Pending)>())
                .saturating_add(4 * size_of::<Self>())
                .div_ceil(size_of::<Constant>()),
        )
    }
}
impl super::Storage {
    pub(in crate::const_eval::memory) fn abandon_reads(&mut self) {
        if let Some(reads) = self.reads.as_mut() {
            for (_, state) in reads.entries.drain() {
                if let Source::Backing(id) = state.key.source
                    && let Some(root) = self.roots.get_mut(id.0 as usize)
                {
                    root.pending_reads = root.pending_reads.saturating_sub(1);
                }
            }
            reads.retained = 0;
        }
    }
}
impl Machine<'_, '_> {
    pub(super) fn resumed_byte_read(
        &mut self,
        key: Key,
    ) -> Result<Option<Read<Constant>>, Failure> {
        let Some(operation) = self.read_operation else {
            return Ok(None);
        };
        let Some(reads) = self.memory.bytes.reads.as_mut() else {
            return Ok(None);
        };
        let Some(state) = reads.entries.remove(&operation) else {
            return Ok(None);
        };
        reads.retained = reads
            .retained
            .checked_sub(state.retained_slots())
            .ok_or("pending byte read retention underflow")?;
        if state.key != key {
            self.unpin_byte_read(state.key.source)?;
            return Err("pending typed byte read resumed at a different source or context".into());
        }
        self.run_byte_duties(operation, state, true).map(Some)
    }
    pub(super) fn complete_byte_read(
        &mut self,
        key: Key,
        value: Constant,
        duties: Vec<Duty>,
    ) -> Result<Read<Constant>, Failure> {
        if duties.is_empty() {
            return Ok(Read::Value(value));
        }
        let operation = self
            .read_operation
            .ok_or("reference byte read lacks its exact execution operation")?;
        self.run_byte_duties(
            operation,
            Pending {
                key,
                value,
                duties,
                next: 0,
            },
            false,
        )
    }
    fn run_byte_duties(
        &mut self,
        operation: ReadOperation,
        mut state: Pending,
        pinned: bool,
    ) -> Result<Read<Constant>, Failure> {
        let retained = state.retained_slots();
        let previous = self.slots;
        if let Err(error) = self.reserve_cells(retained) {
            if pinned {
                self.unpin_byte_read(state.key.source)?;
            }
            return Err(error);
        }
        self.slots = previous
            .checked_add(retained)
            .ok_or("typed byte duty retention overflow")?;
        let checked = (|| {
            while let Some(duty) = state.duties.get(state.next).copied() {
                #[cfg(test)]
                {
                    self.memory.bytes.checked_duties += 1;
                }
                self.charge(1)?;
                let Read::Value(value) =
                    self.validate_word_reference(duty.address, duty.ty, duty.mutable)?
                else {
                    if self.waiting.is_none() {
                        return Err(
                            "pending reference validity has no preparation requirement".into()
                        );
                    }
                    return Ok(false);
                };
                if value != duty.address {
                    return Err("reference word validation changed its checked provenance".into());
                }
                state.next += 1;
                #[cfg(test)]
                {
                    self.memory.bytes.paid_duties += 1;
                }
            }
            Ok(true)
        })();
        self.slots = previous;
        match checked {
            Ok(false) => {
                let source = state.key.source;
                let result = self.park_byte_read(operation, state, pinned);
                if result.is_err() && pinned {
                    self.unpin_byte_read(source)?;
                }
                result.map(|()| Read::Pending)
            }
            result => {
                if pinned {
                    self.unpin_byte_read(state.key.source)?;
                }
                result.map(|_| Read::Value(state.value))
            }
        }
    }
    fn park_byte_read(
        &mut self,
        operation: ReadOperation,
        state: Pending,
        pinned: bool,
    ) -> Result<(), Failure> {
        let length = self
            .memory
            .bytes
            .reads
            .as_ref()
            .map_or(0, |reads| reads.entries.len());
        self.reserve_cells(
            state.retained_slots().saturating_add(
                length
                    .saturating_add(4)
                    .saturating_mul(4 * size_of::<(ReadOperation, Pending)>())
                    .div_ceil(size_of::<Constant>()),
            ),
        )?;
        let reads = self
            .memory
            .bytes
            .reads
            .get_or_insert_with(|| Box::new(Reads::default()));
        reads
            .entries
            .try_reserve(1)
            .map_err(|_| "typed byte continuation allocation failed")?;
        if reads.entries.contains_key(&operation) {
            return Err("typed byte operation already has a parked continuation".into());
        }
        let retained = reads
            .retained
            .checked_add(state.retained_slots())
            .ok_or("typed byte continuation retention overflow")?;
        if !pinned && let Source::Backing(id) = state.key.source {
            let backing = self
                .memory
                .bytes
                .roots
                .get_mut(id.0 as usize)
                .ok_or("pending byte source disappeared")?;
            backing.pending_reads = backing
                .pending_reads
                .checked_add(1)
                .ok_or("pending byte reader count overflow")?;
        }
        reads.retained = retained;
        reads.entries.insert(operation, state);
        Ok(())
    }
    fn unpin_byte_read(&mut self, source: Source) -> Result<(), Failure> {
        if let Source::Backing(id) = source {
            let backing = self
                .memory
                .bytes
                .roots
                .get_mut(id.0 as usize)
                .ok_or("pending byte source disappeared")?;
            backing.pending_reads = backing
                .pending_reads
                .checked_sub(1)
                .ok_or("pending byte reader count underflow")?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/transmute_layouts.rs"]
mod transmute_layout_tests;
