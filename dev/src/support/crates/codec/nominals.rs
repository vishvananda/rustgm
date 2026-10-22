//! Profile three encodes one graph, then canonical declaration and field roots.
use super::{Reader, Writer, signatures};
use crate::support::crates::{
    CrateIdentity, ExternalDef, ExternalKind, MAX_COMPONENTS, Nominals, Record, RecordField,
    TypeGraph, TypeRef,
};

pub(super) fn encode(out: &mut Writer, nominals: &Nominals) -> Result<(), &'static str> {
    signatures::encode_graph(out, nominals.types(), true)?;
    out.u32(nominals.records().len())?;
    for record in nominals.records() {
        out.u32(0)?; // Current artifact owner; never a local definition ordinal.
        out.bytes(&[0, 3, u8::from(record.public())])?; // Type namespace, named record.
        out.path(record.definition().path())?;
        out.u32(record.fields().len())?;
        for field in record.fields() {
            out.string(field.name())?;
            out.u16(field.ty().0)?;
            out.u8(u8::from(field.public()))?;
        }
    }
    Ok(())
}
pub(super) fn decode(
    input: &mut Reader<'_>,
    owner: &CrateIdentity,
    dependencies: usize,
) -> Result<Nominals, &'static str> {
    let types = TypeGraph::new(signatures::decode_graph(input, true)?)?;
    let count = input.count(TypeGraph::MAX_NODES, 18)?;
    input.atoms(count)?;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        input.owner(dependencies)?;
        if input.bytes(2)? != [0, 3] {
            return Err("invalid metadata record identity");
        }
        let public = visibility(input)?;
        let definition = ExternalDef::new(owner.clone(), input.path()?, ExternalKind::Record)?;
        let count = input.count(MAX_COMPONENTS, 8)?;
        input.atoms(count.saturating_mul(2))?;
        let mut fields = Vec::with_capacity(count);
        for _ in 0..count {
            fields.push(RecordField::new(
                input.string(256)?,
                TypeRef(input.u16()?),
                visibility(input)?,
            )?);
        }
        records.push(Record::new(definition, public, fields)?);
    }
    Nominals::new(types, records)
}
fn visibility(input: &mut Reader<'_>) -> Result<bool, &'static str> {
    match input.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err("unknown metadata field or record visibility"),
    }
}
