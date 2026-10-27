//! Prepared headers may discharge declaration obligations; bodies require completed impls.
mod identity;
mod member_contracts;
mod registry_keys;
#[path = "implementations/coherence_owner.rs"]
mod specialization_access;
pub(in crate::sema) use identity::Identity;
pub(in crate::sema) use registry_keys::EntryKey;
mod pending;
mod polarity;
pub(in crate::sema) use pending::{Candidates, Header as PendingHeader};
pub(in crate::sema) use polarity::Polarity;
use polarity::Status;
pub(in crate::sema) mod overlap;
use super::heads::{Head, Subject};
use crate::support::hash::HashMap;
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::{ItemId, Kind, Store, parameters::predicates::Predicate, shape_key::ShapeKey},
};
pub(in crate::sema) use member_contracts::ConstantRequest;
#[cfg(test)]
#[path = "../../../tests/invariants/polarity_registry.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::sema) enum Availability {
    Validated,
    /// Executable preparation waits for compatible headers to finish validation.
    Executable,
    /// The declaration pipeline must validate every header before returning facts.
    Declarations,
}

pub struct Implementation {
    pub owner: DefId,
    pub source: Option<NodeKey>,
    pub predicate: Predicate,
    /// Trait declaration identity paired with an override, or its checked default.
    pub methods: Vec<(DefId, Option<DefId>)>,
    /// Trait associated type identity paired with the implementing definition.
    pub types: Vec<(DefId, DefId)>,
    pub constants: Vec<(DefId, Option<DefId>)>,
}
struct Entry {
    value: Implementation,
    status: Status,
    checked_constants: HashMap<DefId, DefId>,
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Headers {
    #[default]
    Open,
    Constants,
    Validated,
}
#[derive(Default)]
pub(in crate::sema) struct Implementations {
    headers: Headers,
    profile_arena: Option<crate::types::arena::Identity>,
    pending: Option<Box<pending::Pending>>,
    entries: Vec<Entry>,
    owners: HashMap<DefId, usize>,
    traits: HashMap<Identity, Vec<usize>>,
    heads: HashMap<(Identity, Option<Head>), Vec<usize>>,
    open: HashMap<(Identity, Option<Head>), Vec<usize>>,
    closed: HashMap<(Identity, ShapeKey), Vec<usize>>,
    pairs: Vec<(usize, usize)>,
    graph: Option<Box<crate::sema::specialization::Graph>>,
}
impl Implementations {
    pub(in crate::sema) fn pending_owners(&self) -> impl Iterator<Item = DefId> + '_ {
        self.pending.iter().flat_map(|pending| pending.owners())
    }

    /// Cost only: reserve constructs the same pending inventory in both paths.
    pub(in crate::sema) fn reservation_allocation_bound(count: usize) -> usize {
        pending::Pending::allocation_bound(count)
    }

