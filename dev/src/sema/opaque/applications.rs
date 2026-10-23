//! Deferred full applications wake only when their exact defining identity closes.
use crate::{
    support::{hash::HashMap, retained},
    types::{TypeId, opaque::OpaqueId},
};

const APPLICATIONS: usize = 100_000;
const BYTES: usize = 64 * 1024 * 1024;

#[cfg(test)]
#[path = "../../../tests/invariants/waiting_heads.rs"]
mod tests;

#[derive(Default)]
pub(super) struct Preparation {
    pub layouts: crate::abi::templates::Templates,
    waiting: HashMap<OpaqueId, Vec<TypeId>>,
    pub ready: Vec<TypeId>,
    waiting_entries: usize,
    peak_keys: usize,
    peak_waiting: usize,
    peak_ready: usize,
    pub steps: usize,
    pub failed: Option<&'static str>,
}

impl Preparation {
    pub fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|&steps| steps <= 32_000_000)
            .ok_or("opaque storage preparation work budget exceeded")?;
        Ok(())
    }
    fn reserve(&mut self, keys: usize, waiting: usize, ready: usize) -> Result<(), &'static str> {
        if waiting > APPLICATIONS || ready > APPLICATIONS {
            return Err("opaque storage application budget exceeded");
        }
        let keys = keys.max(self.peak_keys);
        let waiting = waiting.max(self.peak_waiting);
        let ready = ready.max(self.peak_ready);
        let bytes = Self::queue_bytes(keys, waiting, ready);
        if bytes > BYTES {
            return Err("opaque storage preparation exceeds storage budget");
        }
        self.peak_keys = keys;
        self.peak_waiting = waiting;
        self.peak_ready = ready;
        Ok(())
    }
    fn queue_bytes(keys: usize, waiting: usize, ready: usize) -> usize {
        size_of::<Self>()
            .saturating_add(retained::new_map::<OpaqueId, Vec<TypeId>>(keys))
            .saturating_add(
                waiting
                    .saturating_add(ready)
                    .saturating_mul(4 * size_of::<TypeId>()),
            )
    }
    pub fn wait(&mut self, id: OpaqueId, ty: TypeId) -> Result<(), &'static str> {
        self.charge(1)?;
        let keys = self.waiting.len() + usize::from(!self.waiting.contains_key(&id));
        self.reserve(
            keys,
            self.waiting_entries.saturating_add(1),
            self.ready.len(),
        )?;
        self.waiting.entry(id).or_default().push(ty);
        self.waiting_entries += 1;
        Ok(())
    }
    pub fn wake(&mut self, id: OpaqueId) -> Result<(), &'static str> {
        if let Some(error) = self.failed {
            return Err(error);
        }
        let result = self.wake_ready(id);
        self.failed = result.as_ref().err().copied();
        result
    }
    fn wake_ready(&mut self, id: OpaqueId) -> Result<(), &'static str> {
        let Some(rows) = self.waiting.get(&id) else {
            return Ok(());
        };
        let count = rows.len();
        self.charge(count.saturating_add(1))?;
        self.reserve(
            self.waiting.len(),
            self.waiting_entries,
            self.ready.len().saturating_add(count),
        )?;
        let rows = self
            .waiting
            .remove(&id)
            .ok_or("lost deferred opaque applications")?;
        self.waiting_entries -= count;
        self.ready.extend(rows);
        Ok(())
    }
    /// Roots, pending heads, cycle paths and the detached replay vector coexist.
    pub fn scratch(&self, count: usize, replay_capacity: usize) -> Result<(), &'static str> {
        if count > APPLICATIONS {
            return Err("opaque storage application budget exceeded");
        }
        let bytes = count
            .saturating_mul(16 * size_of::<TypeId>())
            .saturating_add(retained::new_map::<TypeId, TypeId>(count))
            .saturating_add(retained::new_map::<TypeId, ()>(count))
            .saturating_add(replay_capacity.saturating_mul(size_of::<TypeId>()))
            .saturating_add(Self::queue_bytes(
                self.peak_keys,
                self.peak_waiting,
                self.peak_ready,
            ));
        if bytes > BYTES {
            return Err("opaque storage preparation scratch exceeds storage budget");
        }
        Ok(())
    }
}
