//! Resolved free bindings cross each closure boundary once, independently of capture mode.
use crate::resolve::model::{DefId, DefKind, Namespace, Resolution, ScopeId, Target};
use std::collections::{HashMap, HashSet};

/// Lexical type inputs only. A mention does not establish a read, loan or move.
#[derive(Default)]
pub struct Imports {
    bindings: HashMap<ScopeId, Vec<DefId>>,
    work: usize,
}
impl Imports {
    /// Index resolved references without walking source subtrees for every closure.
    /// # Errors
    /// Rejects malformed scope links, item-crossing locals and exhausted work.
    pub fn collect(resolution: &Resolution) -> Result<Self, &'static str> {
        let first = resolution
            .scopes
            .iter()
            .position(|scope| scope.label_barrier && !scope.barrier);
        let mut result = Self {
            work: first.map_or(resolution.scopes.len(), |index| index + 1),
            ..Self::default()
        };
        if result.work > 1_000_000 {
            return Err("closure import work limit exceeded");
        }
        if first.is_none() {
            return Ok(result);
        }
        let owners = result.owners(resolution)?;
        let mut seen = HashSet::new();
        for reference in &resolution.references {
            result.charge()?;
            if reference.namespace != Namespace::Value {
                continue;
            }
            let Target::Definition(binding) = reference.target else {
                continue;
            };
            let definition = resolution
                .definitions
                .get(binding.0 as usize)
                .ok_or("invalid closure import definition")?;
            if !matches!(definition.kind, DefKind::Local | DefKind::Parameter) {
                continue;
            }
            let declared = *owners
                .get(definition.scope.0 as usize)
                .ok_or("invalid closure binding scope")?;
            let mut owner = *owners
                .get(reference.scope.0 as usize)
                .ok_or("invalid closure reference scope")?;
            while owner != declared {
                result.charge()?;
                let scope = owner.ok_or("closure reference escaped its binding owner")?;
                let frame = &resolution.scopes[scope.0 as usize];
                if frame.barrier || !frame.label_barrier {
                    return Err("closure import crosses an item boundary");
                }
                if !seen.insert((scope, binding)) {
                    break;
                }
                result.bindings.entry(scope).or_default().push(binding);
                owner = frame.parent.and_then(|parent| owners[parent.0 as usize]);
            }
        }
        Ok(result)
    }
    fn charge(&mut self) -> Result<(), &'static str> {
        self.work = self
            .work
            .checked_add(1)
            .ok_or("closure import work overflow")?;
        if self.work > 1_000_000 {
            return Err("closure import work limit exceeded");
        }
        Ok(())
    }
    fn owners(&mut self, resolution: &Resolution) -> Result<Vec<Option<ScopeId>>, &'static str> {
        let mut owners = vec![None; resolution.scopes.len()];
        let mut states = vec![0_u8; owners.len()];
        let mut pending = Vec::new();
        for index in 0..owners.len() {
            let mut cursor = Some(index);
            while let Some(current) = cursor {
                self.charge()?;
                match states.get(current) {
                    Some(2) => break,
                    Some(1) => return Err("cyclic closure scope parents"),
                    Some(0) => {}
                    _ => return Err("invalid closure scope parent"),
                }
                states[current] = 1;
                pending.push(current);
                cursor = resolution.scopes[current]
                    .parent
                    .map(|parent| parent.0 as usize);
            }
            while let Some(current) = pending.pop() {
                self.charge()?;
                let frame = &resolution.scopes[current];
                owners[current] = if frame.barrier || frame.label_barrier {
                    Some(ScopeId(
                        u32::try_from(current).map_err(|_| "closure scope identity overflow")?,
                    ))
                } else {
                    frame.parent.and_then(|parent| owners[parent.0 as usize])
                };
                states[current] = 2;
            }
        }
        Ok(owners)
    }
    #[must_use]
    pub fn bindings(&self, scope: ScopeId) -> &[DefId] {
        self.bindings.get(&scope).map_or(&[], Vec::as_slice)
    }
    /// Actual scope/reference/import operations, including repeated lookup attempts.
    #[must_use]
    pub const fn work(&self) -> usize {
        self.work
    }
}
