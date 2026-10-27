//! Logical discriminator type does not depend on optimized physical tag storage.
use super::{Declarations, Error, header_attributes};
use crate::types::{ItemId, Kind, Primitive, TypeId, aggregate};

pub(in crate::sema) enum Output {
    Ready(TypeId),
    Fields(ItemId),
    Unresolved,
}

pub(in crate::sema::compiler_items) fn classify(
    facts: &Declarations,
    mut ty: TypeId,
) -> Result<Output, Error> {
    loop {
        header_attributes::charge_source_contract_query(facts, 1)?;
        return Ok(
            match facts.types.kind(ty).ok_or_else(|| {
                Error::unlocated("source discriminant input is outside its type arena")
            })? {
                Kind::Pattern { base, .. } => {
                    ty = base;
                    continue;
                }
                Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) => Output::Unresolved,
                Kind::Adt(item, _) => {
                    let Some(declaration) = facts.aggregates.get(item) else {
                        return Ok(Output::Fields(item));
                    };
                    Output::Ready(TypeId::primitive(
                        if declaration.kind == aggregate::Kind::Enum {
                            declaration.repr.tag.unwrap_or(Primitive::Isize)
                        } else {
                            Primitive::U8
                        },
                    ))
                }
                _ => Output::Ready(TypeId::primitive(Primitive::U8)),
            },
        );
    }
}
