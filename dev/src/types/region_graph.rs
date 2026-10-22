//! Bounded outlives propagation; universal assumptions never become obligations.
mod components;
mod members;
mod output;
mod quantifier_elimination;
pub(crate) use output::{Conditional, OutputError};
mod verify;
use crate::support::hash::{HashMap, HashSet};
pub use members::{ChoicesId, MembershipId};
pub(crate) use verify::Answers;
pub use verify::VerificationId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Region(pub u32);
impl Region {
    pub const STATIC: Self = Self(0);
    /// Upper bound on a borrow of storage owned by the current function.
    /// Finer storage/loan liveness is checked independently in the body CFG.
    pub const LOCAL: Self = Self(1);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Static,
    Local,
    Universal { universe: u32 },
    Variable { universe: u32 },
}
impl Kind {
    const fn universe(self) -> u32 {
        match self {
            Self::Static | Self::Local => 0,
            Self::Universal { universe } | Self::Variable { universe } => universe,
        }
    }
    const fn rigid(self) -> bool {
        !matches!(self, Self::Variable { .. })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedOutput,
    Identity,
    Budget,
    Membership {
        source: u32,
    },
    Verification {
        source: u32,
    },
    Unsatisfied {
        source: u32,
        longer: Region,
        shorter: Region,
    },
}
impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(match self {
            Self::UnsupportedOutput => "conditional nested output shape is not implemented",
            Self::Identity => "invalid lifetime constraint identity",
            Self::Budget => "lifetime constraint work budget exceeded",
            Self::Membership { .. } => "hidden lifetime has no unique permitted choice",
            Self::Verification { .. } => "required type does not outlive its target",
            Self::Unsatisfied { .. } => "required lifetime does not outlive its target",
        })
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Copy)]
struct Edge {
    target: Region,
    source: u32,
    next: Option<usize>,
}
/// All caches belong to this one body/environment. Mutations precede `solve`.
pub struct Graph {
    kinds: Vec<Kind>,
    assumptions: Vec<Edge>,
    assumed_heads: Vec<Option<usize>>,
    obligations: Vec<Edge>,
    incoming: Vec<Option<usize>>,
    verifications: Option<Box<verify::Verifications>>,
    members: Option<Box<members::Members>>,
    limit: usize,
}
pub struct Solution {
    pub regions: usize,
    pub obligations: usize,
    pub propagated: usize,
    pub steps: usize,
    graph: Graph,
}
/// A previously proved directed requirement: `longer` contains the queried
/// shorter region. Source indexes still name the original typed obligation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub longer: Region,
    pub source: u32,
}
impl Solution {
    /// Lifetimes required to contain static by this successfully solved graph.
    /// This conservative query follows retained obligations, not new assumptions.
    /// # Errors
    /// Rejects exhausted traversal work or invalid retained identities.
    pub fn required_static(&self) -> Result<Vec<Region>, Error> {
        if self.graph.incoming[0].is_none() {
            return Ok(Vec::new());
        }
        let mut seen = HashSet::from_iter([Region::STATIC]);
        let mut work = vec![Region::STATIC];
        let mut steps = 0_usize;
        while let Some(shorter) = work.pop() {
            for requirement in self.requirements(shorter)? {
                steps = steps.checked_add(1).ok_or(Error::Budget)?;
                if steps > self.graph.limit {
                    return Err(Error::Budget);
                }
                if seen.insert(requirement.longer) {
                    work.push(requirement.longer);
                }
            }
        }
        seen.remove(&Region::STATIC);
        let mut result: Vec<_> = seen.into_iter().collect();
        result.sort_unstable_by_key(|region| region.0);
        Ok(result)
    }
    /// Graph construction assigns compact identities in insertion order.
    pub fn nodes(&self) -> impl Iterator<Item = (Region, Kind)> + '_ {
        (0_u32..)
            .zip(&self.graph.kinds)
            .map(|(id, &kind)| (Region(id), kind))
    }
    #[must_use]
    pub fn assumption_count(&self) -> usize {
        self.graph.assumptions.len()
    }
    #[must_use]
    pub fn kind(&self, region: Region) -> Option<Kind> {
        self.graph.kinds.get(region.0 as usize).copied()
    }
    /// Iterate retained requirements without rebuilding or solving the graph.
    ///
    /// # Errors
    /// Rejects an identity outside this body's proven graph.
    pub fn requirements(
        &self,
        shorter: Region,
    ) -> Result<impl Iterator<Item = Requirement> + '_, Error> {
        let mut edge = *self
            .graph
            .incoming
            .get(shorter.0 as usize)
            .ok_or(Error::Identity)?;
        Ok(std::iter::from_fn(move || {
            let relation = self.graph.obligations.get(edge?)?;
            edge = relation.next;
            Some(Requirement {
                longer: relation.target,
                source: relation.source,
            })
        }))
    }
    /// Declared assumptions remain distinct from requirements; downstream passes
    /// may use the proven environment but cannot strengthen it with obligations.
    ///
    /// # Errors
    /// Rejects an identity outside this body's proven graph.
    pub fn assumptions(&self, longer: Region) -> Result<impl Iterator<Item = Region> + '_, Error> {
        let mut edge = *self
            .graph
            .assumed_heads
            .get(longer.0 as usize)
            .ok_or(Error::Identity)?;
        Ok(std::iter::from_fn(move || {
            let relation = self.graph.assumptions.get(edge?)?;
            edge = relation.next;
            Some(relation.target)
        }))
    }
}
impl Graph {
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            kinds: vec![Kind::Static, Kind::Local],
            assumptions: Vec::new(),
            assumed_heads: vec![None; 2],
            obligations: Vec::new(),
            incoming: vec![None; 2],
            verifications: None,
            members: None,
            limit,
        }
    }
    /// # Errors
    /// Rejects arena overflow or the body constraint budget.
    pub fn region(&mut self, kind: Kind) -> Result<Region, Error> {
        if matches!(kind, Kind::Static | Kind::Local) {
            return Err(Error::Identity);
        }
        self.capacity(1)?;
        let id = Region(u32::try_from(self.kinds.len()).map_err(|_| Error::Budget)?);
        self.kinds.push(kind);
        self.assumed_heads.push(None);
        self.incoming.push(None);
        Ok(id)
    }
    fn capacity(&self, extra: usize) -> Result<(), Error> {
        if self
            .kinds
            .len()
            .saturating_add(self.assumptions.len())
            .saturating_add(self.obligations.len())
            .saturating_add(self.verifications.as_ref().map_or(0, |v| v.cost))
            .saturating_add(self.members.as_ref().map_or(0, |m| m.cost))
            .saturating_add(extra)
            > self.limit
        {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn kind(&self, region: Region) -> Result<Kind, Error> {
        self.kinds
            .get(region.0 as usize)
            .copied()
            .ok_or(Error::Identity)
    }
    /// Declared or implied well-formedness bounds may relate only universals.
    /// They are immutable while obligations are solved.
    ///
    /// # Errors
    /// Rejects invalid/nonuniversal identities and exhausted constraint storage.
    pub fn assume(&mut self, longer: Region, shorter: Region) -> Result<(), Error> {
        let universal = |kind| matches!(kind, Kind::Static | Kind::Universal { .. });
        if !universal(self.kind(longer)?) || !universal(self.kind(shorter)?) {
            return Err(Error::Identity);
        }
        self.capacity(1)?;
        self.assumptions.push(Edge {
            target: shorter,
            source: 0,
            next: self.assumed_heads[longer.0 as usize],
        });
        self.assumed_heads[longer.0 as usize] = Some(self.assumptions.len() - 1);
        Ok(())
    }
    /// # Errors
    /// Rejects invalid identities and exhausted constraint storage.
    pub fn require(&mut self, longer: Region, shorter: Region, source: u32) -> Result<(), Error> {
        self.kind(longer)?;
        self.kind(shorter)?;
        if longer == shorter || longer == Region::STATIC {
            return Ok(());
        }
        self.capacity(1)?;
        self.obligations.push(Edge {
            target: longer,
            source,
            next: self.incoming[shorter.0 as usize],
        });
        self.incoming[shorter.0 as usize] = Some(self.obligations.len() - 1);
        Ok(())
    }
    /// Propagate universal lower bounds backwards through obligations. An outer
    /// variable cannot name a younger placeholder: its required lower bound is
    /// promoted to static. This permits valid shortening but rejects binder leaks.
    ///
    /// # Errors
    /// Reports an unentailed universal relationship or bounded-resource failure.
    pub fn solve(self) -> Result<Solution, Error> {
        self.solve_charged(0)
    }
    fn solve_charged(mut self, prior: usize) -> Result<Solution, Error> {
        let member_steps = self.select_members()?;
        let mut solver = Solver::new(&self);
        solver.charge_many(prior)?;
        solver.charge_many(member_steps)?;
        solver.propagate(|longer, shorter, source| {
            Err(Error::Unsatisfied {
                source,
                longer,
                shorter,
            })
        })?;
        let verified = solver.verify()?;
        let propagated = solver.reached.len();
        let steps = solver.steps;
        if let Some(verifications) = &mut self.verifications {
            verifications.verified = verified;
        }
        Ok(Solution {
            regions: self.kinds.len(),
            obligations: self.obligations.len(),
            propagated,
            steps,
            graph: self,
        })
    }
}
struct Solver<'a> {
    graph: &'a Graph,
    steps: usize,
    reached: HashSet<(Region, Region)>,
    work: Vec<(Region, Region)>,
    assumed: HashMap<(Region, Region), bool>,
    marks: Vec<u32>,
    epoch: u32,
    search: Vec<Region>,
}
impl Solver<'_> {
    fn charge(&mut self) -> Result<(), Error> {
        self.charge_many(1)
    }
    fn charge_many(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.checked_add(count).ok_or(Error::Budget)?;
        if self.steps > self.graph.limit {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn insert(&mut self, region: Region, bound: Region) -> Result<(), Error> {
        self.charge()?;
        if self.reached.insert((region, bound)) {
            self.work.push((region, bound));
        }
        Ok(())
    }
    fn entails(&mut self, longer: Region, shorter: Region) -> Result<bool, Error> {
        if longer == shorter || longer == Region::STATIC {
            return Ok(true);
        }
        if shorter == Region::LOCAL && self.graph.kind(longer)?.universe() == 0 {
            return Ok(true);
        }
        if let Some(result) = self.assumed.get(&(longer, shorter)) {
            return Ok(*result);
        }
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.marks.fill(0);
            self.epoch = 1;
        }
        self.search.clear();
        self.search.push(longer);
        self.marks[longer.0 as usize] = self.epoch;
        let mut result = false;
        while let Some(region) = self.search.pop() {
            self.charge()?;
            if region == shorter || region == Region::STATIC {
                result = true;
                break;
            }
            let mut edge = self.graph.assumed_heads[region.0 as usize];
            while let Some(index) = edge {
                self.charge()?;
                let relation = self.graph.assumptions[index];
                edge = relation.next;
                if self.marks[relation.target.0 as usize] != self.epoch {
                    self.marks[relation.target.0 as usize] = self.epoch;
                    self.search.push(relation.target);
                }
            }
        }
        self.assumed.insert((longer, shorter), result);
        Ok(result)
    }
}

