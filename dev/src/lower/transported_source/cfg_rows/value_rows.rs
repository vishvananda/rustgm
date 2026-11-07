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
            } if stage.nominal_transport() => aggregate(stage, facts, value.ty, variant, fields)?,
            ir::Rvalue::IsVariant { .. }
            | ir::Rvalue::Cast {
                kind: crate::sema::model::Cast::EnumToInteger,
                direct_variant: false,
                ..
            } if stage.body_nodes() => variant_operation(stage, value.kind)?,
            ir::Rvalue::Scalar(id) if stage.feature_predicates() => {
                scalar(facts, body, value.ty, id)?
            }
            ir::Rvalue::Text(text) if stage.body_nodes() => {
                text_literal(stage, facts, raw, value.ty, text)?
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
            ir::Rvalue::Call { .. } => call(stage, facts, context, index, value.kind)?,
            _ => return Err("source rvalue requires a complete wider transport row"),
        };
        let ty = stage.ty(facts, value.ty, context)?;
        if stage.feature_predicates() && matches!(kind, s::ValueKind::Function(_))
            || requested_formation(stage, facts, value.ty)
            || parameter_reborrow(stage, facts, value.ty, kind)
        {
            sites.function_formation(stage, body.source.root_body_site(), value.ty, ty)?;
        }
        stage.reserve(4 * size_of::<s::Value>())?;
        raw.values.push(s::Value { ty, kind });
    }
    Ok(())
}

/// A direct call keeps its own actual signature and argument range.
fn call(
    stage: &mut Stage,
    facts: &mut Declarations,
    context: Context,
    index: usize,
    kind: ir::Rvalue,
) -> Result<s::ValueKind, &'static str> {
    let ir::Rvalue::Call {
        signature,
        callee,
        arguments,
    } = kind
    else {
        return Err("source call changed its actual form");
    };
    Ok(s::ValueKind::Call {
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
    })
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

/// Version thirteen: a closed nominal owns no local formation obligation, so
/// the encoder requests one each consumer re-proves.
pub(super) fn requested_formation(
    stage: &Stage,
    facts: &Declarations,
    ty: crate::types::TypeId,
) -> bool {
    stage.body_nodes() && facts.types.contains_nominals(ty) == Some(true)
}
/// Version thirteen: reborrowing a reference to a type parameter has no
/// checked formation fact of its own, so the original requests one. The
/// consumer proves it again; the request carries no outlives claim.
fn parameter_reborrow(
    stage: &Stage,
    facts: &Declarations,
    ty: crate::types::TypeId,
    kind: s::ValueKind,
) -> bool {
    stage.body_nodes()
        && matches!(
            kind,
            s::ValueKind::Borrow(_) | s::ValueKind::TwoPhaseBorrow(_)
        )
        && matches!(
            facts.types.kind(ty),
            Some(crate::types::Kind::Reference { referent, .. })
                if matches!(facts.types.kind(referent), Some(crate::types::Kind::Parameter(_)))
        )
}
/// Version thirteen: a literal joins its original's text column in value
/// order. Only a shared `'static` reference to `str` is a text value.
fn text_literal(
    stage: &mut Stage,
    facts: &Declarations,
    raw: &mut s::Original,
    ty: crate::types::TypeId,
    text: crate::sema::text::Id,
) -> Result<s::ValueKind, &'static str> {
    if !matches!(
        facts.types.kind(ty),
        Some(crate::types::Kind::Reference {
            mutable: false,
            region: crate::types::RegionId::STATIC,
            referent,
        }) if referent == crate::types::TypeId::primitive(crate::types::Primitive::Str)
    ) {
        return Err("source text literal is not a shared static str reference");
    }
    let bytes = facts.texts.get(text)?;
    stage.charge(bytes.len().saturating_add(1))?;
    stage.reserve(bytes.len().saturating_add(4 * size_of::<Box<str>>()))?;
    let contents = std::str::from_utf8(bytes).map_err(|_| "source text literal is not UTF-8")?;
    let texts = raw.texts.get_or_insert_with(Box::default);
    let ordinal = u32::try_from(texts.rows.len()).map_err(|_| "source text column overflow")?;
    texts.rows.push(contents.into());
    Ok(s::ValueKind::Text(ordinal))
}
/// A constructor test or a fieldless enum's integer conversion.
fn variant_operation(stage: &mut Stage, kind: ir::Rvalue) -> Result<s::ValueKind, &'static str> {
    Ok(match kind {
        ir::Rvalue::IsVariant {
            place: target,
            variant,
        } => s::ValueKind::IsVariant {
            place: place(target)?,
            variant: stage.variant(variant)?,
        },
        ir::Rvalue::Cast { operand, .. } => s::ValueKind::EnumCast(operand.0),
        _ => return Err("source enum operation changed its actual form"),
    })
}
fn aggregate(
    stage: &mut Stage,
    facts: &Declarations,
    ty: crate::types::TypeId,
    variant: crate::sema::members::VariantId,
    fields: ir::ValueList,
) -> Result<s::ValueKind, &'static str> {
    let Some(crate::types::Kind::Adt(owner, _)) = facts.types.kind(ty) else {
        return Err("source aggregate value lacks its actual nominal type");
    };
    let fields = g::Range {
        start: fields.start,
        end: fields.end,
    };
    let enumerated = facts
        .aggregates
        .get(owner)
        .is_some_and(|shape| shape.kind == crate::types::aggregate::Kind::Enum);
    if enumerated && stage.body_nodes() && facts.members.variants(owner).contains(&variant) {
        return Ok(s::ValueKind::VariantAggregate {
            variant: stage.variant(variant)?,
            fields,
        });
    }
    if enumerated || facts.members.variants(owner) != [variant] {
        return Err("source aggregate changed its actual nominal constructor field group");
    }
    Ok(s::ValueKind::Aggregate { fields })
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
