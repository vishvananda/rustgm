//! Deterministic layout inspection consumes canonical types and completed ABI queries.
use super::{model::Declarations, type_view};
use crate::{
    abi::layout::{Error, Shape},
    resolve::{
        expansion::Report,
        model::{DefKind, Namespace},
    },
    support::quoted,
    types::TypeId,
};
use std::io::{self, Write};

/// Complete fixed-size layouts before emitting any inspection output.
/// Unsized canonical types are printed explicitly without inventing a size.
///
/// # Errors
/// Rejects unsupported representations and invalid storage graphs.
pub fn analyze(facts: &mut Declarations) -> Result<(), Error> {
    for index in 0..facts.types.len() {
        let ty = TypeId(u32::try_from(index).map_err(|_| Error {
            ty: TypeId::UNIT,
            message: "too many layout types",
        })?);
        if facts.types.is_sized(ty) == Some(true) {
            facts.layouts.demand(&facts.types, &facts.aggregates, ty)?;
        }
    }
    Ok(())
}
/// # Errors
/// Propagates output errors or an incomplete layout inspection query.
pub fn write(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "layouts 1 target=x86_64-unknown-linux-gnu")?;
    type_view::types(&facts.types, out)?;
    for (index, ty) in facts.definitions.iter().enumerate() {
        let Some(ty) = ty else {
            continue;
        };
        let definition = &report.resolution.definitions[index];
        if definition.namespace != Namespace::Type || definition.kind == DefKind::Variant {
            continue;
        }
        write!(out, "def {index} type={} name=", ty.0)?;
        quoted::bytes(
            out,
            report.package.names.get(definition.name).as_bytes(),
            false,
        )?;
        if let Some(key) = definition.node {
            write!(out, " source={}:{}", key.file.0, key.node.0)?;
        }
        writeln!(out)?;
    }
    for index in 0..facts.types.len() {
        let ty = TypeId(u32::try_from(index).map_err(io::Error::other)?);
        if facts.types.is_sized(ty) != Some(true) {
            writeln!(out, "layout {index} unsized")?;
            continue;
        }
        let layout = facts
            .layouts
            .get(ty)
            .ok_or_else(|| io::Error::other("missing inspected layout"))?;
        write!(
            out,
            "layout {index} size={} align={} inhabited={} references={}",
            layout.size, layout.align, layout.inhabited, layout.contains_reference
        )?;
        match layout.shape {
            Shape::Scalar => writeln!(out, " scalar")?,
            Shape::WidePointer => writeln!(out, " wide-pointer data=0 length=8")?,
            Shape::Array {
                element,
                stride,
                length,
            } => writeln!(
                out,
                " array element={} stride={stride} length={length}",
                element.0
            )?,
            Shape::Aggregate { tag, .. } => {
                write!(out, " aggregate tag=")?;
                if let Some(tag) = tag {
                    writeln!(out, "{}@{}", tag.ty.name(), tag.offset)?;
                } else {
                    writeln!(out, "-")?;
                }
                for (ordinal, variant) in facts.layouts.variants(layout).iter().enumerate() {
                    writeln!(
                        out,
                        "variant {index}:{ordinal} discriminant={}",
                        variant.discriminant
                    )?;
                    let fields = facts
                        .types
                        .list(variant.fields)
                        .ok_or_else(|| io::Error::other("missing inspected fields"))?;
                    let offsets = facts.layouts.offsets(variant);
                    if fields.len() != offsets.len() {
                        return Err(io::Error::other("incomplete inspected field offsets"));
                    }
                    for (field, (ty, offset)) in fields.iter().zip(offsets).enumerate() {
                        writeln!(
                            out,
                            "field {index}:{ordinal}:{field} type={} offset={offset}",
                            ty.0
                        )?;
                    }
                }
            }
        }
    }
    write!(out, "eof ")?;
    count(facts, out)
}
/// # Errors
/// Propagates output errors.
pub fn count(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut variants = 0;
    let mut fields = 0;
    for index in 0..facts.types.len() {
        let ty = TypeId(u32::try_from(index).map_err(io::Error::other)?);
        if let Some(layout) = facts.layouts.get(ty) {
            let groups = facts.layouts.variants(layout);
            variants += groups.len();
            fields += groups
                .iter()
                .map(|v| facts.layouts.offsets(v).len())
                .sum::<usize>();
        }
    }
    writeln!(
        out,
        "types {} layouts {} variants {variants} fields {fields}",
        facts.types.len(),
        facts.layouts.len()
    )
}