impl<'a> Solver<'a> {
    fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            steps: 0,
            reached: HashSet::default(),
            work: Vec::new(),
            assumed: HashMap::default(),
            marks: vec![0; graph.kinds.len()],
            epoch: 0,
            search: Vec::new(),
        }
    }

    /// Stop propagation at rigid frontiers. Their own seeds already reach every
    /// predecessor; following their descendants too would duplicate that work.
    fn propagate(
        &mut self,
        mut unmet: impl FnMut(Region, Region, u32) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.seed_rigids()?;
        self.propagate_work(&mut unmet, |_, _| {})
    }

    fn seed_rigids(&mut self) -> Result<(), Error> {
        for (index, kind) in self.graph.kinds.iter().enumerate() {
            if kind.rigid() {
                let id = Region(u32::try_from(index).map_err(|_| Error::Budget)?);
                self.insert(id, id)?;
            }
        }
        Ok(())
    }

    fn propagate_work(
        &mut self,
        mut unmet: impl FnMut(Region, Region, u32) -> Result<(), Error>,
        mut observed: impl FnMut(Region, Region),
    ) -> Result<(), Error> {
        while let Some((shorter, bound)) = self.work.pop() {
            observed(shorter, bound);
            let mut edge = self.graph.incoming[shorter.0 as usize];
            while let Some(index) = edge {
                self.charge()?;
                let relation = self.graph.obligations[index];
                edge = relation.next;
                let longer = relation.target;
                let kind = self.graph.kind(longer)?;
                let bound = if kind.universe() < self.graph.kind(bound)?.universe() {
                    Region::STATIC
                } else {
                    bound
                };
                if kind.rigid() {
                    if !self.entails(longer, bound)? {
                        unmet(longer, bound, relation.source)?;
                    }
                } else {
                    self.insert(longer, bound)?;
                }
            }
        }
        Ok(())
    }
}
