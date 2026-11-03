//! Canonical boolean DAGs keep reference/region obligations symbolic and shared.
use super::Budget;
use crate::{
    support::{hash::HashMap, retained::Growth},
    types::{RegionId, TypeId},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Obligation {
    Transmutable { source: TypeId, destination: TypeId },
    Outlives { long: RegionId, short: RegionId },
    Freeze(TypeId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConditionId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    No,
    Yes,
    Obligation(Obligation),
    All(ConditionId, ConditionId),
    Any(ConditionId, ConditionId),
}
pub struct Proof {
    pub root: ConditionId,
    nodes: Vec<Condition>,
}
impl Proof {
    #[must_use]
    pub fn condition(&self, id: ConditionId) -> Option<Condition> {
        self.nodes.get(id.0 as usize).copied()
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
pub(super) struct Conditions {
    nodes: Vec<Condition>,
    index: HashMap<Condition, ConditionId>,
}
impl Conditions {
    pub const NO: ConditionId = ConditionId(0);
    pub const YES: ConditionId = ConditionId(1);
    pub fn new(budget: &mut Budget<'_>) -> Result<Self, &'static str> {
        let mut result = Self {
            nodes: Vec::new(),
            index: HashMap::default(),
        };
        result.intern(Condition::No, budget)?;
        result.intern(Condition::Yes, budget)?;
        Ok(result)
    }
    pub fn finish(self, root: ConditionId) -> Proof {
        Proof {
            root,
            nodes: self.nodes,
        }
    }
    pub fn obligation(
        &mut self,
        value: Obligation,
        budget: &mut Budget<'_>,
    ) -> Result<ConditionId, &'static str> {
        self.intern(Condition::Obligation(value), budget)
    }
    fn intern(
        &mut self,
        value: Condition,
        budget: &mut Budget<'_>,
    ) -> Result<ConditionId, &'static str> {
        budget.charge(1, 0)?;
        if let Some(&id) = self.index.get(&value) {
            return Ok(id);
        }
        let mut growth = Growth::vector::<Condition>(self.nodes.len(), self.nodes.capacity(), 1);
        growth.include(Growth::map::<Condition, ConditionId>(
            self.index.len(),
            self.index.capacity(),
            1,
        ));
        budget.charge(growth.work, growth.bytes)?;
        self.nodes
            .try_reserve(1)
            .map_err(|_| "condition allocation failed")?;
        self.index
            .try_reserve(1)
            .map_err(|_| "condition index allocation failed")?;
        let id = ConditionId(
            u32::try_from(self.nodes.len()).map_err(|_| "condition identity overflow")?,
        );
        self.nodes.push(value);
        self.index.insert(value, id);
        Ok(id)
    }
    pub fn combine(
        &mut self,
        all: bool,
        a: ConditionId,
        b: ConditionId,
        budget: &mut Budget<'_>,
    ) -> Result<ConditionId, &'static str> {
        budget.charge(1, 0)?;
        let (identity, absorbing) = if all {
            (Self::YES, Self::NO)
        } else {
            (Self::NO, Self::YES)
        };
        if a == absorbing || b == absorbing {
            return Ok(absorbing);
        }
        if a == identity || a == b {
            return Ok(b);
        }
        if b == identity {
            return Ok(a);
        }
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        self.intern(
            if all {
                Condition::All(a, b)
            } else {
                Condition::Any(a, b)
            },
            budget,
        )
    }
}
