//! Typed signatures decode into bounded wire graphs before frontend interning.
use super::{Reader, Writer};
use crate::support::crates::{Lifetime, Signature, TypeGraph, TypeNode, TypeRef};

pub(super) fn encode(
    out: &mut Writer,
    signature: &Signature,
    typed: bool,
) -> Result<(), &'static str> {
    if !typed {
        out.u16(u16::try_from(signature.parameters().len()).map_err(|_| "too many parameters")?)?;
        for &id in signature
            .parameters()
            .iter()
            .chain(std::iter::once(&signature.result()))
        {
            let Some(TypeNode::Scalar(scalar)) = signature.types().node(id) else {
                return Err("scalar metadata contains a reference signature");
            };
            out.u8(scalar as u8)?;
        }
        return Ok(());
    }
    out.u16(signature.bound_regions())?;
    out.u16(
        u16::try_from(signature.types().nodes().len()).map_err(|_| "too many metadata types")?,
    )?;
    for node in signature.types().nodes() {
        match *node {
            TypeNode::Scalar(scalar) => out.bytes(&[0, scalar as u8])?,
            TypeNode::Reference {
                region,
                mutable,
                referent,
            } => {
                out.u8(1)?;
                match region {
                    Lifetime::Static => out.u8(0)?,
                    Lifetime::Bound(slot) => {
                        out.u8(1)?;
                        out.u16(slot)?;
                    }
                }
                out.u8(u8::from(mutable))?;
                out.u16(referent.0)?;
            }
        }
    }
    out.u16(u16::try_from(signature.parameters().len()).map_err(|_| "too many parameters")?)?;
    for id in signature.parameters() {
        out.u16(id.0)?;
    }
    out.u16(signature.result().0)
}
pub(super) fn decode(
    input: &mut Reader<'_>,
    typed: bool,
    unsafe_: bool,
) -> Result<Signature, &'static str> {
    if !typed {
        let count = usize::from(input.u16()?);
        if count > 256 || count.saturating_add(1) > input.remaining() {
            return Err("invalid metadata parameter count");
        }
        input.atoms(count + 1)?;
        let mut parameters = Vec::with_capacity(count);
        for _ in 0..count {
            parameters.push(input.scalar()?);
        }
        return Signature::new(parameters, input.scalar()?, unsafe_);
    }
    let bound_regions = input.u16()?;
    let count = usize::from(input.u16()?);
    if bound_regions > 256
        || count == 0
        || count > TypeGraph::MAX_NODES
        || count > input.remaining() / 2
    {
        return Err("invalid metadata type graph count");
    }
    input.atoms(count + usize::from(bound_regions))?;
    let mut nodes = Vec::with_capacity(count);
    for _ in 0..count {
        nodes.push(match input.u8()? {
            0 => TypeNode::Scalar(input.scalar()?),
            1 => {
                let region = match input.u8()? {
                    0 => Lifetime::Static,
                    1 => Lifetime::Bound(input.u16()?),
                    _ => return Err("unknown metadata lifetime kind"),
                };
                let mutable = match input.u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err("unknown metadata reference mutability"),
                };
                TypeNode::Reference {
                    region,
                    mutable,
                    referent: TypeRef(input.u16()?),
                }
            }
            _ => return Err("unknown metadata type graph node"),
        });
    }
    let count = usize::from(input.u16()?);
    if count > 256 || count.saturating_add(1) > input.remaining() / 2 {
        return Err("invalid metadata typed parameter count");
    }
    input.atoms(count + 1)?;
    let mut parameters = Vec::with_capacity(count);
    for _ in 0..count {
        parameters.push(TypeRef(input.u16()?));
    }
    Signature::with_types(
        nodes,
        bound_regions,
        parameters,
        TypeRef(input.u16()?),
        unsafe_,
    )
}
