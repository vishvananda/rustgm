//! Alias contracts retain source quantifiers until their final slot counts are known.
use super::{DefId, Obligations, Requirements};
use crate::types::lifetime_proof::ScopeId;

struct Binder {
    parent: ScopeId,
    depth: u16,
    count: Option<u32>,
}
#[derive(Default)]
pub(super) struct Scopes {
    bindings: Vec<Binder>,
}
pub(super) struct Mapping {
    root: ScopeId,
    nested: Vec<(ScopeId, u16)>,
}
impl Mapping {
    pub fn get(&self, scope: ScopeId) -> Result<(ScopeId, u16), &'static str> {
        if scope == ScopeId::ROOT {
            return Ok((self.root, 0));
        }
        self.nested
            .get(scope.0 as usize - 1)
            .copied()
            .ok_or("invalid alias proof scope")
    }
}
impl Scopes {
    fn scope(&mut self, parent: ScopeId) -> Result<ScopeId, &'static str> {
        let depth = if parent == ScopeId::ROOT {
            0
        } else {
            self.bindings
                .get(parent.0 as usize - 1)
                .ok_or("invalid alias scope parent")?
                .depth
        };
        let depth = depth
            .checked_add(1)
            .filter(|d| *d < 0x8000)
            .ok_or("alias proof scope nesting exhausted")?;
        if self.bindings.len() >= 1_000_000 {
            return Err("alias proof scope budget exhausted");
        }
        self.bindings.push(Binder {
            parent,
            depth,
            count: None,
        });
        let id =
            ScopeId(u32::try_from(self.bindings.len()).map_err(|_| "too many alias proof scopes")?);
        Ok(id)
    }
    pub fn len(&self) -> usize {
        self.bindings.len()
    }
    pub fn complete(&self) -> bool {
        self.bindings.iter().all(|b| b.count.is_some())
    }
    pub fn instantiate(
        scopes: Option<&Self>,
        result: &mut Obligations,
        root: ScopeId,
    ) -> Result<Mapping, &'static str> {
        let mut mapping = Mapping {
            root,
            nested: Vec::with_capacity(scopes.map_or(0, |scopes| scopes.bindings.len())),
        };
        if let Some(scopes) = scopes {
            for binder in &scopes.bindings {
                let parent = mapping.get(binder.parent)?.0;
                let id =
                    result.scope(parent, binder.count.ok_or("unfinished alias proof binder")?)?;
                mapping.nested.push((id, binder.depth));
            }
        }
        Ok(mapping)
    }
}
impl Requirements {
    pub(in crate::sema) fn scope(
        &mut self,
        owner: DefId,
        parameters: DefId,
        parent: ScopeId,
    ) -> Result<ScopeId, &'static str> {
        self.building(owner, parameters)?
            .scopes
            .get_or_insert_with(Default::default)
            .scope(parent)
    }
    pub(in crate::sema) fn complete_scope(&mut self, owner: DefId, scope: ScopeId, count: u32) {
        if let Some(scopes) = self
            .data
            .as_mut()
            .and_then(|data| data.contracts.get_mut(&owner))
            .and_then(|c| c.scopes.as_mut())
            && let Some(binder) = scope
                .0
                .checked_sub(1)
                .and_then(|id| scopes.bindings.get_mut(id as usize))
        {
            binder.count = Some(count);
        }
    }
}
