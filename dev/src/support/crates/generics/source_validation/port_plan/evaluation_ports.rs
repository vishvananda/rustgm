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
    access: Vec<Access>,
}
/// A projection's write access. `writable` admits a write or a mutable borrow
/// of the place itself. `unique` holds when every dereference on its path is
/// mutable: rustc then admits a write through a mutable dereference of it even
/// where its root local is immutable.
#[derive(Clone, Copy, Default)]
struct Access {
    writable: bool,
    unique: bool,
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
            .saturating_add(projections.saturating_mul(size_of::<Access>()));
        arena.held_scratch(bytes, work)?;
        work.charge(values.saturating_mul(2).saturating_add(projections))?;
        let mut ports = Self {
            evaluated: vec![None; values],
            requested: vec![None; values],
            access: vec![Access::default(); projections],
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
            let (raw_region, raw_mutable) = match projection.kind {
                s::ProjectionKind::Field { owner, ordinal } => {
                    builder.nominal_projection(
                        original,
                        Self::place_type(original, projection.base, work)?,
                        projection.ty,
                        owner,
                        ordinal,
                        work,
                    )?;
                    // Version thirteen reads a field of a temporary, such as
                    // `make().n`; the field is never writable, so a write
                    // through it still requires wider proof.
                    let writable = match projection.base {
                        s::Place::Temporary(_) if builder.domains.bodies => false,
                        base => self.writable(original, base, work)?,
                    };
                    self.access[index] = Access {
                        writable,
                        unique: self.unique(projection.base),
                    };
                    continue;
                }
                s::ProjectionKind::Dereference { region, mutable } => (region, mutable),
            };
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
                        raw_region,
                        super::nodes::Rewrite::Original,
                        work,
                    )?
                || mutable != raw_mutable
            {
                return Err("source dereference changed its referent, lifetime or mutability");
            }
            // Version thirteen follows rustc: a mutable dereference is writable
            // through a unique path even from an immutable owning local. Older
            // profiles keep requiring a writable parent projection.
            let path = match projection.base {
                s::Place::Projection(parent) if !builder.domains.bodies => {
                    self.access[parent as usize].writable
                }
                base => self.unique(base),
            };
            self.access[index] = Access {
                writable: mutable && path,
                unique: mutable && path,
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
            s::Place::Temporary(index) => original.values.get(index as usize).map(|value| value.ty),
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
            s::Place::Projection(index) => self
                .access
                .get(index as usize)
                .map(|access| access.writable),
            s::Place::Temporary(_) => {
                return Err("source direct temporary writes require wider proof");
            }
        }
        .ok_or("source writable place is foreign to its original body")
    }
    /// Locals and temporaries own their values uniquely; a projection's path
    /// is recorded before any projection that names it.
    fn unique(&self, place: s::Place) -> bool {
        match place {
            s::Place::Local(_) | s::Place::Temporary(_) => true,
            s::Place::Projection(parent) => self
                .access
                .get(parent as usize)
                .is_some_and(|access| access.unique),
        }
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
