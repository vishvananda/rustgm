//! Complete call requests retain their actual signature opening and effect data.
use super::{Input, Writer, g, id, option, range, recipe_columns, s};

pub(super) fn write(out: &mut Writer, row: &s::CallRequest) -> Result<(), &'static str> {
    for value in [row.value, row.site, row.target.0] {
        id(out, value)?;
    }
    option(out, row.method.map(|method| method.0))?;
    id(out, row.signature.0)?;
    out.u8(u8::from(row.opening.is_some()))?;
    if let Some(opening) = row.opening {
        id(out, opening.binder.0)?;
        range(out, opening.regions)?;
    }
    recipe_columns::effect_write(out, row.effect)
}
pub(super) fn read(input: &mut Input<'_, '_>) -> Result<s::CallRequest, &'static str> {
    Ok(s::CallRequest {
        value: input.id()?,
        site: input.id()?,
        target: s::FunctionApplicationRef(input.id()?),
        method: input.option()?.map(s::TraitApplicationRef),
        signature: g::TypeRef(input.id()?),
        opening: if input.boolean()? {
            Some(s::Opening {
                binder: g::BinderRef(input.id()?),
                regions: input.range()?,
            })
        } else {
            None
        },
        effect: recipe_columns::callable_effect_read(
            input,
            "source call effect requires its wider profile",
        )?,
    })
}
