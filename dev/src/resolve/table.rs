//! Root bindings use CPPGM's flat lookup table; opaque item contexts use a cold
//! index into the same ordered entries. Local histories retain their own marks.
mod contextual;
mod histories;
use super::{
    model::{Binding, Namespace, Origin, ScopeId},
    names::NameId,
};
use crate::expand::marks::{ItemContext, LocalContext};

pub(crate) type BindingKey = (ScopeId, NameId, Namespace, ItemContext);

const NONE: u32 = u32::MAX;

/// An open-addressed slot holds the key hash's high half (its tag) beside
/// `entry + 1`; zero is vacant. Probes reject most other keys by tag without
/// touching their entries.
const fn occupied(tag: u64, entry: u32) -> u64 {
    tag | (entry as u64 + 1)
}
const fn slot_entry(slot: u64) -> Option<u32> {
    match (slot & !TAG).checked_sub(1) {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the low half stores a 32-bit entry plus one"
        )]
        Some(entry) => Some(entry as u32),
        None => None,
    }
}
const TAG: u64 = 0xffff_ffff_0000_0000;
struct Entry {
    scope: ScopeId,
    name: NameId,
    namespace: Namespace,
    /// Whether the key has entered its scope's first-binding order. A removed
    /// binding keeps its position, so a later binding reuses it.
    recorded: bool,
    binding: Option<Binding>,
    /// The next recorded entry of the same scope.
    next: u32,
    /// The newest hygienic local of this key, in any syntax context.
    local: u32,
}
/// A hygienic local; older definitions of the same key follow `next`.
#[derive(Clone, Copy)]
struct Local {
    context: LocalContext,
    binding: Binding,
    next: u32,
}
/// Each scope's recorded keys form a list through their entries. Two name
/// filters, one bit per name hash, stop most probes for absent names before
/// they reach a cold table line; ordinary lookups ignore hygienic locals.
#[derive(Clone, Copy)]
struct Order {
    first: u32,
    last: u32,
    bound: u64,
    local: u64,
}
const NO_ORDER: Order = Order {
    first: NONE,
    last: NONE,
    bound: 0,
    local: 0,
};
/// The result of one probe, answering ordinary and hygienic queries alike.
#[derive(Clone, Copy)]
pub(crate) struct EntryId(u32);

/// First-binding tails at an import worklist boundary. A previously local-only
/// entry can be recorded later, so its creation ordinal is not that boundary.
pub(crate) struct Snapshot {
    tails: Vec<u32>,
}