    /// Any explicit impl replaces the structural default for the entire nominal
    /// family, even when the requested application does not match its bounds.
    pub fn auto_override(
        &self,
        store: &Store,
        obligation: crate::types::auto_traits::Obligation,
    ) -> bool {
        let Some(Kind::Adt(item, _)) = store.kind(obligation.ty) else {
            return false;
        };
        let identity = Identity::Auto(obligation.marker);
        let head = Head::Nominal(item);
        self.heads.contains_key(&(identity, Some(head)))
            || self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.may_have_subject(identity, Subject::Rigid(head)))
    }

    /// An actual source auto impl replaces the structural default for its
    /// entire typed head family, independently of a particular RHS match.
    pub(in crate::sema) fn source_auto_override(&self, item: ItemId, kind: Kind) -> bool {
        let identity = Identity::User(item);
        self.heads
            .contains_key(&(identity, Some(Head::canonical(kind))))
            || self.heads.contains_key(&(identity, None))
    }

    /// Reserve the complete source inventory before any early candidate selection.
    pub fn reserve(&mut self, headers: Vec<PendingHeader>) -> Result<(), &'static str> {
        if self.pending.is_some() || !self.entries.is_empty() || self.headers != Headers::Open {
            return Err("implementation candidate inventory is already initialized");
        }
        self.pending = Some(Box::new(pending::Pending::new(headers)?));
        Ok(())
    }
    pub(in crate::sema) fn register_collection_heads(
        &mut self,
        report: &crate::resolve::expansion::Report,
    ) -> Result<(), &'static str> {
        if report.resolution.builtin_traits.collections().is_none()
            && report.resolution.builtin_traits.ranges().is_none()
        {
            return Ok(());
        }
        let pending = self
            .pending
            .as_deref_mut()
            .ok_or("library heads require reserved inventory")?;
        if report.resolution.builtin_traits.collections().is_some() {
            pending.install_collections(report)?;
        }
        if report.resolution.builtin_traits.ranges().is_some() {
            pending.install_ranges(report)?;
        }
        Ok(())
    }
    pub(in crate::sema) fn headers_validated(&self) -> bool {
        self.headers == Headers::Validated
    }
    pub(in crate::sema) fn finish_validation(&mut self) -> Result<(), &'static str> {
        if self.headers == Headers::Open
            || self.entries.iter().any(|entry| !entry.status.validated())
        {
            return Err("implementation inventory contains unvalidated declarations");
        }
        self.headers = Headers::Validated;
        Ok(())
    }
    pub(super) fn may_have_subject(&self, item: ItemId, subject: Subject) -> bool {
        let item = Identity::User(item);
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
        let item = Identity::goal(goal)?;
        let subject = pending_subject(store, context, goal.subject())?;
        let owners = self.pending_candidates(store, context, item, subject, goal.subject())?;
        if !owners.is_empty() {
            return Ok(Candidates::Pending(owners));
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
                let (shape, work) =
                    candidate_shape(store, value).map_err(|_| Error::InvalidIdentity)?;
                context.account_work(work)?;
                Some(shape)
            }
            Ok(_) | Err(Error::Unresolved) => None,
            Err(error) => return Err(error),
        };
        let mut result = Vec::new();
        let mut unvalidated = Vec::new();
        if let Some(shape) = shape {
            self.append_candidates(
                context,
                self.closed.get(&(item, shape)),
                availability,
                &mut result,
                &mut unvalidated,
            )?;
        }
        for head in subject.heads() {
            let bucket = if shape.is_some() {
                self.open.get(&(item, Some(head)))
            } else {
                self.heads.get(&(item, Some(head)))
            };
            self.append_candidates(context, bucket, availability, &mut result, &mut unvalidated)?;
        }
        self.append_candidates(
            context,
            self.heads.get(&(item, None)),
            availability,
            &mut result,
            &mut unvalidated,
        )?;
        if !unvalidated.is_empty() {
            unvalidated.sort_unstable_by_key(|id| id.0);
            return Ok(Candidates::Unvalidated(unvalidated));
        }
        result.sort_unstable_by_key(|id| id.0);
        Ok(Candidates::Ready(result))
    }
    fn pending_candidates(
        &self,
        store: &Store,
        context: &mut crate::types::infer::Context,
        item: Identity,
        subject: Option<Subject>,
        term: crate::types::infer::TermId,
    ) -> Result<Vec<DefId>, crate::types::infer::Error> {
        let Some(pending) = &self.pending else {
            return Ok(Vec::new());
        };
        let mut owners = pending.candidates(context, item, subject)?;
        pending.filter_collections(store, context, term, &mut owners)?;
        Ok(owners)
    }
    pub(in crate::sema) fn selection_goal_ready(
        &self,
        store: &Store,
        context: &mut crate::types::infer::Context,
        goal: &crate::types::infer::predicates::Predicate,
    ) -> Result<bool, crate::types::infer::Error> {
        if self.selection_headers_ready() {
            return Ok(true);
        }
        if !self.constant_headers_ready() {
            return Ok(false);
        }
        if goal.trait_arguments().is_none()
            && !matches!(
                goal.builtin_bound(),
                Some(crate::types::parameters::predicates::Bound::Auto(_))
            )
        {
            return Ok(true);
        }
        let item = Identity::goal(goal)?;
        let subject = pending_subject(store, context, goal.subject())?;
        Ok(self
            .pending_candidates(store, context, item, subject, goal.subject())?
            .is_empty())
    }
    pub(in crate::sema) fn selection_predicate_ready(
        &self,
        store: &Store,
        context: &mut crate::types::infer::Context,
        goal: crate::types::parameters::predicates::Predicate,
    ) -> Result<bool, crate::types::infer::Error> {
        use crate::types::{infer::Error, parameters::predicates::Bound};
        if self.selection_headers_ready() {
            return Ok(true);
        }
        if !self.constant_headers_ready() {
            return Ok(false);
        }
        let item = match goal.bound {
            Bound::Trait(application)
            | Bound::ConstTrait(application)
            | Bound::MaybeConstTrait(application) => Identity::User(
                store
                    .trait_application(application)
                    .ok_or(Error::InvalidIdentity)?
                    .item,
            ),
            Bound::Auto(marker) => Identity::Auto(marker),
            _ => return Ok(true),
        };
        let Some(pending) = &self.pending else {
            return Ok(true);
        };
        let subject = Subject::Rigid(Head::canonical(
            store.kind(goal.ty).ok_or(Error::InvalidIdentity)?,
        ));
        let mut owners = pending.candidates(context, item, Some(subject))?;
        pending.filter_known_collections(store, context, goal.ty, &mut owners)?;
        Ok(owners.is_empty())
    }
    fn append_candidates(
        &self,
        context: &mut crate::types::infer::Context,
        bucket: Option<&Vec<usize>>,
        availability: Availability,
        output: &mut Vec<DefId>,
        unvalidated: &mut Vec<DefId>,
    ) -> Result<(), crate::types::infer::Error> {
        context.account_work(bucket.map_or(0, Vec::len))?;
        for &index in bucket.into_iter().flatten() {
            let entry = &self.entries[index];
            if entry.status.validated() || matches!(availability, Availability::Declarations) {
                output.push(entry.value.owner);
            } else if matches!(availability, Availability::Executable) {
                unvalidated.push(entry.value.owner);
            }
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn prepare(
        &mut self,
        store: &mut Store,
        item: impl Into<Identity>,
        value: Implementation,
    ) -> Result<(), &'static str> {
        self.prepare_with_status(store, item.into(), value, Status::PositiveHeader)
    }

    fn prepare_with_status(
        &mut self,
        store: &mut Store,
        item: Identity,
        value: Implementation,
        status: Status,
    ) -> Result<(), &'static str> {
        self.prepare_with_observer::<false>(store, item, value, status, &|_, _| Ok(()))
            .map(|_| ())
    }

    fn insertion_growth(
        &self,
        item: Identity,
        head: Option<Head>,
        shape: Option<ShapeKey>,
        pairs: usize,
    ) -> crate::support::retained::Growth {
        use crate::support::retained::Growth;
        let mut growth = Growth::vector::<Entry>(self.entries.len(), self.entries.capacity(), 1);
        growth.include(Growth::map::<DefId, usize>(
            self.owners.len(),
            self.owners.capacity(),
            1,
        ));
        growth.include(bucket_growth(&self.traits, &item));
        growth.include(bucket_growth(&self.heads, &(item, head)));
        growth.include(if let Some(shape) = shape {
            bucket_growth(&self.closed, &(item, shape))
        } else {
            bucket_growth(&self.open, &(item, head))
        });
        growth.include(Growth::vector::<(usize, usize)>(
            self.pairs.len(),
            self.pairs.capacity(),
            pairs,
        ));
        growth
    }

    fn insertion_allocation(
        &self,
        item: Identity,
        head: Option<Head>,
        shape: Option<ShapeKey>,
    ) -> usize {
        use crate::support::retained::{map, vector};
        vector(&self.entries)
            .saturating_add(vector(&self.pairs))
            .saturating_add(map(&self.owners))
            .saturating_add(map(&self.traits))
            .saturating_add(map(&self.heads))
            .saturating_add(map(&self.closed))
            .saturating_add(map(&self.open))
            .saturating_add(self.traits.get(&item).map_or(0, vector))
            .saturating_add(self.heads.get(&(item, head)).map_or(0, vector))
            .saturating_add(if let Some(shape) = shape {
                self.closed.get(&(item, shape)).map_or(0, vector)
            } else {
                self.open.get(&(item, head)).map_or(0, vector)
            })
    }

    fn prepare_with_observer<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        item: Identity,
        value: Implementation,
        status: Status,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<usize, &'static str> {
        if self.headers != Headers::Open
            && (!value.constants.is_empty()
                || !self
                    .pending
                    .as_deref()
                    .is_some_and(|pending| pending.collection_owner(value.owner)))
        {
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
        if let Some(graph) = &self.graph {
            graph.require_insertion(item, head)?;
        }
        if let Some(pending) = &self.pending {
            pending.validate(value.owner, item, head)?;
        }
        if CHARGED {
            let arguments = value
                .predicate
                .bound
                .trait_parts()
                .and_then(|(id, _)| store.trait_application(id))
                .and_then(|application| store.arguments(application.arguments))
                .map_or(0, |arguments| arguments.len());
            charge(arguments.saturating_add(1), 0)?;
        }
        let mut shape_bytes = 0;
        let shape = if value.predicate.contains_parameters(store)? {
            None
        } else {
            Some(if CHARGED {
                let mut predicate = value.predicate;
                if let Some((application, _)) = predicate.bound.trait_parts() {
                    predicate.bound =
                        crate::types::parameters::predicates::Bound::Trait(application);
                }
                let (key, _, bytes) = store.shape_key_charged(predicate, charge)?;
                shape_bytes = bytes;
                key
            } else {
                candidate_shape(store, value.predicate)?.0
            })
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
        let previous = if CHARGED {
            let growth = self.insertion_growth(item, head, shape, count);
            charge(
                growth.work.saturating_add(count).saturating_add(1),
                growth.bytes,
            )?;
            self.insertion_allocation(item, head, shape)
        } else {
            0
        };
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
            status,
            checked_constants: HashMap::default(),
        });
        Ok(if CHARGED {
            shape_bytes.saturating_add(
                self.insertion_allocation(item, head, shape)
                    .saturating_sub(previous),
            )
        } else {
            0
        })
    }
    pub fn headers(&self) -> impl Iterator<Item = &Implementation> {
        self.entries.iter().map(|entry| &entry.value)
    }
    pub(in crate::sema) fn header_count(&self) -> usize {
        self.entries.len()
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
            .filter(|entry| entry.status.validated())
            .map(|entry| &entry.value)
    }
    pub fn is_validated(&self, owner: DefId) -> bool {
        self.owners
            .get(&owner)
            .is_some_and(|&index| self.entries[index].status.validated())
    }
    pub fn method(&self, owner: DefId, declaration: DefId) -> Option<DefId> {
        let entry = &self.entries[*self.owners.get(&owner)?];
        if !entry.status.validated() {
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
        if !entry.status.validated() {
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
        if entry.status.validated() {
            return Err("trait implementation was validated twice");
        }
        if entry.status.negative()
            && (!methods.is_empty() || !types.is_empty() || !constants.is_empty())
        {
            return Err("negative implementation cannot publish members");
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
        entry.status = entry.status.complete();
        Ok(())
    }
}

fn pending_subject(
    store: &Store,
    context: &mut crate::types::infer::Context,
    term: crate::types::infer::TermId,
) -> Result<Option<Subject>, crate::types::infer::Error> {
    use crate::types::infer::Error;
    match Subject::query(store, context, term) {
        Ok(subject) => Ok(subject),
        Err(Error::Normalization) => Ok(Some(Subject::Rigid(Head::Projection(
            context
                .projection_item(store, term)?
                .ok_or(Error::InvalidIdentity)?,
        )))),
        Err(error) => Err(error),
    }
}

/// Coarse input buckets do not authorize a trait use. Effects remain exact in
/// canonical headers, trial views, evidence and every proof/memo key.
fn candidate_shape(
    store: &mut Store,
    mut predicate: Predicate,
) -> Result<(ShapeKey, usize), &'static str> {
    if let Some((application, _)) = predicate.bound.trait_parts() {
        predicate.bound = crate::types::parameters::predicates::Bound::Trait(application);
    }
    store.shape_key(predicate)
}

fn bucket_growth<K: Eq + std::hash::Hash>(
    map: &HashMap<K, Vec<usize>>,
    key: &K,
) -> crate::support::retained::Growth {
    use crate::support::retained::Growth;
    let bucket = map.get(key);
    let mut growth =
        Growth::map::<K, Vec<usize>>(map.len(), map.capacity(), usize::from(bucket.is_none()));
    growth.include(Growth::vector::<usize>(
        bucket.map_or(0, Vec::len),
        bucket.map_or(0, Vec::capacity),
        1,
    ));
    growth
}
