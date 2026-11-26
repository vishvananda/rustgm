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
            self.projection_row(builder, original, index, projection, work)?;
        }
        Ok(())
    }
    fn projection_row(
        &mut self,
        builder: &mut Builder<'_, '_, '_>,
        original: &s::Original,
        index: usize,
        projection: &s::Projection,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let base = builder.node(
            original,
            Self::place_type(builder.domains.headers, original, projection.base, work)?,
            work,
        )?;
        let (raw_region, raw_mutable) = match projection.kind {
            s::ProjectionKind::CaptureBoundary { mutable } => {
                let actual = builder.node(original, projection.ty, work)?;
                if actual != base {
                    return Err("source capture boundary changed its projected type");
                }
                let sub = self.subplace_access(builder, original, projection.base, work)?;
                self.access[index] = Access {
                    writable: mutable && sub.writable,
                    unique: sub.unique,
                };
                return Ok(());
            }
            s::ProjectionKind::Field { owner, ordinal } => {
                builder.nominal_projection(
                    original,
                    Self::place_type(builder.domains.headers, original, projection.base, work)?,
                    projection.ty,
                    owner,
                    ordinal,
                    work,
                )?;
                self.access[index] =
                    self.subplace_access(builder, original, projection.base, work)?;
                return Ok(());
            }
            s::ProjectionKind::UnionField { owner, ordinal } => {
                builder.union_projection(
                    original,
                    Self::place_type(builder.domains.headers, original, projection.base, work)?,
                    projection.ty,
                    owner,
                    ordinal,
                    work,
                )?;
                self.access[index] =
                    self.subplace_access(builder, original, projection.base, work)?;
                return Ok(());
            }
            s::ProjectionKind::Tuple(ordinal) => {
                Self::tuple_projection(builder, original, base, projection.ty, ordinal, work)?;
                self.access[index] =
                    self.subplace_access(builder, original, projection.base, work)?;
                return Ok(());
            }
            s::ProjectionKind::RawDereference {
                mutable: raw_mutable,
            } => {
                self.access[index] = Self::raw_deref_access(
                    builder,
                    original,
                    base,
                    projection.ty,
                    raw_mutable,
                    work,
                )?;
                return Ok(());
            }
            s::ProjectionKind::Index { .. }
            | s::ProjectionKind::IndexParameter { .. }
            | s::ProjectionKind::SliceIndex { .. }
            | s::ProjectionKind::ConstantIndex(_)
            | s::ProjectionKind::FromEnd { .. }
            | s::ProjectionKind::Subslice { .. }
            | s::ProjectionKind::Subarray { .. } => {
                Self::sequence_projection(builder, original, base, projection, work)?;
                self.access[index] =
                    self.subplace_access(builder, original, projection.base, work)?;
                return Ok(());
            }
            s::ProjectionKind::Dereference { region, mutable } => (region, mutable),
        };
        self.access[index] = self.deref_access(
            builder,
            original,
            base,
            projection,
            (raw_region, raw_mutable),
            work,
        )?;
        Ok(())
    }
    fn tuple_projection(
        builder: &mut Builder<'_, '_, '_>,
        original: &s::Original,
        base: super::nodes::NodeId,
        ty: g::TypeRef,
        ordinal: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let elements = match builder.arena.nodes[base.0 as usize] {
            Node::Tuple { elements } => elements,
            Node::Closure { environment, .. } if builder.domains.declaration_policies => {
                let Node::Tuple { elements } = builder.arena.nodes[environment.0 as usize] else {
                    return Err("source closure capture projection base has no capture tuple");
                };
                elements
            }
            _ => return Err("source tuple projection base is not an actual tuple"),
        };
        let expected = builder.arena.nominal_argument(elements, ordinal, work)?;
        let actual = builder.node(original, ty, work)?;
        if actual != expected {
            return Err("source tuple projection differs from its ordered element");
        }
        Ok(())
    }
    fn deref_access(
        &self,
        builder: &mut Builder<'_, '_, '_>,
        original: &s::Original,
        base: super::nodes::NodeId,
        projection: &s::Projection,
        (raw_region, raw_mutable): (s::RegionRef, bool),
        work: &mut Work,
    ) -> Result<Access, &'static str> {
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
        let path = match projection.base {
            s::Place::Projection(parent) if !builder.domains.bodies => {
                self.access[parent as usize].writable
            }
            base => self.unique(base),
        };
        Ok(Access {
            writable: mutable && path,
            unique: mutable && path,
        })
    }
    pub(super) fn place_type(
        headers: &crate::support::crates::generics::source::HeaderIndex<'_>,
        original: &s::Original,
        place: s::Place,
        work: &mut Work,
    ) -> Result<g::TypeRef, &'static str> {
        work.charge(1)?;
        match place {
            s::Place::Static(declaration) => {
                return headers
                    .static_header(declaration, work)
                    .map(|header| header.ty);
            }
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
        allow_unnamed_storage: bool,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(1)?;
        match place {
            s::Place::Static(_) => Some(false),
            s::Place::Local(index) => original.locals.get(index as usize).map(|local| {
                // An unnamed parameter owns the input storage for its checked pattern.
                local.mutable
                    || allow_unnamed_storage
                        && local.definition.is_none()
                        && matches!(local.kind, g::LocalKind::Parameter(_))
            }),
            s::Place::Projection(index) => self
                .access
                .get(index as usize)
                .map(|access| access.writable),
            s::Place::Temporary(index) if allow_unnamed_storage => {
                ((index as usize) < original.values.len()).then_some(true)
            }
            s::Place::Temporary(_) => {
                return Err("source direct temporary writes require wider proof");
            }
        }
        .ok_or("source writable place is foreign to its original body")
    }
    /// Locals and temporaries own their values uniquely; a projection's path
    /// is recorded before any projection that names it.
    pub(super) fn unique(&self, place: s::Place) -> bool {
        match place {
            s::Place::Local(_) | s::Place::Temporary(_) => true,
            s::Place::Static(_) => false,
            s::Place::Projection(parent) => self
                .access
                .get(parent as usize)
                .is_some_and(|access| access.unique),
        }
    }
    fn raw_deref_access(
        builder: &mut Builder<'_, '_, '_>,
        original: &s::Original,
        base: super::nodes::NodeId,
        ty: g::TypeRef,
        raw_mutable: bool,
        work: &mut Work,
    ) -> Result<Access, &'static str> {
        let raw_base = match builder.arena.nodes[base.0 as usize] {
            Node::Pattern { base, .. } => builder.arena.nodes[base.0 as usize],
            node => node,
        };
        let Node::RawPointer { mutable, pointee } = raw_base else {
            return Err("source raw dereference base is not an actual raw pointer");
        };
        if mutable != raw_mutable || pointee != builder.node(original, ty, work)? {
            return Err("source raw dereference changed its pointee or mutability");
        }
        Ok(Access {
            writable: mutable,
            unique: mutable,
        })
    }
    fn subplace_access(
        &self,
        builder: &Builder<'_, '_, '_>,
        original: &s::Original,
        base: s::Place,
        work: &mut Work,
    ) -> Result<Access, &'static str> {
        let writable = match base {
            s::Place::Temporary(_) if builder.domains.declaration_policies => true,
            s::Place::Temporary(_) if builder.domains.bodies => false,
            place => self.writable(original, place, builder.domains.declaration_policies, work)?,
        };
        Ok(Access {
            writable,
            unique: self.unique(base),
        })
    }
    fn sequence_projection(
        builder: &mut Builder<'_, '_, '_>,
        original: &s::Original,
        base: super::nodes::NodeId,
        projection: &s::Projection,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let actual = builder.node(original, projection.ty, work)?;
        let base_node = builder.arena.nodes[base.0 as usize];
        let u64_const =
            |arena: &Arena, node: super::nodes::NodeId| match arena.nodes[node.0 as usize] {
                Node::ConstValue {
                    scalar: crate::support::crates::Scalar::Usize,
                    bits,
                } => {
                    let value = u128::from(bits[0])
                        | (u128::from(bits[1]) << 32)
                        | (u128::from(bits[2]) << 64)
                        | (u128::from(bits[3]) << 96);
                    u64::try_from(value).ok()
                }
                _ => None,
            };
        match projection.kind {
            s::ProjectionKind::Index { length, .. } => {
                let Node::Array { element, length: l } = base_node else {
                    return Err("source array index base is not an actual array");
                };
                if actual != element || u64_const(&builder.arena, l) != Some(length) {
                    return Err("source array index changed its element or length");
                }
            }
            s::ProjectionKind::IndexParameter { length, .. } => {
                let Node::Array { element, length: l } = base_node else {
                    return Err("source parameterized array index base is not an actual array");
                };
                if actual != element || builder.arena.nodes[l.0 as usize] != Node::Parameter(length)
                {
                    return Err("source parameterized array index changed its element or slot");
                }
            }
            s::ProjectionKind::SliceIndex { .. } => {
                let Node::Slice(element) = base_node else {
                    return Err("source slice index base is not an actual slice");
                };
                if actual != element {
                    return Err("source slice index changed its element type");
                }
            }
            s::ProjectionKind::ConstantIndex(offset) => match base_node {
                Node::Slice(element) if actual == element => {}
                Node::Array { element, length }
                    if actual == element
                        && u64_const(&builder.arena, length).is_none_or(|len| offset < len) => {}
                _ => return Err("source constant index changed its sequence base or element"),
            },
            s::ProjectionKind::FromEnd { offset, minimum } => match base_node {
                Node::Slice(element) | Node::Array { element, .. }
                    if actual == element && offset >= 1 && offset <= minimum => {}
                _ => return Err("source suffix index changed its sequence base or bounds"),
            },
            s::ProjectionKind::Subslice { .. } => {
                if !matches!(base_node, Node::Slice(_)) || actual != base {
                    return Err("source subslice projection changed its slice type");
                }
            }
            s::ProjectionKind::Subarray { start, end } => {
                let (
                    Node::Array { element, .. },
                    Node::Array {
                        element: out_element,
                        length,
                    },
                ) = (base_node, builder.arena.nodes[actual.0 as usize])
                else {
                    return Err("source subarray projection requires array base and result");
                };
                if start > end
                    || out_element != element
                    || u64_const(&builder.arena, length) != Some(end - start)
                {
                    return Err("source subarray projection changed its element or range length");
                }
            }
            _ => return Err("unexpected sequence projection kind"),
        }
        Ok(())
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
