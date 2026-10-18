//! Prepared headers may discharge declaration obligations; bodies require completed impls.
mod member_contracts;
mod pending;
pub(in crate::sema) use pending::{Candidates, Header as PendingHeader};
pub(in crate::sema) mod overlap;
use super::heads::{Head, Subject};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::{ItemId, Kind, Store, parameters::predicates::Predicate, shape_key::ShapeKey},
};
pub(in crate::sema) use member_contracts::ConstantRequest;
use std::collections::HashMap;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(in crate::sema) enum Availability {
    Validated,
    /// The declaration pipeline must validate every header before returning facts.
    Declarations,
}

pub struct Implementation {
    pub owner: DefId,
    pub source: NodeKey,
    pub predicate: Predicate,
    /// Trait declaration identity paired with an override, or its checked default.
    pub methods: Vec<(DefId, Option<DefId>)>,
    /// Trait associated type identity paired with the implementing definition.
    pub types: Vec<(DefId, DefId)>,
    pub constants: Vec<(DefId, Option<DefId>)>,
}
struct Entry {
    value: Implementation,
    validated: bool,
    checked_constants: HashMap<DefId, DefId>,
}
#[derive(Default)]
pub(in crate::sema) struct Implementations {
    constant_headers_ready: bool,
    pending: Option<Box<pending::Pending>>,
    entries: Vec<Entry>,
    owners: HashMap<DefId, usize>,
    traits: HashMap<ItemId, Vec<usize>>,
    heads: HashMap<(ItemId, Option<Head>), Vec<usize>>,
    open: HashMap<(ItemId, Option<Head>), Vec<usize>>,
    closed: HashMap<(ItemId, ShapeKey), Vec<usize>>,
    pairs: Vec<(usize, usize)>,
}
impl Implementations {
    /// Reserve the complete source inventory before any early candidate selection.
    pub fn reserve(&mut self, headers: Vec<PendingHeader>) -> Result<(), &'static str> {
        if self.pending.is_some() || !self.entries.is_empty() || self.constant_headers_ready {
            return Err("implementation candidate inventory is already initialized");
        }
        self.pending = Some(Box::new(pending::Pending::new(headers)?));
        Ok(())
    }
    pub(super) fn may_have_subject(&self, item: ItemId, subject: Subject) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|pending| pending.may_have_subject(item, subject))
            || self.heads.contains_key(&(item, None))
            || subject
                .heads()
                .any(|head| self.heads.contains_key(&(item, Some(head))))
    }
    pub fn candidates(
        &mut self,
        store: &mut Store,
        context: &mut crate::types::infer::Context,
        goal: &crate::types::infer::predicates::Predicate,
        availability: Availability,
    ) -> Result<Candidates, crate::types::infer::Error> {
        use crate::types::infer::Error;
        let (item, _) = goal.trait_arguments().ok_or(Error::InvalidIdentity)?;
        let subject = match Subject::query(store, context, goal.subject()) {
            Ok(subject) => subject,
            Err(Error::Normalization) => Some(Subject::Rigid(Head::Projection(
                context
                    .projection_item(store, goal.subject())?
                    .ok_or(Error::InvalidIdentity)?,
            ))),
            Err(error) => return Err(error),
        };
        if let Some(pending) = &self.pending {
            let owners = pending.candidates(context, item, subject)?;
            if !owners.is_empty() {
                return Ok(Candidates::Pending(owners));
            }
        }
        let Some(subject) = subject else {
            return Ok(Candidates::Ready(Vec::new()));
        };
        let shape = match context.resolved_predicate(store, goal) {
            Ok(value)
                if !value
                    .contains_parameters(store)
                    .map_err(|_| Error::InvalidIdentity)? =>
            {
                let (shape, work) = store.shape_key(value).map_err(|_| Error::InvalidIdentity)?;
                context.account_work(work)?;
                Some(shape)
            }
            Ok(_) | Err(Error::Unresolved) => None,
            Err(error) => return Err(error),
        };
        let mut result = Vec::new();
        if let Some(shape) = shape {
            self.append_candidates(
                context,
                self.closed.get(&(item, shape)),
                availability,
                &mut result,
            )?;
        }
        for head in subject.heads() {
            let bucket = if shape.is_some() {
                self.open.get(&(item, Some(head)))
            } else {
                self.heads.get(&(item, Some(head)))
            };
            self.append_candidates(context, bucket, availability, &mut result)?;
        }
        self.append_candidates(
            context,
            self.heads.get(&(item, None)),
            availability,
            &mut result,
        )?;
        result.sort_unstable_by_key(|id| id.0);
        Ok(Candidates::Ready(result))
    }
    fn append_candidates(
        &self,
        context: &mut crate::types::infer::Context,
        bucket: Option<&Vec<usize>>,
        availability: Availability,
        output: &mut Vec<DefId>,
    ) -> Result<(), crate::types::infer::Error> {
        context.account_work(bucket.map_or(0, Vec::len))?;
        for &index in bucket.into_iter().flatten() {
            let entry = &self.entries[index];
            if entry.validated || matches!(availability, Availability::Declarations) {
                output.push(entry.value.owner);
            }
        }
        Ok(())
    }
    pub fn prepare(
        &mut self,
        store: &mut Store,
        item: ItemId,
        value: Implementation,
    ) -> Result<(), &'static str> {
        if self.constant_headers_ready {
            return Err("implementation candidate headers are already frozen");
        }
        if self.owners.contains_key(&value.owner) {
            return Err("trait implementation was prepared twice");
        }
        let index = self.entries.len();
        let kind = store
            .kind(value.predicate.ty)
            .ok_or("invalid implementation subject")?;
        let head = if matches!(kind, Kind::Parameter(_)) {
            None
        } else {
            Some(Head::canonical(kind))
        };
        if let Some(pending) = &self.pending {
            pending.validate(value.owner, item, head)?;
        }
        let shape = if value.predicate.contains_parameters(store)? {
            None
        } else {
            Some(store.shape_key(value.predicate)?.0)
        };
        let (first, second, third) = if let Some(shape) = shape {
            (
                self.closed
                    .get(&(item, shape))
                    .map_or(&[][..], Vec::as_slice),
                self.open.get(&(item, head)).map_or(&[][..], Vec::as_slice),
                self.open.get(&(item, None)).map_or(&[][..], Vec::as_slice),
            )
        } else if head.is_none() {
            (
                self.traits.get(&item).map_or(&[][..], Vec::as_slice),
                &[][..],
                &[][..],
            )
        } else {
            (
                self.heads.get(&(item, head)).map_or(&[][..], Vec::as_slice),
                self.heads.get(&(item, None)).map_or(&[][..], Vec::as_slice),
                &[][..],
            )
        };
        let count = first
            .len()
            .saturating_add(second.len())
            .saturating_add(third.len());
        if self.pairs.len().saturating_add(count) > 1_000_000 {
            return Err("trait implementation overlap pair budget exceeded");
        }
        self.pairs.extend(
            first
                .iter()
                .chain(second)
                .chain(third)
                .map(|&other| (other, index)),
        );
        self.owners.insert(value.owner, index);
        self.traits.entry(item).or_default().push(index);
        self.heads.entry((item, head)).or_default().push(index);
        if let Some(shape) = shape {
            self.closed.entry((item, shape)).or_default().push(index);
        } else {
            self.open.entry((item, head)).or_default().push(index);
        }
        if let Some(pending) = &mut self.pending {
            pending.complete(value.owner);
        }
        self.entries.push(Entry {
            value,
            validated: false,
            checked_constants: HashMap::new(),
        });
        Ok(())
    }
    pub fn headers(&self) -> impl Iterator<Item = &Implementation> {
        self.entries.iter().map(|entry| &entry.value)
    }
    pub fn header(&self, owner: DefId) -> Option<&Implementation> {
        self.owners
            .get(&owner)
            .map(|&index| &self.entries[index].value)
    }
    pub fn take_overlap_pairs(&mut self) -> Vec<(Predicate, DefId, Predicate, DefId)> {
        std::mem::take(&mut self.pairs)
            .into_iter()
            .map(|(a, b)| {
                let a = &self.entries[a].value;
                let b = &self.entries[b].value;
                (a.predicate, a.owner, b.predicate, b.owner)
            })
            .collect()
    }
    pub fn validated(&self) -> impl Iterator<Item = &Implementation> {
        self.entries
            .iter()
            .filter(|entry| entry.validated)
            .map(|entry| &entry.value)
    }
    pub fn is_validated(&self, owner: DefId) -> bool {
        self.owners
            .get(&owner)
            .is_some_and(|&index| self.entries[index].validated)
    }
    pub fn method(&self, owner: DefId, declaration: DefId) -> Option<DefId> {
        let entry = &self.entries[*self.owners.get(&owner)?];
        if !entry.validated {
            return None;
        }
        let index = entry
            .value
            .methods
            .binary_search_by_key(&declaration.0, |(id, _)| id.0)
            .ok()?;
        Some(entry.value.methods[index].1.unwrap_or(declaration))
    }
    pub fn constant(&self, owner: DefId, declaration: DefId) -> Option<DefId> {
        let entry = &self.entries[*self.owners.get(&owner)?];
        if !entry.validated {
            return None;
        }
        let index = entry
            .value
            .constants
            .binary_search_by_key(&declaration.0, |(id, _)| id.0)
            .ok()?;
        Some(entry.value.constants[index].1.unwrap_or(declaration))
    }
    pub fn complete(
        &mut self,
        owner: DefId,
        mut methods: Vec<(DefId, Option<DefId>)>,
        mut types: Vec<(DefId, DefId)>,
        mut constants: Vec<(DefId, Option<DefId>)>,
    ) -> Result<(), &'static str> {
        let index = *self
            .owners
            .get(&owner)
            .ok_or("missing prepared trait implementation")?;
        let entry = &mut self.entries[index];
        if entry.validated {
            return Err("trait implementation was validated twice");
        }
        methods.sort_unstable_by_key(|(declaration, _)| declaration.0);
        if methods.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err("implementation repeats a selected method declaration");
        }
        types.sort_unstable_by_key(|(declaration, _)| declaration.0);
        if types.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err("implementation repeats an associated type declaration");
        }
        constants.sort_unstable_by_key(|(declaration, _)| declaration.0);
        if constants.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err("implementation repeats an associated constant declaration");
        }
        for (&declaration, &selected) in &entry.checked_constants {
            let index = constants
                .binary_search_by_key(&declaration.0, |(id, _)| id.0)
                .map_err(|_| "checked constant is absent from completed implementation")?;
            if constants[index].1.unwrap_or(declaration) != selected {
                return Err("completed implementation changed a checked constant selection");
            }
        }
        entry.value.methods = methods;
        entry.value.types = types;
        entry.value.constants = constants;
        entry.validated = true;
        Ok(())
    }
}
