//! Named evaluated references retain a provider allocation and typed callable targets.
use super::{
    ConstantBody, ConstantHeader, HeaderIndex, HeaderRef, Range, SourceRows, TypeRef, Work,
};
use crate::support::crates::{
    generics::source::{ArtifactRef, DefinitionKind, Region, Type},
    owner_catalog::DeclarationRef,
};

/// The provider-local native handler's stable ordinal and full Rust signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromotedHandler {
    pub identity: u32,
    pub signature: TypeRef,
}

pub(super) fn reference(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    declaration: DeclarationRef,
    constant: ConstantHeader,
    anchor: DeclarationRef,
    handlers: Range,
    work: &mut Work,
) -> Result<(), &'static str> {
    if anchor != declaration {
        let Some(HeaderRef::Constant(index)) = headers.header(anchor, work)? else {
            return Err("source promoted reference has no actual allocation owner");
        };
        let definition = rows.constants()[index as usize];
        if anchor.0 >= declaration.0
            || definition.ty != constant.ty
            || handlers.start != handlers.end
            || !matches!(definition.body, ConstantBody::PromotedReference { anchor: owner, .. } if owner == anchor)
        {
            return Err("source promoted reference changed its actual allocation owner or type");
        }
        return Ok(());
    }
    let fields = fields(rows, headers, constant.ty, work)?;
    let all = &rows.cold().promoted_handlers;
    let group = handlers.slice(all)?;
    if fields.len() != group.len() {
        return Err("source promoted reference changed its complete field count");
    }
    for (ordinal, (field, handler)) in fields.iter().zip(group).enumerate() {
        work.charge(1)?;
        let current = handlers.start as usize + ordinal;
        if field.ty != handler.signature
            || handler.identity as usize > current
            || all.get(handler.identity as usize) != Some(handler)
            || !matches!(
                rows.types.get(handler.signature.0 as usize),
                Some(Type::Function(_))
            )
        {
            return Err("source promoted relocation changed its actual field signature");
        }
    }
    Ok(())
}

fn fields<'rows>(
    rows: &'rows SourceRows,
    headers: &HeaderIndex<'_>,
    ty: TypeRef,
    work: &mut Work,
) -> Result<&'rows [crate::support::crates::generics::source::NominalField], &'static str> {
    let error = "source promoted reference requires an actual closed shared record";
    let Some(&Type::Reference {
        mutable: false,
        region,
        referent,
    }) = rows.types.get(ty.0 as usize)
    else {
        return Err(error);
    };
    let Some(&Type::Nominal {
        declaration,
        arguments,
    }) = rows.types.get(referent.0 as usize)
    else {
        return Err(error);
    };
    if rows.regions.get(region.0 as usize) != Some(&Region::Static)
        || declaration.artifact != ArtifactRef(0)
        || arguments.start != arguments.end
        || rows
            .declarations
            .get(declaration.declaration.0 as usize)
            .is_none_or(|row| row.definition.kind != DefinitionKind::Struct)
    {
        return Err(error);
    }
    let Some(HeaderRef::Nominal(index)) = headers.header(declaration.declaration, work)? else {
        return Err(error);
    };
    let fields = rows.nominals[index as usize].fields.slice(&rows.fields)?;
    if fields.is_empty() {
        return Err(error);
    }
    Ok(fields)
}
