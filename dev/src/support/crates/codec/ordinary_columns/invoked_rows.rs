//! Complete call requests retain their actual signature opening and effect data.
use super::{Input, Writer, g, id, option, range, recipe_columns, s};

pub(super) fn write(out: &mut Writer, row: &s::CallRequest) -> Result<(), &'static str> {
    if row.safety != s::CallSafety::Safe {
        return Err("source call safety requires its version-eight profile");
    }
    fields(out, row)
}
pub(super) fn safety_write(out: &mut Writer, row: &s::CallRequest) -> Result<(), &'static str> {
    fields(out, row)?;
    out.u8(match row.safety {
        s::CallSafety::Safe => 0,
        s::CallSafety::ExplicitUnsafeBlock => 1,
        s::CallSafety::UnsafeFunction => 2,
    })
}
fn fields(out: &mut Writer, row: &s::CallRequest) -> Result<(), &'static str> {
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
        effect: recipe_columns::call_effect_read(
            input,
            "source call effect requires its wider profile",
        )?,
        safety: if input.safety {
            match input.reader.u8()? {
                0 => s::CallSafety::Safe,
                1 => s::CallSafety::ExplicitUnsafeBlock,
                2 => s::CallSafety::UnsafeFunction,
                _ => return Err("unknown source call safety context"),
            }
        } else {
            s::CallSafety::Safe
        },
    })
}
