//! One body-local port ledger; source-site equality never merges Call values.
use super::{
    Work, g,
    nodes::{Arena, Node},
    requests::Builder,
    s,
};

pub(super) struct Ports {
    pub evaluated: Vec<Option<u32>>,
    pub requested: Vec<Option<u32>>,
    writable: Vec<bool>,
}
impl Ports {
    pub(super) fn new(
        original: &s::Original,
        arena: &mut Arena,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let values = original.values.len();
        let projections = original.projections.len();
        let bytes = size_of::<Self>()
            .saturating_add(values.saturating_mul(2 * size_of::<Option<u32>>()))
            .saturating_add(projections);
        arena.held_scratch(bytes, work)?;
        work.charge(values.saturating_mul(2).saturating_add(projections))?;
        let mut ports = Self {
            evaluated: vec![None; values],
            requested: vec![None; values],
            writable: vec![false; projections],
        };
        for statement in &original.statements {
            work.charge(1)?;
            if let s::Action::Evaluate(value) = statement.action {
                let site = ports
                    .evaluated
                    .get_mut(value as usize)
                    .ok_or("source evaluation has no actual value")?;
                if site.replace(statement.site).is_some() {
                    return Err("source value has duplicate evaluation ownership");
                }
            }
        }
        for (ordinal, request) in original.calls.iter().enumerate() {
            work.charge(1)?;
            let slot = ports
                .requested
                .get_mut(request.value as usize)
                .ok_or("source Call request has no actual value")?;
            if slot
                .replace(u32::try_from(ordinal).map_err(|_| "source request ordinal overflow")?)
                .is_some()
            {
                return Err("source value has duplicate actual Call requests");
            }
        }
        Ok(ports)
    }
    pub(super) fn projections(
        &mut self,
        builder: &mut Builder<'_, '_, '_>,
        original: &s::Original,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        for (index, projection) in original.projections.iter().enumerate() {
            work.charge(1)?;
            let base = builder.node(
                original,
                Self::place_type(original, projection.base, work)?,
                work,
            )?;
            let Node::Reference {
                mutable,
                region,
                referent,
            } = builder.arena.nodes[base.0 as usize]
            else {
                return Err("source dereference base is not an actual reference");
            };
            if referent != builder.node(original, projection.ty, work)?
                || region
                    != super::uses::region(
                        builder.domains,
                        &builder.arena,
                        projection.region,
                        super::nodes::Rewrite::Original,
                        work,
                    )?
                || mutable != projection.mutable
            {
                return Err("source dereference changed its referent, lifetime or mutability");
            }
            self.writable[index] = mutable
                && match projection.base {
                    s::Place::Local(_) => true,
                    s::Place::Projection(parent) => self.writable[parent as usize],
                };
        }
        Ok(())
    }
    pub(super) fn place_type(
        original: &s::Original,
        place: s::Place,
        work: &mut Work,
    ) -> Result<g::TypeRef, &'static str> {
        work.charge(1)?;
        match place {
            s::Place::Local(index) => original.locals.get(index as usize).map(|local| local.ty),
            s::Place::Projection(index) => original
                .projections
                .get(index as usize)
                .map(|projection| projection.ty),
        }
        .ok_or("source typed place is foreign to its original body")
    }
    pub(super) fn writable(
        &self,
        original: &s::Original,
        place: s::Place,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(1)?;
        match place {
            s::Place::Local(index) => original
                .locals
                .get(index as usize)
                .map(|local| local.mutable),
            s::Place::Projection(index) => self.writable.get(index as usize).copied(),
        }
        .ok_or("source writable place is foreign to its original body")
    }
    pub(super) fn site(&self, value: u32, work: &mut Work) -> Result<u32, &'static str> {
        work.charge(1)?;
        self.evaluated
            .get(value as usize)
            .copied()
            .flatten()
            .ok_or("source typed value omitted its actual evaluation site")
    }
}