#[derive(Default)]
pub struct BindingTable {
    slots: Vec<u64>,
    /// `64 - log2(capacity)`: a tag's top bits select its home slot.
    shift: u32,
    entries: Vec<Entry>,
    locals: Vec<Local>,
    /// Single-local entries need no secondary index or history allocation.
    histories: Option<Box<histories::Index>>,
    contextual: Option<Box<contextual::Index>>,
    scopes: Vec<Order>,
}
/// A multiplicative hash of `(scope + 1) << 32 | name` beside the namespace,
/// after CPPGM's `Mix64`. Every key bit reaches the high half, which supplies
/// both the slot tag and (from its top bits) the home slot, so growth rehashes
/// from the slots alone.
const fn tag(scope: ScopeId, name: NameId, namespace: Namespace) -> u64 {
    let key = (((scope.0 as u64) + 1) << 32 | name.0 as u64).rotate_left(3) ^ namespace as u64;
    key.wrapping_mul(0x9e37_79b9_7f4a_7c15) & TAG
}
const fn name_bit(name: NameId) -> u64 {
    1 << (name.0.wrapping_mul(0x9e37_79b9) >> 26)
}
/// Where a probe ended: at the key's entry or at the vacant slot it may take.
enum Probe {
    Found(u32),
    Vacant(usize),
}
impl BindingTable {
    fn home(&self, tag: u64) -> usize {
        usize::try_from(tag >> self.shift).unwrap_or(0)
    }
    /// The name filter bits `scope` has for `name`, ordinary and local.
    fn filters(&self, scope: ScopeId, name: NameId) -> (bool, bool) {
        let bit = name_bit(name);
        self.scopes
            .get(scope.0 as usize)
            .map_or((false, false), |order| {
                (order.bound & bit != 0, order.local & bit != 0)
            })
    }
    fn probe_slots(&self, scope: ScopeId, name: NameId, namespace: Namespace) -> Probe {
        let tag = tag(scope, name, namespace);
        let mask = self.slots.len() - 1;
        let mut index = self.home(tag);
        loop {
            let slot = self.slots[index];
            if slot == 0 {
                return Probe::Vacant(index);
            }
            if slot & TAG == tag
                && let Some(entry) = slot_entry(slot)
            {
                let record = &self.entries[entry as usize];
                if record.scope == scope && record.name == name && record.namespace == namespace {
                    return Probe::Found(entry);
                }
            }
            index = (index + 1) & mask;
        }
    }
    fn search(&self, scope: ScopeId, name: NameId, namespace: Namespace) -> Option<u32> {
        if self.slots.is_empty() {
            return None;
        }
        match self.probe_slots(scope, name, namespace) {
            Probe::Found(entry) => Some(entry),
            Probe::Vacant(_) => None,
        }
    }
    /// The entry of a key that has had an ordinary binding.
    fn find(&self, scope: ScopeId, name: NameId, namespace: Namespace) -> Option<u32> {
        if !self.filters(scope, name).0 {
            return None;
        }
        self.search(scope, name, namespace)
    }
    fn grow(&mut self) {
        let capacity = (self.slots.len() * 2).max(64);
        // A zeroed allocation needs no fill: vacant slots are zero.
        let old = std::mem::replace(&mut self.slots, vec![0; capacity]);
        self.shift = 64 - capacity.trailing_zeros();
        let mask = self.slots.len() - 1;
        for slot in old {
            if slot != 0 {
                let mut index = self.home(slot & TAG);
                while self.slots[index] != 0 {
                    index = (index + 1) & mask;
                }
                self.slots[index] = slot;
            }
        }
    }
    /// Find or create the entry of a key; a new entry has no binding.
    fn entry(
        &mut self,
        scope: ScopeId,
        name: NameId,
        namespace: Namespace,
    ) -> Result<u32, &'static str> {
        // Half full at most, like CPPGM's table.
        let contextual = self.contextual.as_ref().map_or(0, |index| index.len());
        if (self.entries.len() - contextual + 1) * 2 > self.slots.len() {
            self.grow();
        }
        let (bound, local) = self.filters(scope, name);
        let slot = if bound || local {
            match self.probe_slots(scope, name, namespace) {
                Probe::Found(entry) => return Ok(entry),
                Probe::Vacant(slot) => slot,
            }
        } else {
            let mask = self.slots.len() - 1;
            let mut index = self.home(tag(scope, name, namespace));
            while self.slots[index] != 0 {
                index = (index + 1) & mask;
            }
            index
        };
        let id = self.new_entry(scope, name, namespace)?;
        self.slots[slot] = occupied(tag(scope, name, namespace), id);
        Ok(id)
    }
    fn new_entry(
        &mut self,
        scope: ScopeId,
        name: NameId,
        namespace: Namespace,
    ) -> Result<u32, &'static str> {
        let id = u32::try_from(self.entries.len())
            .ok()
            .filter(|id| *id < NONE - 1)
            .ok_or("too many lexical bindings")?;
        self.entries.push(Entry {
            scope,
            name,
            namespace,
            recorded: false,
            binding: None,
            next: NONE,
            local: NONE,
        });
        Ok(id)
    }
    fn order_mut(&mut self, scope: ScopeId) -> &mut Order {
        let index = scope.0 as usize;
        if index >= self.scopes.len() {
            self.scopes.resize(index + 1, NO_ORDER);
        }
        &mut self.scopes[index]
    }
    /// Append a key to its scope's first-binding order once.
    fn record(&mut self, entry: u32) {
        let Some(record) = self.entries.get_mut(entry as usize) else {
            return;
        };
        if record.recorded {
            return;
        }
        record.recorded = true;
        let (scope, bit) = (record.scope, name_bit(record.name));
        let order = self.order_mut(scope);
        order.bound |= bit;
        let last = std::mem::replace(&mut order.last, entry);
        if last == NONE {
            order.first = entry;
        } else {
            self.entries[last as usize].next = entry;
        }
    }
    #[must_use]
    pub fn get(&self, scope: ScopeId, name: NameId, namespace: Namespace) -> Option<&Binding> {
        self.get_key((scope, name, namespace, ItemContext::ROOT))
    }
    pub(crate) fn get_key(&self, key: BindingKey) -> Option<&Binding> {
        let entry = self.find_key(key)?;
        self.entries[entry as usize].binding.as_ref()
    }
    #[must_use]
    pub fn contains(&self, scope: ScopeId, name: NameId, namespace: Namespace) -> bool {
        self.get(scope, name, namespace).is_some()
    }
    /// Install a binding, recording its key's first-binding position.
    ///
    /// # Errors
    /// Rejects a table that cannot be indexed with 32-bit entries.
    pub(crate) fn insert(
        &mut self,
        scope: ScopeId,
        name: NameId,
        namespace: Namespace,
        binding: Binding,
    ) -> Result<Option<Binding>, &'static str> {
        self.insert_key((scope, name, namespace, ItemContext::ROOT), binding)
    }
    pub(crate) fn insert_key(
        &mut self,
        key: BindingKey,
        binding: Binding,
    ) -> Result<Option<Binding>, &'static str> {
        let entry = self.entry_key(key)?;
        self.record(entry);
        Ok(self.entries[entry as usize].binding.replace(binding))
    }
    /// Install a direct declaration unless the key already has a binding that
    /// it may not replace: only shadowing declarations replace direct ones.
    /// One probe decides duplication, first-binding order and the binding.
    ///
    /// # Errors
    /// Rejects a table that cannot be indexed with 32-bit entries.
    pub(crate) fn declare_key(
        &mut self,
        key: BindingKey,
        binding: Binding,
        shadow: bool,
    ) -> Result<bool, &'static str> {
        let entry = self.entry_key(key)?;
        let record = &mut self.entries[entry as usize];
        if let Some(existing) = record.binding
            && !shadow
            && !matches!(existing.origin, Origin::Glob(_))
        {
            return Ok(false);
        }
        record.binding = Some(binding);
        self.record(entry);
        Ok(true)
    }
    /// Remove a binding; its key keeps its recorded position.
    pub(crate) fn remove_key(&mut self, key: BindingKey) {
        if let Some(entry) = self.find_key(key) {
            self.entries[entry as usize].binding = None;
        }
    }
    /// Probe once for a key whose ordinary binding and locals are both needed.
    pub(crate) fn probe(
        &self,
        scope: ScopeId,
        name: NameId,
        namespace: Namespace,
    ) -> Option<EntryId> {
        let (bound, local) = self.filters(scope, name);
        if !bound && !local {
            return None;
        }
        self.search(scope, name, namespace).map(EntryId)
    }
    pub(crate) fn binding(&self, entry: EntryId) -> Option<&Binding> {
        self.entries[entry.0 as usize].binding.as_ref()
    }
    /// The newest local of `context` whose definition precedes `visible`.
    pub(crate) fn local(
        &self,
        entry: EntryId,
        context: LocalContext,
        visible: usize,
    ) -> Option<Binding> {
        let local = self
            .locals
            .get(self.entries[entry.0 as usize].local as usize)?;
        if local.next == NONE {
            return (local.context == context && (local.binding.target.0 as usize) < visible)
                .then_some(local.binding);
        }
        self.histories
            .as_ref()?
            .get(entry.0, context, visible, &self.locals)
    }
    /// Add a hygienic local. Locals stay outside the ordinary binding order;
    /// shadowing keeps the older definitions for definition-site visibility.
    ///
    /// # Errors
    /// Rejects an unshadowed duplicate, unordered definitions and identity overflow.
    pub(crate) fn define_local(
        &mut self,
        key: (ScopeId, NameId, Namespace),
        context: LocalContext,
        binding: Binding,
        shadow: bool,
    ) -> Result<(), &'static str> {
        let entry = self.entry(key.0, key.1, key.2)?;
        self.order_mut(key.0).local |= name_bit(key.1);
        let head = self.entries[entry as usize].local;
        if let Some(previous) = self.locals.get(head as usize) {
            if !shadow
                && if previous.next == NONE {
                    previous.context == context
                } else {
                    self.histories
                        .as_ref()
                        .is_some_and(|index| index.contains(entry, context))
                }
            {
                return Err("duplicate hygienic local binding");
            }
            if previous.binding.target.0 >= binding.target.0 {
                return Err("hygienic local definitions are not in creation order");
            }
        }
        let id = u32::try_from(self.locals.len())
            .ok()
            .filter(|id| *id < NONE)
            .ok_or("too many hygienic locals")?;
        if let Some(previous) = self.locals.get(head as usize) {
            let index = self.histories.get_or_insert_with(Box::default);
            if previous.next == NONE {
                index.push(entry, previous.context, head);
            }
            index.push(entry, context, id);
        }
        self.locals.push(Local {
            context,
            binding,
            next: head,
        });
        self.entries[entry as usize].local = id;
        Ok(())
    }
    /// Recorded keys of `scope` in first-binding order, including removed ones,
    /// with their creation ordinal.
    pub(crate) fn keys(
        &self,
        scope: ScopeId,
    ) -> impl Iterator<Item = ((NameId, Namespace, ItemContext), u32)> {
        let mut current = self
            .scopes
            .get(scope.0 as usize)
            .map_or(NONE, |order| order.first);
        std::iter::from_fn(move || {
            let entry = self.entries.get(current as usize)?;
            #[cfg(test)]
            item_tests::visit_key();
            let item = (
                (entry.name, entry.namespace, self.item_context(current)),
                current,
            );
            current = entry.next;
            Some(item)
        })
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            tails: self.scopes.iter().map(|scope| scope.last).collect(),
        }
    }
    pub(crate) fn keys_before(
        &self,
        scope: ScopeId,
        snapshot: &Snapshot,
    ) -> impl Iterator<Item = (NameId, Namespace, ItemContext)> {
        let tail = snapshot
            .tails
            .get(scope.0 as usize)
            .copied()
            .unwrap_or(NONE);
        let mut ended = tail == NONE;
        self.keys(scope)
            .take_while(move |(_, entry)| {
                let include = !ended;
                ended |= *entry == tail;
                include
            })
            .map(|(key, _)| key)
    }
    /// Root-context bindings of `scope` in first-binding order.
    pub fn scope(&self, scope: ScopeId) -> impl Iterator<Item = (NameId, Namespace, &Binding)> {
        self.keys(scope).filter_map(|(key, entry)| {
            if key.2 != ItemContext::ROOT {
                return None;
            }
            let entry = &self.entries[entry as usize];
            Some((entry.name, entry.namespace, entry.binding.as_ref()?))
        })
    }
    /// Root-context bindings, in creation order of their keys.
    pub fn iter(&self) -> impl Iterator<Item = (ScopeId, NameId, Namespace, &Binding)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let id = u32::try_from(index).ok()?;
                if self.item_context(id) != ItemContext::ROOT {
                    return None;
                }
                Some((
                    entry.scope,
                    entry.name,
                    entry.namespace,
                    entry.binding.as_ref()?,
                ))
            })
    }
    /// Trait discovery includes opaque trait bindings; the method identifier
    /// supplies the independent context used to select their members.
    pub(crate) fn all_bindings(&self) -> impl Iterator<Item = (ScopeId, Namespace, &Binding)> {
        self.entries
            .iter()
            .filter_map(|entry| Some((entry.scope, entry.namespace, entry.binding.as_ref()?)))
    }
    /// Entries created so far; later entries have larger creation ordinals.
    #[must_use]
    pub fn created(&self) -> u32 {
        u32::try_from(self.entries.len()).unwrap_or(NONE)
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/local_histories.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/invariants/item_bindings.rs"]
mod item_tests;
