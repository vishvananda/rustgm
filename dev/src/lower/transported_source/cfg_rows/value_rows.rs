//! Calls keep their own actual signature; source context is checked separately.
use super::{Body, Context, Declarations, Stage, g, ir, place, s};
pub(super) fn emit(
    stage: &mut Stage,
    facts: &mut Declarations,
    body: &Body,
    context: Context,
    raw: &mut s::Original,
) -> Result<(), &'static str> {
    stage.reserve(body.arguments.len().saturating_mul(size_of::<u32>()))?;
    for argument in &body.arguments {
        stage.charge(1)?;
        raw.arguments.push(argument.0);
    }
    for (index, value) in body.values.iter().enumerate() {
        stage.charge(1)?;
        stage.prepare_type_arena(facts, value.ty)?;
        let kind = match value.kind {
            ir::Rvalue::Unit => s::ValueKind::Unit,
            ir::Rvalue::Read(target) => s::ValueKind::Read(place(target)?),
            ir::Rvalue::Borrow {
                place: target,
                kind: ir::BorrowKind::Immediate,
            } => s::ValueKind::Borrow(place(target)?),
            ir::Rvalue::Constrain(value) => s::ValueKind::Constrain(value.0),
            ir::Rvalue::Function(definition) => {
                let Some(crate::types::Kind::FunctionItem {
                    item, arguments, ..
                }) = facts.types.kind(value.ty)
                else {
                    return Err("source function value has no actual function-item application");
                };
                if item.0 != definition.0 {
                    return Err("source function value changed its actual declaration");
                }
                if facts
                    .source_bindings
                    .as_deref()
                    .map(|c| c.intrinsic_kind(definition))
                    .transpose()?
                    .flatten()
                    == Some(crate::support::bindings::SourceIntrinsic::UbChecks)
                {
                    return Err(
                        "source runtime-check transport requires an explicit body policy profile",
                    );
                }
                s::ValueKind::Function(stage.function_application(facts, item, arguments, context)?)
            }
            ir::Rvalue::Call {
                signature,
                callee,
                arguments,
            } => s::ValueKind::Call {
                signature: stage.call_signature(
                    facts,
                    signature,
                    context,
                    u32::try_from(index).map_err(|_| "source call value overflow")?,
                )?,
                callee: callee.0,
                arguments: g::Range {
                    start: arguments.start,
                    end: arguments.end,
                },
            },
            _ => return Err("source rvalue requires a complete wider transport row"),
        };
        stage.reserve(4 * size_of::<s::Value>())?;
        raw.values.push(s::Value {
            ty: stage.ty(facts, value.ty, context)?,
            kind,
        });
    }
    Ok(())
}
