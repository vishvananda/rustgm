//! A registry owns the only mutable builder; executable queries need its sealed forest.
use super::{proof_attempt::Attempt, retention_work::Budget};
use crate::{
    resolve::{
        expansion::Report,
        specialization_authority::{ImplPolicy, Requests},
    },
    sema::{
        model::Declarations,
        traits::{
            heads::Head,
            implementations::{EntryKey, Identity, Implementations},
        },
    },
    support::hash::{HashMap, HashSet},
    types::{GenericArgs, arena, parameters::predicates::Predicate},
};
use std::num::NonZeroU32;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct PairKey {
    first: EntryKey,
    second: EntryKey,
}
impl PairKey {
    pub(super) fn new(a: EntryKey, b: EntryKey) -> Result<Self, &'static str> {
        if a == b {
            return Err("specialization cannot compare an entry with itself");
        }
        Ok(if a.ordinal() < b.ordinal() {
            Self {
                first: a,
                second: b,
            }
        } else {
            Self {
                first: b,
                second: a,
            }
        })
    }
    pub(super) fn entries(self) -> [EntryKey; 2] {
        [self.first, self.second]
    }
}
#[derive(Clone, Copy)]
pub(super) struct VertexId(NonZeroU32);
impl VertexId {
    pub(super) fn new(index: usize) -> Result<Self, &'static str> {
        u32::try_from(index)
            .ok()
            .and_then(|n| n.checked_add(1))
            .and_then(NonZeroU32::new)
            .map(Self)
            .ok_or("specialization vertex identity overflow")
    }
    pub(super) fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}
#[derive(Clone, Copy)]
pub(super) struct EdgeId(NonZeroU32);
impl EdgeId {
    pub(super) fn new(index: usize) -> Result<Self, &'static str> {
        u32::try_from(index)
            .ok()
            .and_then(|n| n.checked_add(1))
            .and_then(NonZeroU32::new)
            .map(Self)
            .ok_or("specialization edge identity overflow")
    }
    pub(super) fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}
pub(super) struct Witness {
    pub key: EntryKey,
    pub predicate: Predicate,
    pub policy: ImplPolicy,
}
#[derive(Clone, Copy)]
pub(super) struct Relation {
    pub child: EntryKey,
    pub parent: EntryKey,
    pub parent_arguments: GenericArgs,
}
pub(super) enum Comparison {
    Running,
    Pending {
        attempt: Box<Attempt>,
        dependency: Option<crate::sema::traits::selection::Dependency>,
    },
    Ordered(EdgeId),
}
#[derive(Default)]
pub(super) struct Builder {
    pub sources: HashMap<EntryKey, Witness>,
    pub attempts: HashMap<PairKey, Comparison>,
    pub relations: Vec<Relation>,
}
pub(super) struct Vertex {
    pub witness: Witness,
    pub parent: Option<EdgeId>,
    pub depth: u32,
    pub preorder: u32,
    pub subtree_end: u32,
}
pub(super) struct Forest {
    pub vertices: Vec<Vertex>,
    pub parents: Vec<Relation>,
    pub by_entry: HashMap<EntryKey, VertexId>,
    pub families: HashMap<Identity, HashSet<Option<Head>>>,
}
pub(super) enum State {
    Building(Box<Builder>),
    Conforming(Box<Forest>),
    Sealed(Box<Forest>),
    Failed,
}
pub(in crate::sema) struct Graph {
    pub(super) arena: arena::Identity,
    pub(super) requests: Requests,
    pub(super) budget: Budget,
    pub(super) state: State,
}
impl Graph {
    pub(super) fn new(facts: &mut Declarations, report: &Report) -> Result<Box<Self>, String> {
        let identity_bytes = usize::from(facts.types.existing_arena_identity().is_none())
            * arena::Identity::allocation_bytes();
        let budget = Budget::new(size_of::<Self>() + size_of::<Builder>() + identity_bytes)?;
        let requests = Requests::new(report).map_err(|error| error.to_string())?;
        Ok(Box::new(Self {
            arena: facts.types.arena_identity(),
            requests,
            budget,
            state: State::Building(Box::default()),
        }))
    }
    pub(super) fn current(&self, facts: &Declarations) -> Result<(), &'static str> {
        self.budget.charge(1, 0)?;
        if !facts.types.has_identity(&self.arena) {
            return Err("specialization graph belongs to another type arena");
        }
        if matches!(self.state, State::Failed) {
            return Err("specialization graph validation previously failed");
        }
        Ok(())
    }
    pub(in crate::sema) fn require_insertion(
        &self,
        identity: Identity,
        head: Option<Head>,
    ) -> Result<(), &'static str> {
        self.budget.charge(1, 0)?;
        match &self.state {
            State::Building(_) => Ok(()),
            State::Failed => Err("cannot insert into a failed specialization registry"),
            State::Conforming(forest) | State::Sealed(forest) => {
                if forest.families.get(&identity).is_some_and(|heads| {
                    head.is_none()
                        || matches!(head, Some(Head::Projection(_)))
                        || heads.contains(&None)
                        || heads.contains(&head)
                }) {
                    return Err(
                        "compatible implementation cannot be inserted after specialization seal",
                    );
                }
                Ok(())
            }
        }
    }
    pub(super) fn building(&mut self) -> Result<&mut Builder, &'static str> {
        match &mut self.state {
            State::Building(builder) => Ok(builder),
            State::Conforming(_) | State::Sealed(_) => {
                Err("specialization graph is already sealed")
            }
            State::Failed => Err("specialization graph validation previously failed"),
        }
    }
    pub(super) fn witness(
        &self,
        facts: &Declarations,
        report: &Report,
        key: EntryKey,
    ) -> Result<Option<Witness>, String> {
        self.current(facts)?;
        if !matches!(self.state, State::Building(_)) {
            return Err("sealed specialization source witnesses cannot be extended".into());
        }
        let registry = facts
            .generics
            .implementations
            .as_deref()
            .ok_or("specialization registry disappeared")?;
        let header = registry.checked_entry(key)?;
        self.requests
            .validate(report)
            .map_err(|error| error.to_string())?;
        if let State::Building(builder) = &self.state
            && let Some(witness) = builder.sources.get(&key)
        {
            if witness.predicate != header.predicate || Some(witness.policy.source) != header.source
            {
                return Err("specialization source header changed".into());
            }
            witness
                .policy
                .validate(report, &self.requests, |steps, bytes| {
                    self.budget.charge(steps, bytes)
                })
                .map_err(|error| error.to_string())?;
            return Ok(None);
        }
        self.budget
            .charge(1, crate::support::retained::new_map::<EntryKey, Witness>(1))?;
        let policy = self
            .requests
            .capture(
                report,
                &mut report.macro_allowances(),
                key.owner(),
                |steps, bytes| self.budget.charge(steps, bytes),
            )
            .map_err(|error| error.to_string())?;
        let witness = Witness {
            key,
            predicate: header.predicate,
            policy,
        };
        Ok(Some(witness))
    }
    pub(super) fn fail(&mut self) {
        self.state = State::Failed;
    }
}
impl Forest {
    pub(super) fn validate_seal(&self, registry: &Implementations) -> Result<(), &'static str> {
        for vertex in &self.vertices {
            let header = registry.checked_entry(vertex.witness.key)?;
            if header.predicate != vertex.witness.predicate
                || header.source != Some(vertex.witness.policy.source)
            {
                return Err("sealed specialization header changed");
            }
        }
        Ok(())
    }
}
