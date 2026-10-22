//! Compiler-local handles retain caller/result identity, never a proof graph or permit.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::{inline_constants::InlineInstance, values::Instance},
    types::TypeId,
};
use std::{
    collections::HashMap,
    num::NonZeroU32,
    sync::atomic::{AtomicU32, Ordering},
};

// This is a compiler-owned lifetime discriminator, not an environment or process
// identity. Never format/serialize it or use it for canonical ordering.
static NEXT_EPOCH: AtomicU32 = AtomicU32::new(1);
const LIMIT: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Handle {
    epoch: NonZeroU32,
    row: NonZeroU32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct CallerId(NonZeroU32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Enclosing {
    Original,
    Callable(crate::types::instance::Instance),
    Initializer(Instance),
    Inline(InlineInstance),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct CallerKey {
    pub source: NodeKey,
    pub declaration: Option<DefId>,
    pub universals: u32,
    pub enclosing: Enclosing,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Publication([NonZeroU32; 4]);
impl Publication {
    pub(super) fn new(ranges: [std::ops::Range<usize>; 2]) -> Result<Self, &'static str> {
        if ranges[0].end < ranges[0].start
            || ranges[0].end > ranges[1].start
            || ranges[1].end < ranges[1].start
        {
            return Err("source caller publication ranges are invalid");
        }
        let [first, late] = ranges;
        let mut result = [NonZeroU32::MIN; 4];
        for (slot, value) in result
            .iter_mut()
            .zip([first.start, first.end, late.start, late.end])
        {
            *slot = value
                .checked_add(1)
                .and_then(|n| u32::try_from(n).ok())
                .and_then(NonZeroU32::new)
                .ok_or("source caller publication index space exhausted")?;
        }
        Ok(Self(result))
    }
    pub(super) fn within(
        self,
        facts: &crate::sema::model::Declarations,
    ) -> Result<(), &'static str> {
        if self.0[3].get() as usize - 1 > facts.region_equalities.len() {
            return Err("source caller publication is stale");
        }
        Ok(())
    }
    pub(super) fn validate(
        self,
        facts: &crate::sema::model::Declarations,
        body: NodeKey,
    ) -> Result<(), &'static str> {
        for range in [
            self.0[0].get() as usize - 1..self.0[1].get() as usize - 1,
            self.0[2].get() as usize - 1..self.0[3].get() as usize - 1,
        ] {
            let slice = facts
                .region_equalities
                .get(range)
                .ok_or("source caller publication is stale")?;
            if slice.iter().any(|fact| fact.body != body) {
                return Err("source caller publication has a foreign owner");
            }
        }
        Ok(())
    }
}
struct Caller {
    key: CallerKey,
    publication: Publication,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Use {
    pub caller: CallerId,
    pub input: Instance,
    pub selected: Instance,
    pub original: NodeKey,
    pub actual_result: TypeId,
    pub requested: TypeId,
    pub seal: Option<super::seals::Seal>,
}
pub(super) struct Catalog {
    epoch: NonZeroU32,
    callers: Vec<Caller>,
    caller_indices: HashMap<CallerKey, CallerId>,
    originals: HashMap<NodeKey, CallerId>,
    uses: Vec<Use>,
    use_indices: HashMap<Use, NonZeroU32>,
    bytes: usize,
    pub(super) namespaces: Option<Box<super::namespaces::Namespaces>>,
}
impl Catalog {
    pub(super) fn new() -> Result<Self, &'static str> {
        let epoch = NEXT_EPOCH
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or("source caller catalog epoch space exhausted")?;
        Ok(Self {
            epoch,
            callers: Vec::new(),
            caller_indices: HashMap::new(),
            originals: HashMap::new(),
            uses: Vec::new(),
            use_indices: HashMap::new(),
            bytes: size_of::<Self>(),
            namespaces: None,
        })
    }
    pub(super) fn caller(
        &mut self,
        key: CallerKey,
        publication: Publication,
    ) -> Result<CallerId, &'static str> {
        if let Some(&id) = self.caller_indices.get(&key) {
            let previous = self
                .callers
                .get(id.0.get() as usize - 1)
                .ok_or("source caller catalog index is stale")?;
            if previous.key != key || previous.publication != publication {
                return Err("source caller publication changed after registration");
            }
            return Ok(id);
        }
        let source_bytes = if matches!(key.enclosing, Enclosing::Original) {
            crate::support::retained::new_map::<NodeKey, CallerId>(1)
        } else {
            0
        };
        let bytes = self
            .bytes
            .saturating_add(source_bytes)
            .saturating_add(2 * size_of::<Caller>())
            .saturating_add(crate::support::retained::new_map::<CallerKey, CallerId>(1));
        self.preflight(bytes, self.callers.len())?;
        if matches!(key.enclosing, Enclosing::Original) && self.originals.contains_key(&key.source)
        {
            return Err("source body has a different completed caller context");
        }
        let id = CallerId(index(self.callers.len())?);
        if matches!(key.enclosing, Enclosing::Original) {
            self.originals.insert(key.source, id);
        }
        self.callers.push(Caller { key, publication });
        self.caller_indices.insert(key, id);
        self.bytes = bytes;
        Ok(id)
    }
    /// Input validation and debt retention belong to the enclosing checked minting path.
    pub(super) fn retain_use(&mut self, value: Use) -> Result<Handle, &'static str> {
        self.callers
            .get(value.caller.0.get() as usize - 1)
            .ok_or("source computation caller is stale")?;
        if let Some(&row) = self.use_indices.get(&value) {
            if self.uses.get(row.get() as usize - 1) != Some(&value) {
                return Err("source computation use index is stale");
            }
            return Ok(Handle {
                epoch: self.epoch,
                row,
            });
        }
        let bytes = self
            .bytes
            .saturating_add(2 * size_of::<Use>())
            .saturating_add(crate::support::retained::new_map::<Use, NonZeroU32>(1));
        self.preflight(bytes, self.uses.len())?;
        let row = index(self.uses.len())?;
        self.uses.push(value);
        self.use_indices.insert(value, row);
        self.bytes = bytes;
        Ok(Handle {
            epoch: self.epoch,
            row,
        })
    }
    pub(super) fn lookup(&self, handle: Handle) -> Result<(Use, CallerKey), &'static str> {
        if handle.epoch != self.epoch {
            return Err("source computation belongs to another declaration catalog");
        }
        let value = *self
            .uses
            .get(handle.row.get() as usize - 1)
            .ok_or("source computation use index is stale")?;
        let caller = self
            .callers
            .get(value.caller.0.get() as usize - 1)
            .ok_or("source computation caller index is stale")?;
        Ok((value, caller.key))
    }
    pub(super) fn original(
        &self,
        source: NodeKey,
    ) -> Result<(CallerKey, Publication), &'static str> {
        let id = self
            .originals
            .get(&source)
            .ok_or("source caller has no completed original publication")?;
        let caller = self
            .callers
            .get(id.0.get() as usize - 1)
            .ok_or("source caller original index is stale")?;
        if caller.key.source != source || !matches!(caller.key.enclosing, Enclosing::Original) {
            return Err("source caller original ownership changed");
        }
        Ok((caller.key, caller.publication))
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.bytes
            .saturating_add(self.namespaces.as_ref().map_or(0, |n| n.bytes))
    }
    pub(super) fn namespace_available(&self) -> usize {
        LIMIT.saturating_sub(self.retained_bytes())
    }
    pub(super) fn namespace_budget(&self, additional: usize) -> Result<(), &'static str> {
        if self.bytes.saturating_add(additional) > LIMIT {
            return Err("source caller namespace exceeds its retained storage budget");
        }
        Ok(())
    }
    pub(super) fn publication(&self, id: CallerId) -> Result<Publication, &'static str> {
        self.callers
            .get(id.0.get() as usize - 1)
            .map(|caller| caller.publication)
            .ok_or("source caller index is stale")
    }
    fn preflight(&self, bytes: usize, entries: usize) -> Result<(), &'static str> {
        if bytes.saturating_add(self.namespaces.as_ref().map_or(0, |n| n.bytes)) > LIMIT
            || entries >= 1_000_000
        {
            return Err("source caller catalog exceeds its retained storage budget");
        }
        Ok(())
    }
}
fn index(length: usize) -> Result<NonZeroU32, &'static str> {
    length
        .checked_add(1)
        .and_then(|n| u32::try_from(n).ok())
        .and_then(NonZeroU32::new)
        .ok_or("source caller catalog index space exhausted")
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SourceComputation(..)")
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/catalog_owners.rs"]
mod tests;
