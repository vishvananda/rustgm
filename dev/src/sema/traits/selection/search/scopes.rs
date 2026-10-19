//! Sparse universe metadata preserves selected graph debts without copying ancestor proofs.
use super::{Decision, Declarations, Evidence, Node, Predicate, Query};
use crate::types::{
    RegionId,
    lifetime_proof::{Obligations, ScopeId},
    substitute::Bound,
};
use std::{collections::HashMap, ops::Range};

struct Universe {
    parent: ScopeId,
    count: u32,
    aliases: Range<u32>,
    opening: Bound,
    proof: Obligations,
    selected: bool,
}
struct Local {
    scope: ScopeId,
    regions: Range<u32>,
    nested: bool,
}
struct Reservation {
    scope: ScopeId,
    regions: Range<u32>,
    alias: bool,
    selected: bool,
}
#[derive(Default)]
pub(super) struct Scopes {
    universes: Vec<Universe>,
    nodes: HashMap<usize, Local>,
    completed: HashMap<(ScopeId, Predicate), Decision>,
    reservations: Vec<Reservation>,
    by_start: HashMap<u32, usize>,
}
impl Query {
    pub(super) fn reserve(
        &mut self,
        scope: ScopeId,
        regions: Range<u32>,
    ) -> Result<(), &'static str> {
        if let Some(scopes) = &mut self.scopes {
            scopes.reserve(scope, regions.clone(), false)?;
        }
        if scope != ScopeId::ROOT
            && !regions.is_empty()
            && let Some(universal) = &mut self.universal
        {
            universal
                .sources
                .push(crate::types::substitute::ExportSource::Private(regions));
        }
        Ok(())
    }
    pub(super) fn push(&mut self, node: Node, scope: ScopeId, regions: Range<u32>) -> usize {
        let index = self.nodes.len();
        if let Some(scopes) = &mut self.scopes {
            let nested = node.children.iter().any(|child| {
                scopes
                    .nodes
                    .get(child)
                    .is_some_and(|child| child.scope != scope || child.nested)
            });
            if nested || scope != ScopeId::ROOT {
                scopes.nodes.insert(
                    index,
                    Local {
                        scope,
                        regions,
                        nested,
                    },
                );
            }
        }
        self.nodes.push(node);
        index
    }
    pub(super) fn node_scope(&self, node: usize) -> ScopeId {
        self.scopes
            .as_ref()
            .and_then(|scopes| scopes.nodes.get(&node))
            .map_or(ScopeId::ROOT, |node| node.scope)
    }
    pub(super) fn cacheable(&self, node: usize) -> bool {
        self.scopes
            .as_ref()
            .and_then(|scopes| scopes.nodes.get(&node))
            .is_none_or(|node| !node.nested)
    }
    pub(super) fn completed(&self, scope: ScopeId, goal: Predicate) -> Option<Decision> {
        if scope == ScopeId::ROOT {
            self.completed.get(&goal).copied()
        } else {
            self.scopes.as_ref()?.completed.get(&(scope, goal)).copied()
        }
    }
    pub(super) fn remember(&mut self, scope: ScopeId, goal: Predicate, decision: Decision) {
        if scope == ScopeId::ROOT {
            self.completed.insert(goal, decision);
        } else if let Some(scopes) = &mut self.scopes {
            scopes.completed.insert((scope, goal), decision);
        }
    }
}
impl Scopes {
    pub(super) fn open_normalization_application(
        &mut self,
        facts: &mut Declarations,
        scope: ScopeId,
        application: crate::types::projection::Application,
    ) -> Result<(crate::types::projection::Application, usize, ScopeId), &'static str> {
        use crate::types::parameters::predicates::Bound as TraitBound;
        let universe = scope
            .0
            .checked_sub(1)
            .and_then(|id| self.universes.get_mut(id as usize))
            .ok_or("missing normalization universe")?;
        let before = universe.opening.steps;
        let predicate = universe.opening.predicate(
            &mut facts.types,
            Predicate {
                ty: application.subject,
                bound: TraitBound::Trait(application.trait_application),
            },
        )?;
        let TraitBound::Trait(trait_application) = predicate.bound else {
            return Err("normalization application lost its parent trait");
        };
        let arguments = universe
            .opening
            .apply_arguments(&mut facts.types, application.arguments)?;
        Ok((
            crate::types::projection::Application {
                subject: predicate.ty,
                trait_application,
                arguments,
                ..application
            },
            universe.opening.steps.saturating_sub(before) + 1,
            universe.parent,
        ))
    }
    pub(super) fn open_type(
        &mut self,
        facts: &mut Declarations,
        mut scope: ScopeId,
        parent: ScopeId,
        mut ty: crate::types::TypeId,
    ) -> Result<(crate::types::TypeId, usize), &'static str> {
        let mut steps = 0;
        while scope != parent {
            let universe = scope
                .0
                .checked_sub(1)
                .and_then(|id| self.universes.get_mut(id as usize))
                .ok_or("normalization scope is not a descendant of its caller")?;
            let before = universe.opening.steps;
            ty = universe.opening.apply(&mut facts.types, ty)?;
            steps += universe.opening.steps.saturating_sub(before) + 1;
            scope = universe.parent;
        }
        Ok((ty, steps))
    }

    pub(super) fn closing(
        &self,
        mut scope: ScopeId,
        parent: ScopeId,
        caller_end: u32,
    ) -> Result<(Bound, usize), &'static str> {
        let mut depth = 0_u16;
        let mut anchors = HashMap::new();
        let mut steps = 0;
        while scope != parent {
            let universe = scope
                .0
                .checked_sub(1)
                .and_then(|id| self.universes.get(id as usize))
                .ok_or("projection closure scope is not a descendant of its caller")?;
            for (slot, region) in universe.aliases.clone().enumerate() {
                anchors.insert(
                    RegionId(region),
                    RegionId::bound(
                        depth,
                        u16::try_from(slot).map_err(|_| "projection closure slot overflow")?,
                    )?,
                );
                steps += 1;
            }
            depth = depth
                .checked_add(1)
                .ok_or("projection closure depth overflow")?;
            steps += 1;
            scope = universe.parent;
        }
        // Allocation order does not make an earlier sibling visible. Only
        // actual ancestors supplement the frame's original caller frontier.
        while scope != ScopeId::ROOT {
            let universe = self
                .universes
                .get(scope.0 as usize - 1)
                .ok_or("missing projection closure ancestor")?;
            for region in universe.aliases.clone() {
                anchors.insert(RegionId(region), RegionId(region));
                steps += 1;
            }
            steps += 1;
            scope = universe.parent;
        }
        Ok((
            Bound::close_projection_regions(caller_end, depth, anchors)?,
            steps,
        ))
    }

    /// Give bound equations a query-wide identity before indexing proof nodes
    /// from different scopes. Captures outside the query remain caller-bound.
    pub(super) fn equation_region(
        &self,
        mut scope: ScopeId,
        region: RegionId,
    ) -> Result<(RegionId, usize), &'static str> {
        let Some((mut depth, slot)) = region.bound_parts() else {
            return Ok((region, 1));
        };
        let mut steps = 1;
        while scope != ScopeId::ROOT {
            let universe = self
                .universes
                .get(scope.0 as usize - 1)
                .ok_or("missing selected equation scope")?;
            steps += 1;
            if depth == 0 {
                if u32::from(slot) >= universe.count {
                    return Err("selected equation binder slot overflow");
                }
                return Ok((RegionId(universe.aliases.start + u32::from(slot)), steps));
            }
            depth -= 1;
            scope = universe.parent;
        }
        Ok((RegionId::bound(depth, slot)?, steps))
    }

    pub fn open_inferred(
        &mut self,
        facts: &mut Declarations,
        frame: &mut super::Frame,
        scope: ScopeId,
    ) -> Result<(), String> {
        frame.scope = scope;
        let mut next = scope;
        let mut aliases = Vec::new();
        while next != ScopeId::ROOT {
            let universe = self
                .universes
                .get(next.0 as usize - 1)
                .ok_or("missing inferred universal scope")?;
            aliases.push(universe.aliases.clone());
            next = universe.parent;
        }
        let mut opening = Bound::open_scopes(aliases)?;
        frame
            .inputs
            .context
            .account_work(opening.steps)
            .map_err(super::inference)?;
        frame.goal = frame
            .inputs
            .context
            .map_predicate_regions(&mut facts.types, &frame.goal, &mut opening)
            .map_err(super::inference)?;
        Ok(())
    }

    pub fn bridge(&mut self, private: RegionId, caller: RegionId) -> Result<usize, &'static str> {
        let mut work = 1;
        let index = self.reservations.partition_point(|reservation| {
            work += 1;
            reservation.regions.end <= private.0
        });
        let owner = self
            .reservations
            .get(index)
            .filter(|reservation| !reservation.alias && reservation.regions.contains(&private.0))
            .ok_or("exported witness has no private proof owner")?;
        self.universes
            .get_mut(owner.scope.0 as usize - 1)
            .ok_or("missing private witness scope")?
            .proof
            .equalities
            .push((private, caller));
        Ok(work)
    }

    pub fn equalities(
        &mut self,
        scope: ScopeId,
        mut equalities: Vec<crate::types::outlives::Pair>,
    ) -> Result<(), &'static str> {
        let index = scope
            .0
            .checked_sub(1)
            .ok_or("universal evidence has no scope")?;
        self.universes
            .get_mut(index as usize)
            .ok_or("missing universal evidence scope")?
            .proof
            .equalities
            .append(&mut equalities);
        Ok(())
    }

    fn reserve(
        &mut self,
        scope: ScopeId,
        regions: Range<u32>,
        alias: bool,
    ) -> Result<(), &'static str> {
        if scope == ScopeId::ROOT || regions.is_empty() {
            return Ok(());
        }
        if scope.0 as usize > self.universes.len()
            || regions.start > regions.end
            || self
                .reservations
                .last()
                .is_some_and(|previous| previous.regions.end > regions.start)
        {
            return Err("invalid quantified impl allocation order");
        }
        self.by_start.insert(regions.start, self.reservations.len());
        self.reservations.push(Reservation {
            scope,
            regions,
            alias,
            selected: false,
        });
        Ok(())
    }
    pub fn introduce(
        &mut self,
        parent: ScopeId,
        count: u32,
        first: u32,
    ) -> Result<ScopeId, &'static str> {
        if parent.0 as usize > self.universes.len()
            || self.universes.len() >= 1_000_000
            || count > 65_536
        {
            return Err("quantified impl scope budget exhausted");
        }
        let end = first
            .checked_add(count)
            .filter(|end| *end < 0x8000_0000)
            .ok_or("quantified impl region identity space exhausted")?;
        self.universes.push(Universe {
            parent,
            count,
            aliases: first..end,
            opening: Bound::new((first..end).map(RegionId).collect()),
            proof: Obligations::new(0),
            selected: false,
        });
        let scope =
            ScopeId(u32::try_from(self.universes.len()).map_err(|_| "too many impl scopes")?);
        self.reserve(scope, first..end, true)?;
        Ok(scope)
    }
    pub fn open(
        &mut self,
        facts: &mut Declarations,
        scope: ScopeId,
        predicate: Predicate,
        first: u32,
    ) -> Result<(Predicate, u32, usize), &'static str> {
        let universe = self
            .universes
            .get_mut(scope.0 as usize - 1)
            .ok_or("missing impl universe")?;
        let before = universe.opening.steps;
        let predicate = universe.opening.predicate(&mut facts.types, predicate)?;
        Ok((
            predicate,
            first.max(universe.aliases.end),
            universe
                .opening
                .steps
                .saturating_sub(before)
                .saturating_add(1),
        ))
    }
    pub fn collect(
        &mut self,
        index: usize,
        node: &mut Node,
        evidence: &mut Evidence,
    ) -> Result<(), &'static str> {
        let local = self
            .nodes
            .get(&index)
            .filter(|local| local.scope != ScopeId::ROOT);
        let proof = if let Some(local) = local {
            if !node.sized.is_empty() {
                return Err("quantified impl proof retained unscoped sized requirements");
            }
            let universe = self
                .universes
                .get_mut(local.scope.0 as usize - 1)
                .ok_or("missing selected impl universe")?;
            universe.selected = true;
            if !local.regions.is_empty() {
                let position = self
                    .by_start
                    .get(&local.regions.start)
                    .copied()
                    .ok_or("missing selected impl allocation")?;
                let reservation = &mut self.reservations[position];
                if reservation.scope != local.scope
                    || reservation.regions != local.regions
                    || reservation.alias
                {
                    return Err("selected impl allocation changed ownership");
                }
                reservation.selected = true;
            }
            &mut universe.proof
        } else {
            evidence.sized.append(&mut node.sized);
            &mut evidence.proof
        };
        proof.equalities.append(&mut node.equalities);
        proof.outlives.append(&mut node.outlives);
        proof.types.append(&mut node.type_outlives);
        Ok(())
    }
    pub fn finish(mut self, proof: &mut Obligations) -> Result<(), &'static str> {
        for index in (0..self.universes.len()).rev() {
            let universe = &self.universes[index];
            if universe.selected && universe.parent != ScopeId::ROOT {
                let parent = universe.parent.0 as usize - 1;
                self.universes[parent].selected = true;
            }
        }
        let mut bindings = vec![ScopeId::ROOT];
        for universe in &mut self.universes {
            let scope = if universe.selected {
                let scope = proof.scope(bindings[universe.parent.0 as usize], universe.count)?;
                for (slot, region) in universe.aliases.clone().enumerate() {
                    universe.proof.equalities.push((
                        RegionId(region),
                        RegionId::bound(
                            0,
                            u16::try_from(slot)
                                .map_err(|_| "too many quantified impl lifetimes")?,
                        )?,
                    ));
                }
                scope
            } else {
                ScopeId::ROOT
            };
            bindings.push(scope);
        }
        for reservation in self.reservations {
            if !(reservation.selected
                || reservation.alias && self.universes[reservation.scope.0 as usize - 1].selected)
            {
                continue;
            }
            let mut allocation = Obligations::new(reservation.regions.start);
            allocation.regions.end = reservation.regions.end;
            proof.place_at(allocation, bindings[reservation.scope.0 as usize])?;
        }
        for (index, mut universe) in self.universes.into_iter().enumerate() {
            if universe.selected {
                universe.proof.regions = proof.regions.end..proof.regions.end;
                proof.place_at(universe.proof, bindings[index + 1])?;
            }
        }
        Ok(())
    }
}
