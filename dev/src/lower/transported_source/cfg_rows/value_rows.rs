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
            ir::Rvalue::Aggregate {
                variant: Some(variant),
                fields,
            } if stage.nominal_transport() => aggregate(facts, value.ty, variant, fields)?,
            ir::Rvalue::Scalar(id) if stage.feature_predicates() => {
                scalar(facts, body, value.ty, id)?
            }
            ir::Rvalue::Read(ir::Place::Temporary(value)) if stage.conversion_roles() => {
                s::ValueKind::Read(s::Place::Temporary(value.0))
            }
            ir::Rvalue::Read(target) => s::ValueKind::Read(place(target)?),
            ir::Rvalue::Borrow {
                place: target,
                kind: ir::BorrowKind::Immediate,
            } => s::ValueKind::Borrow(place(target)?),
            ir::Rvalue::Borrow {
                place: target,
                kind: ir::BorrowKind::TwoPhase,
            } if stage.body_nodes() => s::ValueKind::TwoPhaseBorrow(place(target)?),
            ir::Rvalue::Constrain(value) => s::ValueKind::Constrain(value.0),
            ir::Rvalue::Binary {
                operands,
                operator,
                left,
                right,
            } if stage.body_nodes() => binary(
                facts,
                body,
                index,
                value.ty,
                operands,
                operator,
                [left, right],
            )?,
            ir::Rvalue::Binary { .. } => {
                return Err(
                    "source Binary operations require the version-thirteen body IR profile",
                );
            }
            ir::Rvalue::Cast {
                kind,
                operand,
                direct_variant: false,
            } if stage.conversion_roles() => {
                scalar_cast(facts, body, index, value.ty, operand, kind)?
            }
            ir::Rvalue::Function(definition) => {
                function(stage, facts, value.ty, definition, context)?
            }
            ir::Rvalue::Invoke {
                kind,
                signature,
                arguments,
            } => invoke(
                stage,
                facts,
                (body, index, context),
                kind,
                signature,
                arguments,
            )?,
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

/// A call operator keeps its call trait and caller-side signature; a closure
/// call entry would need its closure transport.
fn invoke(
    stage: &mut Stage,
    facts: &mut Declarations,
    (body, index, context): (&Body, usize, Context),
    kind: crate::support::call_traits::CallTrait,
    signature: crate::types::TypeId,
    arguments: ir::ValueList,
) -> Result<s::ValueKind, &'static str> {
    if !stage.body_nodes() {
        return Err("source call operators require the version-thirteen body IR profile");
    }
    let value = ir::ValueId(u32::try_from(index).map_err(|_| "source call value overflow")?);
    if body
        .closure_calls
        .as_deref()
        .is_some_and(|entries| entries.get(value).is_some())
    {
        return Err("source closure call entries require their closure transport");
    }
    Ok(s::ValueKind::Invoke {
        kind,
        signature: stage.ty(facts, signature, context)?,
        arguments: g::Range {
            start: arguments.start,
            end: arguments.end,
        },
    })
}

fn scalar_cast(
    facts: &Declarations,
    body: &Body,
    before: usize,
    result: crate::types::TypeId,
    operand: ir::ValueId,
    kind: crate::sema::model::Cast,
) -> Result<s::ValueKind, &'static str> {
    if operand.0 as usize >= before {
        return Err("source scalar cast changed its actual preceding operand");
    }
    let input = body
        .values
        .get(operand.0 as usize)
        .ok_or("source scalar cast lost its actual operand")?;
    let from = crate::sema::crates::export_scalar(facts, input.ty)
        .map_err(|_| "source cast operand requires a closed primitive type")?;
    let to = crate::sema::crates::export_scalar(facts, result)
        .map_err(|_| "source cast result requires a closed primitive type")?;
    if crate::sema::crates::scalar_cast(from, to) != Some(kind) {
        return Err("source scalar cast changed its actual primitive conversion");
    }
    Ok(s::ValueKind::ScalarCast(operand.0))
}

