//! Trait objects occupy one type row and retain their inputs in cold columns.
use super::{Input, Writer, address_read, address_write, g, id, range, s};
use crate::support::{crates::MAX_COMPONENTS, marker_traits::AutoTrait, retained::Growth};

pub(super) const TAG: u8 = 20;
const MARKERS: [AutoTrait; 7] = [
    AutoTrait::Send,
    AutoTrait::Sync,
    AutoTrait::Unpin,
    AutoTrait::UnwindSafe,
    AutoTrait::RefUnwindSafe,
    AutoTrait::Freeze,
    AutoTrait::UnsafeUnpin,
];

#[derive(Default)]
pub(super) struct Encoder {
    types: usize,
    bindings: usize,
    markers: usize,
}
impl Encoder {
    pub(super) fn row(
        &mut self,
        out: &mut Writer,
        reference: s::DynamicTypeRef,
        columns: &s::DynamicColumns,
    ) -> Result<(), &'static str> {
        let row = columns
            .objects
            .get(self.types)
            .filter(|_| reference.0 as usize == self.types)
            .ok_or("source dynamic recipes are not in type-row order")?;
        if row.bindings.start as usize != self.bindings
            || row.markers.start as usize != self.markers
        {
            return Err("source dynamic bound columns are not in object order");
        }
        out.u8(TAG)?;
        out.u8(u8::from(row.principal.is_some()))?;
        if let Some(principal) = row.principal {
            address_write(out, principal)?;
        }
        id(out, row.principal_arguments)?;
        range(out, row.arguments)?;
        id(out, row.binder.0)?;
        id(out, row.region.0)?;
        let bindings = row.bindings.slice(&columns.bindings)?;
        out.u32(bindings.len())?;
        for binding in bindings {
            address_write(out, binding.member)?;
            address_write(out, binding.owner)?;
            id(out, binding.arguments)?;
        }
        let markers = row.markers.slice(&columns.markers)?;
        out.u32(markers.len())?;
        for marker in markers {
            let tag = MARKERS
                .iter()
                .position(|&actual| actual == marker.marker)
                .and_then(|index| u8::try_from(index).ok())
                .ok_or("source dynamic marker has no wire identity")?;
            out.u8(tag)?;
            address_write(out, marker.declaration)?;
        }
        self.types += 1;
        self.bindings += bindings.len();
        self.markers += markers.len();
        Ok(())
    }
    pub(super) fn finish(self, columns: &s::DynamicColumns) -> Result<(), &'static str> {
        if self.types != columns.objects.len()
            || self.bindings != columns.bindings.len()
            || self.markers != columns.markers.len()
        {
            return Err("source dynamic rows lack their actual type owners");
        }
        Ok(())
    }
}

pub(super) fn read(
    read: &mut Input<'_, '_>,
    columns: &mut s::DynamicColumns,
) -> Result<s::Type, &'static str> {
    let principal = read.boolean()?.then(|| address_read(read)).transpose()?;
    let principal_arguments = read.id()?;
    let arguments = read.range()?;
    let binder = g::BinderRef(read.id()?);
    let region = s::RegionRef(read.id()?);
    let bindings = append(read, &mut columns.bindings, MAX_COMPONENTS, 20, |read| {
        Ok(s::DynamicBinding {
            member: address_read(read)?,
            owner: address_read(read)?,
            arguments: read.id()?,
        })
    })?;
    let markers = append(read, &mut columns.markers, MARKERS.len(), 9, |read| {
        let marker = *MARKERS
            .get(read.reader.u8()? as usize)
            .ok_or("unknown source dynamic marker")?;
        Ok(s::DynamicMarker {
            marker,
            declaration: address_read(read)?,
        })
    })?;
    let shape = s::DynamicTypeRef(ordinal(columns.objects.len())?);
    reserve(read, &columns.objects, 1)?;
    columns.objects.push(s::DynamicType {
        principal,
        principal_arguments,
        arguments,
        bindings,
        markers,
        binder,
        region,
    });
    Ok(s::Type::Dynamic(shape))
}

fn append<T>(
    read: &mut Input<'_, '_>,
    rows: &mut Vec<T>,
    limit: usize,
    minimum: usize,
    mut row: impl FnMut(&mut Input<'_, '_>) -> Result<T, &'static str>,
) -> Result<g::Range, &'static str> {
    let count = read.reader.count(limit, minimum)?;
    let start = ordinal(rows.len())?;
    read.reader.atoms(count)?;
    reserve(read, rows, count)?;
    rows.reserve(count);
    for _ in 0..count {
        rows.push(row(read)?);
    }
    Ok(g::Range {
        start,
        end: ordinal(rows.len())?,
    })
}
fn reserve<T>(read: &mut Input<'_, '_>, rows: &Vec<T>, count: usize) -> Result<(), &'static str> {
    let growth = Growth::vector::<T>(rows.len(), rows.capacity(), count);
    read.reserve(growth.bytes)
}
fn ordinal(index: usize) -> Result<u32, &'static str> {
    u32::try_from(index).map_err(|_| "source dynamic column exceeds its index range")
}
