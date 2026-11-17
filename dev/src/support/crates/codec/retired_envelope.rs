//! A removed-feature attachment belongs only to the outermost crate identity.
use super::{HEADER, Interface, MAGIC, Reader, SCHEMA, Writer};
use crate::support::{
    crates::{
        MAX_BYTES, MAX_COMPONENTS, MAX_NAME_BYTES, RemovedFeature, RemovedFeatures, RemovedVersion,
    },
    retained::Growth,
};
pub(super) const TAG: u8 = 16;
const VERSION: u8 = 1;

pub(super) fn encode(interface: &Interface, inner: &Vec<u8>) -> Result<Vec<u8>, &'static str> {
    let table = interface
        .removed_features()
        .ok_or("missing removed feature attachment")?;
    // The old encoded buffer coexists with the new one. Writer::source prepays
    // geometric output growth beside this actual retained base before every append.
    let base = interface
        .base_retained_bytes()
        .saturating_add(
            interface
                .generic_payload()
                .map_or(0, crate::support::crates::generics::Payload::retained_bytes),
        )
        .saturating_add(inner.capacity());
    let mut out = Writer::source(base);
    out.bytes(MAGIC)?;
    out.u16(SCHEMA)?;
    out.bytes(&[0, 0, TAG, VERSION])?;
    out.identity(table.owner())?;
    out.u32(table.entries().len())?;
    for row in table.entries() {
        out.string(&row.feature)?;
        out.string(&row.reason)?;
        out.string(&row.link)?;
        for value in [row.since.major, row.since.minor, row.since.patch] {
            out.u16(value)?;
        }
    }
    out.u32(inner.len())?;
    out.bytes(inner)?;
    Ok(out.finish())
}
fn string(input: &mut Reader<'_>, bytes: &mut usize) -> Result<String, &'static str> {
    let value = input.text(MAX_NAME_BYTES)?;
    *bytes = bytes.saturating_add(value.len());
    if *bytes > MAX_BYTES {
        return Err("removed feature decode exceeds retained storage budget");
    }
    Ok(value.to_owned())
}
pub(super) fn decode(input: &mut Reader<'_>) -> Result<Interface, &'static str> {
    if input.u8()? != VERSION {
        return Err("unknown removed library feature metadata version");
    }
    let owner = input.identity()?;
    let count = input.count(MAX_COMPONENTS, 18)?;
    if count == 0 {
        return Err("removed library feature envelope requires declarations");
    }
    input.atoms(count)?;
    let growth = Growth::vector::<RemovedFeature>(0, 0, count);
    let mut bytes = input
        .retained_prefix()
        .saturating_add(size_of::<RemovedFeatures>())
        .saturating_add(growth.bytes);
    if bytes > MAX_BYTES {
        return Err("removed feature decode exceeds retained storage budget");
    }
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| "removed feature decode allocation failed")?;
    if rows.capacity().saturating_mul(size_of::<RemovedFeature>()) > growth.bytes {
        return Err("removed feature decode exceeds prepaid capacity");
    }
    for _ in 0..count {
        rows.push(RemovedFeature {
            feature: string(input, &mut bytes)?,
            reason: string(input, &mut bytes)?,
            link: string(input, &mut bytes)?,
            since: RemovedVersion {
                major: input.u16()?,
                minor: input.u16()?,
                patch: input.u16()?,
            },
        });
    }
    let length = input.u32()?;
    let inner = input.bytes(length)?;
    if !input.done() {
        return Err("trailing removed library feature metadata");
    }
    if matches!(inner.get(HEADER), Some(&TAG)) {
        return Err("repeated removed library feature envelope");
    }
    let interface = super::decode_nested_with_retained(inner, bytes)?;
    interface.with_removed_features(RemovedFeatures::new(owner, rows)?)
}
