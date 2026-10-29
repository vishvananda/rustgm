//! Calls keep their own actual signature; source context is checked separately.
use super::{Body, Context, Declarations, Stage, g, ir, place, s};
pub(super) fn emit(
    stage: &mut Stage,
    facts: &mut Declarations,
    body: &Body,
    context: Context,
    raw: &mut s::Original,
    sites: &mut super::super::locations::Sites,
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
            ir::Rvalue::Scalar(id) if stage.feature_predicates() => {
                let scalar = body
                    .scalars
                    .get(id.0 as usize)
                    .copied()
                    .ok_or("source scalar value has no actual body scalar slot")?;
                let ty = crate::sema::crates::export_scalar(facts, value.ty).map_err(
                    |_| "source scalar value requires a closed supported primitive type",
                )?;
                let value = crate::sema::crates::export_scalar_value(ty, scalar)?;
                s::ValueKind::Scalar(value.bits().to_le_bytes())
            }
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
        let ty = stage.ty(facts, value.ty, context)?;
        if stage.feature_predicates() && matches!(kind, s::ValueKind::Function(_)) {
            sites.function_formation(stage, body.source.root_body_site(), value.ty, ty)?;
        }
        stage.reserve(4 * size_of::<s::Value>())?;
        raw.values.push(s::Value { ty, kind });
    }
    Ok(())
}
