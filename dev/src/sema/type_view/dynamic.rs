//! The object view spells complete bound identities, not private interner indices.
use crate::types::{Kind, Store};
use std::io::{self, Write};

pub(super) fn write(store: &Store, kind: Kind, out: &mut impl Write) -> io::Result<()> {
    let Kind::Dynamic {
        shape: id,
        arguments,
        bound_regions,
        region,
    } = kind
    else {
        return Err(io::Error::other("object view requires a dynamic type"));
    };
    let shape = store
        .dynamic_shape(id)
        .ok_or_else(|| io::Error::other("invalid object shape"))?;
    write!(out, "dynamic principal=")?;
    if let Some(principal) = shape.principal {
        write!(out, "{}", principal.0)?;
    } else {
        write!(out, "none")?;
    }
    write!(out, " trait-arguments={} auto=[", shape.principal_arguments)?;
    for (i, item) in store
        .dynamic_auto_traits(id)
        .ok_or_else(|| io::Error::other("invalid object auto traits"))?
        .iter()
        .enumerate()
    {
        if i != 0 {
            write!(out, ",")?;
        }
        write!(out, "{}", item.name())?;
    }
    write!(out, "] bindings=[")?;
    for (i, binding) in store
        .dynamic_bindings(id)
        .ok_or_else(|| io::Error::other("invalid object associated bindings"))?
        .iter()
        .enumerate()
    {
        if i != 0 {
            write!(out, ",")?;
        }
        write!(
            out,
            "{}@{}/{}",
            binding.item.0, binding.trait_item.0, binding.arguments
        )?;
    }
    write!(out, "] binders={bound_regions} region=")?;
    super::region(region, out)?;
    super::generic_arguments(store, arguments, out)
}