fn function(
    stage: &mut Stage,
    facts: &mut Declarations,
    ty: crate::types::TypeId,
    definition: crate::resolve::model::DefId,
    context: Context,
) -> Result<s::ValueKind, &'static str> {
    use crate::support::bindings::SourceIntrinsic;
    let Some(crate::types::Kind::FunctionItem {
        item, arguments, ..
    }) = facts.types.kind(ty)
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
        .is_some_and(|kind| {
            matches!(
                kind,
                SourceIntrinsic::UbChecks
                    | SourceIntrinsic::ArithOffset
                    | SourceIntrinsic::UncheckedSub
                    | SourceIntrinsic::SubWithOverflow
                    | SourceIntrinsic::ColdPath
            )
        })
    {
        return Err("source intrinsic transport requires an explicit body policy profile");
    }
    Ok(s::ValueKind::Function(
        stage.function_application(facts, item, arguments, context)?,
    ))
}

/// Only closed primitive comparisons and bitwise operations have a row form;
/// checked arithmetic and shifts stay refused until their consumer proof exists.
fn binary(
    facts: &Declarations,
    body: &Body,
    before: usize,
    result: crate::types::TypeId,
    operands: ir::BinaryOperands,
    operator: crate::lex::token::Punctuation,
    [left, right]: [ir::ValueId; 2],
) -> Result<s::ValueKind, &'static str> {
    use crate::lex::token::Punctuation as P;
    let operator = match operator {
        P::EqualEqual => s::BinaryOperator::Equal,
        P::BangEqual => s::BinaryOperator::NotEqual,
        P::Less => s::BinaryOperator::Less,
        P::LessEqual => s::BinaryOperator::LessEqual,
        P::Greater => s::BinaryOperator::Greater,
        P::GreaterEqual => s::BinaryOperator::GreaterEqual,
        P::And => s::BinaryOperator::BitAnd,
        P::Or => s::BinaryOperator::BitOr,
        P::Caret => s::BinaryOperator::BitXor,
        P::Plus => s::BinaryOperator::Add,
        P::Minus => s::BinaryOperator::Sub,
        P::Star => s::BinaryOperator::Mul,
        _ => return Err("source checked division or shift requires a wider transport row"),
    };
    let mut operand = None;
    for value in [left, right] {
        if value.0 as usize >= before {
            return Err("source binary operation changed its actual preceding operand");
        }
        let input = body
            .values
            .get(value.0 as usize)
            .ok_or("source binary operation lost its actual operand")?;
        let scalar = crate::sema::crates::export_scalar(facts, input.ty)
            .map_err(|_| "source binary operand requires a closed primitive type")?;
        if operand.replace(scalar).is_some_and(|first| first != scalar) {
            return Err("source binary operation changed its shared operand type");
        }
    }
    crate::sema::crates::export_scalar(facts, result)
        .map_err(|_| "source binary result requires a closed primitive type")?;
    let operands = match operands {
        ir::BinaryOperands::NotRequired => s::OperandForm::NotRequired,
        ir::BinaryOperands::Neither => s::OperandForm::Neither,
        ir::BinaryOperands::Left => s::OperandForm::Left,
        ir::BinaryOperands::Right => s::OperandForm::Right,
        ir::BinaryOperands::Both => s::OperandForm::Both,
    };
    Ok(s::ValueKind::Binary {
        operator,
        operands,
        left: left.0,
        right: right.0,
    })
}

fn aggregate(
    facts: &Declarations,
    ty: crate::types::TypeId,
    variant: crate::sema::members::VariantId,
    fields: ir::ValueList,
) -> Result<s::ValueKind, &'static str> {
    let Some(crate::types::Kind::Adt(owner, _)) = facts.types.kind(ty) else {
        return Err("source aggregate value lacks its actual nominal type");
    };
    if facts.members.variants(owner) != [variant] {
        return Err("source aggregate changed its actual nominal constructor field group");
    }
    Ok(s::ValueKind::Aggregate {
        fields: g::Range {
            start: fields.start,
            end: fields.end,
        },
    })
}

fn scalar(
    facts: &Declarations,
    body: &Body,
    result: crate::types::TypeId,
    id: ir::ScalarId,
) -> Result<s::ValueKind, &'static str> {
    let scalar = body
        .scalars
        .get(id.0 as usize)
        .copied()
        .ok_or("source scalar value has no actual body scalar slot")?;
    let ty = crate::sema::crates::export_scalar(facts, result)
        .map_err(|_| "source scalar value requires a closed supported primitive type")?;
    let value = crate::sema::crates::export_scalar_value(ty, scalar)?;
    Ok(s::ValueKind::Scalar(value.bits().to_le_bytes()))
}
