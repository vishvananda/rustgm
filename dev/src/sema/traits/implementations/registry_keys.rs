//! Entry handles are made only by this actual registry and rechecked on use.
use super::{DefId, Implementation, Implementations};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) struct EntryKey {
    index: u32,
    owner: DefId,
}
impl EntryKey {
    pub(in crate::sema) const fn owner(self) -> DefId {
        self.owner
    }
    pub(in crate::sema) const fn ordinal(self) -> u32 {
        self.index
    }
}
impl Implementations {
    pub(in crate::sema) fn entry_key(&self, owner: DefId) -> Result<EntryKey, &'static str> {
        let &index = self
            .owners
            .get(&owner)
            .ok_or("specialization owner is not registered")?;
        if self
            .entries
            .get(index)
            .is_none_or(|entry| entry.value.owner != owner)
        {
            return Err("specialization registry owner index changed");
        }
        Ok(EntryKey {
            index: u32::try_from(index).map_err(|_| "specialization registry exceeds key range")?,
            owner,
        })
    }
    pub(in crate::sema) fn checked_entry(
        &self,
        key: EntryKey,
    ) -> Result<&Implementation, &'static str> {
        let value = &self
            .entries
            .get(key.index as usize)
            .ok_or("specialization entry disappeared")?
            .value;
        if value.owner != key.owner || self.owners.get(&key.owner) != Some(&(key.index as usize)) {
            return Err("specialization registry entry identity changed");
        }
        Ok(value)
    }
    pub(in crate::sema) fn direct_method(
        &self,
        key: EntryKey,
        declaration: DefId,
    ) -> Result<Option<DefId>, &'static str> {
        let header = self.checked_entry(key)?;
        if !self.is_validated(key.owner()) {
            return Err("specialization method owner is not validated");
        }
        let index = header
            .methods
            .binary_search_by_key(&declaration.0, |(id, _)| id.0)
            .map_err(|_| "specialization method is not a member of this implementation")?;
        Ok(header.methods[index].1)
    }
}
