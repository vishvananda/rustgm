//! Reconstruct the admitted source operation domain from its sealed current view.
use super::{
    ImportedOriginals, encoded_assembly, encoded_literals, encoded_texts,
    operations::{self, lexical},
    regional_records,
};
use crate::{
    body::{
        drops::values,
        ir::{
            Action, Body, BorrowKind, GuardId, LocalId, Owner, Rvalue, Statement, Value, ValueId,
        },
    },
    sema::crates::SourceOriginal,
    support::crates::generics::source as s,
};

#[cfg(test)]
#[path = "../../../tests/invariants/effect_artifacts.rs"]
mod effect_tests;

pub(super) fn build(
    view: &SourceOriginal<'_>,
    work: &mut ImportedOriginals,
) -> Result<Body, &'static str> {
    let record = view.raw();
    if (!record.variables.is_empty() && !view.numbered())
        || (!v4(view) && (!record.calls.is_empty() || !record.arguments.is_empty()))
    {
        return Err(
            "source regional/projected/call body requires its wider ordinary import profile",
        );
    }
    if !record.projections.is_empty() || !record.openings.is_empty() {
        regional_records::require(view)?;
    }
    work.charge(
        record
            .locals
            .len()
            .saturating_add(record.values.len())
            .saturating_add(record.statements.len())
            .saturating_add(record.blocks.len())
            .saturating_add(record.scopes.len())
            .saturating_add(record.exits.len())
            .saturating_add(record.temporaries.len())
            .saturating_add(1),
    )?;
    let mut body = Body::original(
        owner(view),
        view.source()?,
        view.ty(record.result, &mut work.steps)?,
    );
    body.const_context = view.const_context();
    body.locals = operations::locals(&record.locals, |local| {
        Ok(operations::LocalData {
            ty: view.ty(local.ty, &mut work.steps)?,
            definition: local
                .definition
                .map(|d| view.declaration(d, &mut work.steps))
                .transpose()?,
            source: view.site(local.site, &mut work.steps)?,
        })
    })?;
    body.static_types = regional_records::static_types(view, &mut work.steps)?;
    body.projections = regional_records::projections(view, work)?;
    encoded_literals::allocate(record, &mut body, work)?;
    let mut values = Vec::with_capacity(record.values.len());
    for value in &record.values {
        let ty = view.ty(value.ty, &mut work.steps)?;
        let kind = value_kind(view, *value, &mut body, work)?;
        values.push(Value { ty, kind });
    }
    body.values = values;
    encoded_assembly::payload(view, &mut body, work)?;
    work.charge(record.arguments.len())?;
    body.arguments = record.arguments.iter().copied().map(ValueId).collect();
    body.statements = record
        .statements
        .iter()
        .map(|statement| {
            Ok(Statement {
                source: view.site(statement.site, &mut work.steps)?,
                action: action(view, statement.action, &mut work.steps)?,
            })
        })
        .collect::<Result<_, &'static str>>()?;
    let branches = view.rows().descriptor().profile().branches();
    body.blocks = operations::blocks(&record.blocks, branches, &mut |index| {
        view.site(index, &mut work.steps)
    })?;
    guards_into(view, &mut body, work)?;
    lexical::fill_records(
        lexical::Records {
            scopes: &record.scopes,
            exits: &record.exits,
            temporaries: &record.temporaries,
        },
        &mut body,
        &mut |index| view.site(index, &mut work.steps),
    )?;
    owned(view, &mut body, work)?;
    Ok(body)
}
pub(super) fn owner(view: &SourceOriginal<'_>) -> Owner {
    if view.is_anonymous() {
        return Owner::AnonymousConstant;
    }
    if let Some((identity, signature, execution)) = view.closure_owner() {
        return Owner::Closure {
            identity,
            signature,
            execution,
        };
    }
    let definition = view.definition();
    match view.signature() {
        Some(signature) => Owner::Function {
            definition,
            signature,
            execution: None,
        },
        None => Owner::Initializer {
            definition,
            kind: crate::sema::model::InitializerKind::Const,
        },
    }
}
/// One value row's rebuilt operation; the row's types were re-proved by the
/// consumer's own validation.
fn value_kind(
    view: &SourceOriginal<'_>,
    value: s::Value,
    body: &mut Body,
    work: &mut ImportedOriginals,
) -> Result<Rvalue, &'static str> {
    Ok(match value.kind {
        s::ValueKind::Unit => Rvalue::Unit,
        s::ValueKind::Aggregate { fields } => Rvalue::Aggregate {
            variant: view.aggregate_variant(value.ty, &mut work.steps)?,
            fields: crate::body::ir::ValueList {
                start: fields.start,
                end: fields.end,
            },
        },
        s::ValueKind::Scalar(_) => encoded_literals::append(view, value, body)?,
        s::ValueKind::Text(_) => encoded_texts::append(view)?,
        s::ValueKind::Assembly { .. } | s::ValueKind::AssemblyOutput { .. } => {
            encoded_assembly::value(view, value.kind)?
        }
        s::ValueKind::Read(place) => {
            Rvalue::Read(regional_records::place(view, place, &mut work.steps)?)
        }
        s::ValueKind::Borrow(place) => {
            regional_records::borrow(view, place, BorrowKind::Immediate, &mut work.steps)?
        }
        s::ValueKind::TwoPhaseBorrow(place) => {
            regional_records::borrow(view, place, BorrowKind::TwoPhase, &mut work.steps)?
        }
        s::ValueKind::Constrain(value) => Rvalue::Constrain(ValueId(value)),
        s::ValueKind::Reify(value) if view.rows().descriptor().profile().body_nodes() => {
            Rvalue::Reify(ValueId(value))
        }
        s::ValueKind::Constant(_) | s::ValueKind::AssociatedConstant { .. } => {
            constant_operand(view, value.kind, &mut work.steps)?
        }
        s::ValueKind::ConstArgument(slot) => {
            Rvalue::ConstArgument(view.const_argument(slot, &mut work.steps)?)
        }
        s::ValueKind::Binary { .. } => binary(view, value.kind)?,
        s::ValueKind::ScalarCast(_) | s::ValueKind::PointerCast(_) => {
            let (kind, operand) = cast(view, value)?;
            Rvalue::Cast {
                kind,
                operand,
                direct_variant: false,
            }
        }
        s::ValueKind::Function(application) if v4(view) => {
            Rvalue::Function(view.function(application, &mut work.steps)?)
        }
        s::ValueKind::Call {
            signature,
            callee,
            arguments,
        } if v4(view) => Rvalue::Call {
            signature: view.ty(signature, &mut work.steps)?,
            callee: ValueId(callee),
            arguments: crate::body::ir::ValueList {
                start: arguments.start,
                end: arguments.end,
            },
        },
        // The consumer's validation re-proved the pointer signature and the
        // call's lexical safety; local lowering calls the pointer value.
        s::ValueKind::PointerCall {
            signature,
            callee,
            arguments,
            ..
        } if view.rows().descriptor().profile().body_nodes() => Rvalue::Call {
            signature: view.ty(signature, &mut work.steps)?,
            callee: ValueId(callee),
            arguments: crate::body::ir::ValueList {
                start: arguments.start,
                end: arguments.end,
            },
        },
        s::ValueKind::Invoke {
            kind,
            signature,
            arguments,
        } if view.rows().descriptor().profile().body_nodes() => Rvalue::Invoke {
            kind,
            signature: view.ty(signature, &mut work.steps)?,
            arguments: crate::body::ir::ValueList {
                start: arguments.start,
                end: arguments.end,
            },
        },
        s::ValueKind::VariantAggregate { variant, fields } => Rvalue::Aggregate {
            variant: Some(view.variant(variant, &mut work.steps)?),
            fields: crate::body::ir::ValueList {
                start: fields.start,
                end: fields.end,
            },
        },
        s::ValueKind::IsVariant { place, variant } => Rvalue::IsVariant {
            place: regional_records::place(view, place, &mut work.steps)?,
            variant: view.variant(variant, &mut work.steps)?,
        },
        s::ValueKind::EnumCast(operand) => Rvalue::Cast {
            kind: crate::sema::model::Cast::EnumToInteger,
            operand: ValueId(operand),
            direct_variant: false,
        },
        kind => extended_value_kind(view, kind, &mut work.steps)?,
    })
}
fn extended_value_kind(
    view: &SourceOriginal<'_>,
    kind: s::ValueKind,
    steps: &mut usize,
) -> Result<Rvalue, &'static str> {
    if !view.rows().descriptor().profile().declaration_policies() {
        return Err("source value operation requires its wider actual reconstruction profile");
    }
    Ok(match kind {
        s::ValueKind::UniqueBorrow(place) => {
            regional_records::borrow(view, place, BorrowKind::UniqueImmutable, steps)?
        }
        s::ValueKind::DirectEnumCast(operand) => Rvalue::Cast {
            kind: crate::sema::model::Cast::EnumToInteger,
            operand: ValueId(operand),
            direct_variant: true,
        },
        s::ValueKind::AddressCast { kind, operand } => Rvalue::Cast {
            kind: address_cast_kind(kind),
            operand: ValueId(operand),
            direct_variant: false,
        },
        s::ValueKind::Unary {
            operator,
            operand,
            literal,
        } => Rvalue::Unary {
            operator: match operator {
                s::UnaryOperator::Negate => crate::syntax::ast::Unary::Negate,
                s::UnaryOperator::Not => crate::syntax::ast::Unary::Not,
            },
            operand: ValueId(operand),
            literal,
        },
        s::ValueKind::Address(place) => {
            Rvalue::Address(regional_records::place(view, place, steps)?)
        }
        s::ValueKind::Length(place) => Rvalue::Length(regional_records::place(view, place, steps)?),
        s::ValueKind::Validate(place) => {
            Rvalue::Validate(regional_records::place(view, place, steps)?)
        }
        s::ValueKind::TextEquals { place, .. } => {
            encoded_texts::append_equals(view, regional_records::place(view, place, steps)?)?
        }
        s::ValueKind::Union {
            owner,
            ordinal,
            value,
        } => Rvalue::Union {
            field: view.nominal_field(owner, ordinal, steps)?,
            value: ValueId(value),
        },
        s::ValueKind::Repeat { value, constant } => Rvalue::Repeat {
            value: ValueId(value),
            mode: if constant {
                crate::body::ir::RepeatMode::Constant
            } else {
                crate::body::ir::RepeatMode::Once
            },
        },
        s::ValueKind::Unsize { value, conversion } => Rvalue::Unsize {
            value: ValueId(value),
            conversion: unsize_conversion(view, conversion, steps)?,
        },
        s::ValueKind::InlineConstant(body) => {
            Rvalue::InlineConstant(view.anonymous_body(body, steps)?)
        }
        _ => return Err("source value operation requires its wider actual reconstruction profile"),
    })
}
fn unsize_conversion(
    view: &SourceOriginal<'_>,
    conversion: s::UnsizeConversion,
    steps: &mut usize,
) -> Result<crate::types::unsizing::Conversion<crate::types::TypeId>, &'static str> {
    use crate::types::unsizing::Conversion;
    Ok(match conversion {
        s::UnsizeConversion::Slice(array) => Conversion::Slice(view.ty(array, steps)?),
        s::UnsizeConversion::Object { concrete, target } => Conversion::Object {
            concrete: view.ty(concrete, steps)?,
            target: view.ty(target, steps)?,
        },
        s::UnsizeConversion::Upcast { source, target } => Conversion::Upcast {
            source: view.ty(source, steps)?,
            target: view.ty(target, steps)?,
        },
        s::UnsizeConversion::Nominal { source, target } => Conversion::Nominal {
            source: view.ty(source, steps)?,
            target: view.ty(target, steps)?,
        },
        s::UnsizeConversion::Bound { source, target } => Conversion::Bound {
            source: view.ty(source, steps)?,
            target: view.ty(target, steps)?,
        },
    })
}
fn action(
    view: &SourceOriginal<'_>,
    record: s::Action,
    steps: &mut usize,
) -> Result<Action, &'static str> {
    Ok(match record {
        s::Action::StorageLive(id) => Action::StorageLive(LocalId(id)),
        s::Action::StorageDead(id) => Action::StorageDead(LocalId(id)),
        s::Action::StorageEnd { first, last } => Action::StorageEnd {
            first: LocalId(first),
            last: LocalId(last),
        },
        s::Action::Evaluate(id) => Action::Evaluate(ValueId(id)),
        s::Action::Store { place, value } => Action::Store {
            place: regional_records::place(view, place, steps)?,
            value: ValueId(value),
        },
        s::Action::CheckInitialized(place) => {
            Action::CheckInitialized(regional_records::place(view, place, steps)?)
        }
        s::Action::Mention(place) => Action::Mention(regional_records::place(view, place, steps)?),
        s::Action::Project(index) => regional_records::project(view, index, steps)?,
        s::Action::GuardStart(guard) if guards(view) => Action::GuardStart(GuardId(guard)),
        s::Action::GuardUse(guard) if guards(view) => Action::GuardUse(GuardId(guard)),
        s::Action::BindView { view: local, place } if guards(view) => Action::BindView {
            view: LocalId(local),
            place: regional_records::place(view, place, steps)?,
        },
        s::Action::Select { projection, anchor } if guards(view) => {
            let Action::Project(projection) = regional_records::project(view, projection, steps)?
            else {
                return Err("source selected pattern place changed its actual kind");
            };
            Action::Select {
                projection,
                anchor: regional_records::place(view, anchor, steps)?,
            }
        }
        s::Action::GuardStart(_)
        | s::Action::GuardUse(_)
        | s::Action::BindView { .. }
        | s::Action::Select { .. } => {
            return Err("source guard facts require their version-thirteen reconstruction");
        }
    })
}
fn guards(view: &SourceOriginal<'_>) -> bool {
    view.rows().descriptor().profile().body_nodes()
}
/// Version thirteen: the guard table as recorded. Its ranges were checked by
/// the consumer's validation; the ownership checker re-proves protections.
fn guards_of(
    view: &SourceOriginal<'_>,
    steps: &mut usize,
) -> Result<(Vec<crate::body::ir::Guard>, Vec<crate::body::ir::Place>), &'static str> {
    let Some(raw) = view.raw().guards.as_deref() else {
        return Ok((Vec::new(), Vec::new()));
    };
    if !guards(view) {
        return Err("source guard facts require their version-thirteen reconstruction");
    }
    let rows = raw
        .rows
        .iter()
        .map(|guard| crate::body::ir::Guard {
            tested_start: guard.tested.start,
            tested_end: guard.tested.end,
            views_start: guard.views.start,
            views_end: guard.views.end,
        })
        .collect();
    let places = raw
        .places
        .iter()
        .map(|&place| regional_records::place(view, place, steps))
        .collect::<Result<_, _>>()?;
    Ok((rows, places))
}
fn guards_into(
    view: &SourceOriginal<'_>,
    body: &mut Body,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    if let Some(raw) = view.raw().guards.as_deref() {
        work.charge(raw.rows.len().saturating_add(raw.places.len()))?;
    }
    (body.guards, body.guard_places) = guards_of(view, &mut work.steps)?;
    Ok(())
}
fn same_guards(
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    let (rows, places) = guards_of(view, steps)?;
    Ok(places == body.guard_places
        && rows.len() == body.guards.len()
        && rows.iter().zip(&body.guards).all(|(a, b)| {
            (a.tested_start, a.tested_end, a.views_start, a.views_end)
                == (b.tested_start, b.tested_end, b.views_start, b.views_end)
        }))
}
fn cast(
    view: &SourceOriginal<'_>,
    value: s::Value,
) -> Result<(crate::sema::model::Cast, ValueId), &'static str> {
    if let s::ValueKind::PointerCast(operand) = value.kind {
        if !view.rows().descriptor().profile().declaration_policies() {
            return Err("source pointer cast requires its actual version-fourteen descriptor");
        }
        // The sealed body has checked the actual pointee and mutability.
        return Ok((crate::sema::model::Cast::Pointer, ValueId(operand)));
    }
    if !view.rows().descriptor().profile().conversion_roles() {
        return Err("source primitive cast requires its actual version-nine descriptor");
    }
    let s::ValueKind::ScalarCast(operand) = value.kind else {
        return Err("source cast reconstruction belongs to another operation");
    };
    let (from, to) = value.scalar_cast_types(view.raw(), &view.rows().types)?;
    let kind = crate::sema::crates::source_scalar_cast(from, to)
        .ok_or("source scalar cast has no ordinary primitive conversion")?;
    Ok((kind, ValueId(operand)))
}
fn address_cast_kind(kind: s::AddressCast) -> crate::sema::model::Cast {
    use crate::sema::model::Cast;
    match kind {
        s::AddressCast::PointerThin => Cast::PointerThin,
        s::AddressCast::PointerToInteger => Cast::PointerToInteger,
        s::AddressCast::IntegerToPointer => Cast::IntegerToPointer,
        s::AddressCast::FunctionPointer => Cast::FunctionPointer,
    }
}
/// Rebuild the primitive operator the validated row names; its types were
/// re-proved by the consumer's row validation, not taken from the provider.
fn binary(view: &SourceOriginal<'_>, kind: s::ValueKind) -> Result<Rvalue, &'static str> {
    use crate::{body::ir::BinaryOperands as Form, lex::token::Punctuation as P};
    let s::ValueKind::Binary {
        operator,
        operands,
        left,
        right,
    } = kind
    else {
        return Err("source binary reconstruction belongs to another operation");
    };
    if !view.rows().descriptor().profile().body_nodes() {
        return Err("source Binary operations require the version-thirteen body IR profile");
    }
    let operator = match operator {
        s::BinaryOperator::Equal => P::EqualEqual,
        s::BinaryOperator::NotEqual => P::BangEqual,
        s::BinaryOperator::Less => P::Less,
        s::BinaryOperator::LessEqual => P::LessEqual,
        s::BinaryOperator::Greater => P::Greater,
        s::BinaryOperator::GreaterEqual => P::GreaterEqual,
        s::BinaryOperator::BitAnd => P::And,
        s::BinaryOperator::BitOr => P::Or,
        s::BinaryOperator::BitXor => P::Caret,
        s::BinaryOperator::Add => P::Plus,
        s::BinaryOperator::Sub => P::Minus,
        s::BinaryOperator::Mul => P::Star,
        s::BinaryOperator::Div => P::Slash,
        s::BinaryOperator::Rem => P::Percent,
        s::BinaryOperator::Shl => P::ShiftLeft,
        s::BinaryOperator::Shr => P::ShiftRight,
    };
    let operands = match operands {
        s::OperandForm::NotRequired => Form::NotRequired,
        s::OperandForm::Neither => Form::Neither,
        s::OperandForm::Left => Form::Left,
        s::OperandForm::Right => Form::Right,
        s::OperandForm::Both => Form::Both,
    };
    Ok(Rvalue::Binary {
        operands,
        operator,
        left: ValueId(left),
        right: ValueId(right),
    })
}
/// Any other pairing is a changed operation; a Binary row must rebuild exactly.
fn same_binary(
    view: &SourceOriginal<'_>,
    actual: Rvalue,
    raw: s::ValueKind,
) -> Result<bool, &'static str> {
    let fields = |value| match value {
        Rvalue::Binary {
            operands,
            operator,
            left,
            right,
        } => Some((operands, operator, left, right)),
        _ => None,
    };
    if !matches!(raw, s::ValueKind::Binary { .. }) || fields(actual).is_none() {
        return Ok(false);
    }
    Ok(fields(actual) == fields(binary(view, raw)?))
}
fn owned(
    view: &SourceOriginal<'_>,
    body: &mut Body,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    let Some(record) = &view.raw().owned else {
        return Ok(());
    };
    let mut owned = values::Facts::default();
    for creation in &record.created {
        work.charge(1)?;
        owned.insert(lexical::temporary(creation, &mut |index| {
            view.site(index, &mut work.steps)
        })?)?;
    }
    for release in &record.returned {
        work.charge(1)?;
        owned.returned.push(values::Release {
            value: ValueId(release.value),
            location: lexical::location(release.location),
            exits_before: release.exits_before,
        });
    }
    body.owned = Some(Box::new(owned));
    Ok(())
}
pub(super) fn allocation(
    record: &s::Original,
    work: &mut ImportedOriginals,
) -> Result<(usize, usize), &'static str> {
    use crate::body::{ir, scopes, storage};
    work.charge(record.values.len())?;
    let (scalars, associated) = value_counts(record);
    let bytes = regional_records::allocation(record)
        .saturating_add(encoded_assembly::allocation(record))
        .saturating_add(
            scalars
                .saturating_mul(size_of::<crate::sema::model::Scalar>())
                .saturating_add(
                    record
                        .locals
                        .len()
                        .saturating_mul(size_of::<ir::Local>())
                        .saturating_add(record.values.len().saturating_mul(size_of::<ir::Value>()))
                        .saturating_add(
                            record
                                .statements
                                .len()
                                .saturating_mul(size_of::<ir::Statement>()),
                        )
                        .saturating_add(record.blocks.len().saturating_mul(size_of::<ir::Block>()))
                        .saturating_add(
                            record
                                .scopes
                                .len()
                                .saturating_mul(size_of::<storage::Scope>()),
                        )
                        .saturating_add(
                            record
                                .exits
                                .len()
                                .saturating_mul(size_of::<storage::Exit>()),
                        )
                        .saturating_add(
                            record
                                .temporaries
                                .len()
                                .saturating_mul(size_of::<ir::Temporary>()),
                        )
                        .saturating_add(record.owned.as_ref().map_or(0, |owned| {
                            size_of::<values::Facts>()
                                .saturating_add(
                                    owned
                                        .created
                                        .len()
                                        .saturating_mul(size_of::<ir::Temporary>()),
                                )
                                .saturating_add(
                                    owned
                                        .returned
                                        .len()
                                        .saturating_mul(size_of::<values::Release>()),
                                )
                                .saturating_add(crate::support::retained::new_map::<
                                    ir::ValueId,
                                    usize,
                                >(
                                    owned.created.len()
                                ))
                        }))
                        // Keep alignment overhead conservative without flattening any owner graph.
                        .saturating_add(
                            record
                                .arguments
                                .len()
                                .saturating_mul(size_of::<ir::ValueId>()),
                        )
                        .saturating_add(
                            size_of::<Body>()
                                + size_of::<storage::Facts>()
                                + size_of::<scopes::Id>(),
                        ),
                ),
        );
    Ok((bytes, associated))
}

fn value_counts(record: &s::Original) -> (usize, usize) {
    record
        .values
        .iter()
        .fold((0, 0), |(scalars, associated), value| match value.kind {
            s::ValueKind::Scalar(_) => (scalars + 1, associated),
            s::ValueKind::AssociatedConstant { .. } => (scalars, associated + 1),
            _ => (scalars, associated),
        })
}

pub(super) fn validate(
    facts: &crate::sema::model::Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let record = view.raw();
    if body.source != view.source()?
        || body.const_context != view.const_context()
        || body.instance.is_some()
        || body.owner != owner(view)
        || body.result != view.ty(record.result, steps)?
        || body.locals.len() != record.locals.len()
        || body.values.len() != record.values.len()
        || body.statements.len() != record.statements.len()
        || body.blocks.len() != record.blocks.len()
        || body.temporaries.len() != record.temporaries.len()
        || body.static_types != regional_records::static_types(view, steps)?
        || body.static_permissions.is_some()
        || !body.has_only_caller_sites()
        || body.closure_calls.is_some()
        || !same_guards(view, body, steps)?
        || body.arguments.len() != record.arguments.len()
        || body
            .arguments
            .iter()
            .zip(&record.arguments)
            .any(|(actual, raw)| actual.0 != *raw)
        || (!view.rows().descriptor().profile().declaration_policies()
            && body
                .promotions
                .as_ref()
                .is_some_and(|facts| !facts.is_empty()))
    {
        return Err("source original changed its complete supported body/header domain");
    }
    regional_records::validate(view, body, steps)?;
    ports(facts, view, body, steps)?;
    encoded_texts::validate(&facts.texts, view, body, steps)?;
    super::callsite_coordinates::validate(facts, view, body, steps)?;
    encoded_assembly::validate(view, body, steps)?;
    for (actual, record) in body.statements.iter().zip(&record.statements) {
        *steps = steps.saturating_add(1);
        if actual.source != view.site(record.site, steps)?
            || !same_action(actual.action, action(view, record.action, steps)?)
        {
            return Err("source original changed an actual statement/site");
        }
    }
    let branches = view.rows().descriptor().profile().branches();
    for (actual, record) in body.blocks.iter().zip(&record.blocks) {
        use crate::{body::ir::Terminator as T, support::crates::generics::Terminator as R};
        *steps = steps.saturating_add(1);
        let same = match (actual.terminator, record.terminator) {
            (T::Return(a), R::Return(b)) => a.0 == b,
            (T::Jump(a), R::Jump(b)) => a.0 == b,
            (T::Unreachable, R::Unreachable) => true,
            (
                T::Branch { condition, yes, no },
                R::Branch {
                    condition: c,
                    yes: y,
                    no: n,
                },
            ) => branches && condition.0 == c && yes.0 == y && no.0 == n,
            (T::Panic, R::Panic) => branches,
            (
                T::FalseEdge { real, imaginary },
                R::FalseEdge {
                    real: r,
                    imaginary: i,
                },
            ) => branches && real.0 == r && imaginary.0 == i,
            _ => false,
        };
        if !same
            || actual.start != record.statements.start
            || actual.end != record.statements.end
            || actual.source != view.site(record.site, steps)?
        {
            return Err("source original changed an actual CFG block/edge");
        }
    }
    storage(view, body, steps)?;
    for (actual, record) in body.temporaries.iter().zip(&record.temporaries) {
        let expected = lexical::temporary(record, &mut |index| view.site(index, steps))?;
        if !same_temporary(actual, &expected) {
            return Err("source original changed a scoped temporary");
        }
    }
    validate_owned(view, body, steps)
}
fn ports(
    facts: &crate::sema::model::Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let record = view.raw();
    for (actual, record) in body.locals.iter().zip(&record.locals) {
        *steps = steps.saturating_add(1);
        let kind = match record.kind {
            crate::support::crates::generics::LocalKind::Parameter(index) => {
                crate::body::ir::LocalKind::Parameter(index)
            }
            crate::support::crates::generics::LocalKind::Binding => {
                crate::body::ir::LocalKind::Binding
            }
            crate::support::crates::generics::LocalKind::Merge => crate::body::ir::LocalKind::Merge,
            crate::support::crates::generics::LocalKind::Guard(guard) => {
                crate::body::ir::LocalKind::Guard(GuardId(guard))
            }
        };
        if actual.ty != view.ty(record.ty, steps)?
            || actual.source != view.site(record.site, steps)?
            || actual.kind != kind
            || actual.scope.0 != record.scope
            || actual.storage_parent.map(|id| id.0) != record.storage_parent
            || actual.definition
                != record
                    .definition
                    .map(|id| view.declaration(id, steps))
                    .transpose()?
            || actual.mutable != record.mutable
        {
            return Err("source original changed an actual typed local/binding/scope");
        }
    }
    let mut scalar = 0;
    for (actual, record) in body.values.iter().zip(&record.values) {
        *steps = steps.saturating_add(1);
        let same = same_value(facts, view, (actual, *record), body, &mut scalar, steps)?;
        if !same || actual.ty != view.ty(record.ty, steps)? {
            return Err("source original changed an actual typed value operation");
        }
    }
    if scalar != body.scalars.len() {
        return Err("source original changed its complete scalar storage domain");
    }
    Ok(())
}
/// One rebuilt value against its record; scalars advance their shared cursor.
/// The version-thirteen enum operations; `None` for every other form.
fn same_variant_value(
    view: &SourceOriginal<'_>,
    (actual, record): (Rvalue, s::ValueKind),
    steps: &mut usize,
) -> Result<Option<bool>, &'static str> {
    Ok(Some(match (actual, record) {
        (
            Rvalue::Aggregate {
                variant: Some(actual),
                fields,
            },
            s::ValueKind::VariantAggregate {
                variant,
                fields: raw,
            },
        ) => {
            actual == view.variant(variant, steps)?
                && fields.start == raw.start
                && fields.end == raw.end
        }
        (
            Rvalue::IsVariant { place, variant },
            s::ValueKind::IsVariant {
                place: raw,
                variant: expected,
            },
        ) => {
            place == regional_records::place(view, raw, steps)?
                && variant == view.variant(expected, steps)?
        }
        (
            Rvalue::Cast {
                kind: crate::sema::model::Cast::EnumToInteger,
                operand,
                direct_variant: false,
            },
            s::ValueKind::EnumCast(raw),
        ) => operand.0 == raw,
        (
            _,
            s::ValueKind::VariantAggregate { .. }
            | s::ValueKind::IsVariant { .. }
            | s::ValueKind::EnumCast(_),
        ) => false,
        _ => return Ok(None),
    }))
}
/// Direct calls, calls through a fn pointer and call operators keep their
/// signature, operands and argument range.
fn same_invocation(
    view: &SourceOriginal<'_>,
    (actual, record): (Rvalue, s::ValueKind),
    steps: &mut usize,
) -> Result<Option<bool>, &'static str> {
    Ok(Some(match (actual, record) {
        (
            Rvalue::Call {
                signature,
                callee,
                arguments,
            },
            s::ValueKind::Call {
                signature: raw,
                callee: c,
                arguments: a,
            },
        ) if v4(view) => {
            signature == view.ty(raw, steps)?
                && callee.0 == c
                && arguments.start == a.start
                && arguments.end == a.end
        }
        (
            Rvalue::Call {
                signature,
                callee,
                arguments,
            },
            s::ValueKind::PointerCall {
                signature: raw,
                callee: c,
                arguments: a,
                ..
            },
        ) if view.rows().descriptor().profile().body_nodes() => {
            signature == view.ty(raw, steps)?
                && callee.0 == c
                && arguments.start == a.start
                && arguments.end == a.end
        }
        (
            Rvalue::Invoke {
                kind,
                signature,
                arguments,
            },
            s::ValueKind::Invoke {
                kind: k,
                signature: raw,
                arguments: a,
            },
        ) if view.rows().descriptor().profile().body_nodes() => {
            kind == k
                && signature == view.ty(raw, steps)?
                && arguments.start == a.start
                && arguments.end == a.end
        }
        _ => return Ok(None),
    }))
}
fn same_extended_value(
    facts: &crate::sema::model::Declarations,
    view: &SourceOriginal<'_>,
    (actual, record): (Rvalue, s::ValueKind),
    steps: &mut usize,
) -> Result<Option<bool>, &'static str> {
    if !matches!(
        record,
        s::ValueKind::DirectEnumCast(_)
            | s::ValueKind::AddressCast { .. }
            | s::ValueKind::Unary { .. }
            | s::ValueKind::Address(_)
            | s::ValueKind::Length(_)
            | s::ValueKind::Validate(_)
            | s::ValueKind::TextEquals { .. }
            | s::ValueKind::Union { .. }
            | s::ValueKind::Repeat { .. }
            | s::ValueKind::Unsize { .. }
            | s::ValueKind::InlineConstant(_)
    ) {
        return Ok(None);
    }
    if !view.rows().descriptor().profile().declaration_policies() {
        return Ok(Some(false));
    }
    Ok(Some(match (actual, record) {
        (
            Rvalue::Cast {
                kind: crate::sema::model::Cast::EnumToInteger,
                operand,
                direct_variant: true,
            },
            s::ValueKind::DirectEnumCast(raw),
        ) => operand.0 == raw,
        (
            Rvalue::Cast {
                kind,
                operand,
                direct_variant: false,
            },
            s::ValueKind::AddressCast {
                kind: raw_kind,
                operand: raw_operand,
            },
        ) => kind == address_cast_kind(raw_kind) && operand.0 == raw_operand,
        (
            Rvalue::Unary {
                operator,
                operand,
                literal,
            },
            s::ValueKind::Unary {
                operator: raw_op,
                operand: raw_operand,
                literal: raw_lit,
            },
        ) => {
            let expected = match raw_op {
                s::UnaryOperator::Negate => crate::syntax::ast::Unary::Negate,
                s::UnaryOperator::Not => crate::syntax::ast::Unary::Not,
            };
            operator == expected && operand.0 == raw_operand && literal == raw_lit
        }
        (Rvalue::Address(place), s::ValueKind::Address(raw))
        | (Rvalue::Length(place), s::ValueKind::Length(raw))
        | (Rvalue::Validate(place), s::ValueKind::Validate(raw))
        | (Rvalue::TextEquals { place, .. }, s::ValueKind::TextEquals { place: raw, .. }) => {
            place == regional_records::place(view, raw, steps)?
        }
        (
            Rvalue::Union { field, value },
            s::ValueKind::Union {
                owner,
                ordinal,
                value: raw_value,
            },
        ) => field == view.nominal_field(owner, ordinal, steps)? && value.0 == raw_value,
        (
            Rvalue::Repeat { value, mode },
            s::ValueKind::Repeat {
                value: raw_value,
                constant,
            },
        ) => value.0 == raw_value && (mode == crate::body::ir::RepeatMode::Constant) == constant,
        (
            Rvalue::Unsize { value, conversion },
            s::ValueKind::Unsize {
                value: raw_value,
                conversion: raw_conversion,
            },
        ) => value.0 == raw_value && conversion == unsize_conversion(view, raw_conversion, steps)?,
        (Rvalue::InlineConstant(actual), s::ValueKind::InlineConstant(raw)) => {
            actual == view.anonymous_body(raw, steps)?
        }
        (Rvalue::InlineInstance(instance), s::ValueKind::InlineConstant(raw)) => {
            instance.origin(facts) == Some(view.anonymous_body(raw, steps)?)
        }
        _ => false,
    }))
}
fn same_value(
    facts: &crate::sema::model::Declarations,
    view: &SourceOriginal<'_>,
    (actual, record): (&Value, s::Value),
    body: &Body,
    scalar: &mut usize,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    if let Some(same) = same_variant_value(view, (actual.kind, record.kind), steps)? {
        return Ok(same);
    }
    if let Some(same) = encoded_assembly::same(actual.kind, record.kind) {
        return Ok(same);
    }
    if let Some(same) = same_invocation(view, (actual.kind, record.kind), steps)? {
        return Ok(same);
    }
    if let Some(same) = same_extended_value(facts, view, (actual.kind, record.kind), steps)? {
        return Ok(same);
    }
    Ok(match (actual.kind, record.kind) {
        // Each literal and its ordinal are compared with the arena afterwards.
        (Rvalue::Unit, s::ValueKind::Unit) | (Rvalue::Text(_), s::ValueKind::Text(_)) => true,
        (
            Rvalue::Aggregate {
                variant: actual,
                fields,
            },
            s::ValueKind::Aggregate { fields: raw },
        ) => {
            actual == view.aggregate_variant(record.ty, steps)?
                && fields.start == raw.start
                && fields.end == raw.end
        }
        (Rvalue::Scalar(id), s::ValueKind::Scalar(_)) => {
            encoded_literals::same(view, record, id, body, scalar)?
        }
        (Rvalue::Read(actual), s::ValueKind::Read(record)) => {
            actual == regional_records::place(view, record, steps)?
        }
        (
            Rvalue::Borrow {
                place,
                kind: BorrowKind::Immediate,
            },
            s::ValueKind::Borrow(raw),
        )
        | (
            Rvalue::Borrow {
                place,
                kind: BorrowKind::TwoPhase,
            },
            s::ValueKind::TwoPhaseBorrow(raw),
        ) => {
            regional_records::require(view)?;
            place == regional_records::place(view, raw, steps)?
        }
        (
            Rvalue::Borrow {
                place,
                kind: BorrowKind::UniqueImmutable,
            },
            s::ValueKind::UniqueBorrow(raw),
        ) if view.rows().descriptor().profile().declaration_policies() => {
            regional_records::require(view)?;
            place == regional_records::place(view, raw, steps)?
        }
        (
            Rvalue::Cast {
                kind,
                operand,
                direct_variant: false,
            },
            s::ValueKind::ScalarCast(_) | s::ValueKind::PointerCast(_),
        ) => (kind, operand) == cast(view, record)?,
        (Rvalue::Constrain(actual), s::ValueKind::Constrain(record)) => actual.0 == record,
        (Rvalue::Reify(actual), s::ValueKind::Reify(record))
            if view.rows().descriptor().profile().body_nodes() =>
        {
            actual.0 == record
        }
        (
            actual @ (Rvalue::Constant(_) | Rvalue::AssociatedConstant(_)),
            s::ValueKind::Constant(record),
        ) if view.rows().descriptor().profile().body_nodes() => {
            match (actual, constant_read(view, record, steps)?) {
                (Rvalue::Constant(actual), Rvalue::Constant(expected)) => actual == expected,
                (Rvalue::AssociatedConstant(actual), Rvalue::AssociatedConstant(expected)) => {
                    actual == expected
                }
                _ => false,
            }
        }
        (Rvalue::Function(actual), s::ValueKind::Function(raw)) if v4(view) => {
            actual == view.function(raw, steps)?
        }
        (Rvalue::ConstArgument(actual), s::ValueKind::ConstArgument(slot)) => {
            actual == view.const_argument(slot, steps)?
        }
        (
            Rvalue::AssociatedConstant(actual),
            s::ValueKind::AssociatedConstant {
                declaration,
                arguments,
            },
        ) => actual == view.associated_operand(declaration, arguments, steps)?,
        (actual, raw) => same_binary(view, actual, raw)?,
    })
}

fn constant_operand(
    view: &SourceOriginal<'_>,
    kind: s::ValueKind,
    steps: &mut usize,
) -> Result<Rvalue, &'static str> {
    match kind {
        s::ValueKind::Constant(declaration) if view.rows().descriptor().profile().body_nodes() => {
            constant_read(view, declaration, steps)
        }
        s::ValueKind::AssociatedConstant {
            declaration,
            arguments,
        } => Ok(Rvalue::AssociatedConstant(view.associated_operand(
            declaration,
            arguments,
            steps,
        )?)),
        _ => Err("source value operation requires its wider actual reconstruction profile"),
    }
}

fn constant_read(
    view: &SourceOriginal<'_>,
    declaration: crate::support::crates::owner_catalog::DeclarationRef,
    steps: &mut usize,
) -> Result<Rvalue, &'static str> {
    let definition = view.constant(declaration, steps)?;
    let rows = view.rows();
    if rows.descriptor().profile().declaration_policies()
        && (matches!(rows.header(declaration)?, Some(s::HeaderRef::Constant(index))
            if matches!(rows.constants()[index as usize].body, s::ConstantBody::Original(_)))
            || rows
                .declarations
                .get(declaration.0 as usize)
                .and_then(|row| row.definition.parent)
                .and_then(|parent| rows.header(parent).ok().flatten())
                .is_some_and(|parent| match parent {
                    s::HeaderRef::Implementation(index) => {
                        rows.implementations[index as usize].inherent_constant_owner()
                    }
                    _ => false,
                }))
    {
        return Ok(Rvalue::AssociatedConstant(crate::sema::values::Instance {
            definition,
            arguments: crate::types::GenericArgs::EMPTY,
        }));
    }
    Ok(Rvalue::Constant(definition))
}
fn same_action(a: Action, b: Action) -> bool {
    match (a, b) {
        (Action::StorageLive(a), Action::StorageLive(b))
        | (Action::StorageDead(a), Action::StorageDead(b)) => a == b,
        (Action::StorageEnd { first: a, last: b }, Action::StorageEnd { first: x, last: y }) => {
            a == x && b == y
        }
        (Action::Evaluate(a), Action::Evaluate(b)) => a == b,
        (Action::Project(a), Action::Project(b)) => a == b,
        (Action::Store { place: a, value: b }, Action::Store { place: x, value: y }) => {
            a == x && b == y
        }
        (Action::CheckInitialized(a), Action::CheckInitialized(b))
        | (Action::Mention(a), Action::Mention(b)) => a == b,
        (Action::GuardStart(a), Action::GuardStart(b))
        | (Action::GuardUse(a), Action::GuardUse(b)) => a == b,
        (Action::BindView { view: a, place: b }, Action::BindView { view: x, place: y }) => {
            a == x && b == y
        }
        (
            Action::Select {
                projection: a,
                anchor: b,
            },
            Action::Select {
                projection: x,
                anchor: y,
            },
        ) => a == x && b == y,
        _ => false,
    }
}
fn storage(view: &SourceOriginal<'_>, body: &Body, steps: &mut usize) -> Result<(), &'static str> {
    use crate::{
        body::{scopes, storage},
        support::crates::generics as g,
    };
    let actual = body
        .storage
        .as_ref()
        .ok_or("source original omitted its actual storage boundaries")?;
    let record = view.raw();
    if actual.scopes.len() != record.scopes.len() || actual.exits.len() != record.exits.len() {
        return Err("source original changed its complete lexical/storage domain");
    }
    for (actual, record) in actual.scopes.iter().zip(&record.scopes) {
        *steps = steps.saturating_add(1);
        if actual.parent.map(|id| id.0) != record.parent
            || actual.source != view.site(record.site, steps)?
            || actual.kind != lexical::scope_kind(record.kind)
        {
            return Err("source original changed a lexical scope");
        }
    }
    for (actual, record) in actual.exits.iter().zip(&record.exits) {
        *steps = steps.saturating_add(1);
        let end = match record.end {
            g::End::Scopes { inner, outer } => storage::End::Scopes(storage::Range {
                inner: scopes::Id(inner),
                outer: outer.map(scopes::Id),
            }),
            g::End::Temporaries(r) => storage::End::Temporaries {
                start: r.start,
                end: r.end,
            },
        };
        if actual.location != lexical::location(record.location) || actual.end != end {
            return Err("source original changed a storage exit/order");
        }
    }
    Ok(())
}
fn validate_owned(
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    match (&view.raw().owned, &body.owned) {
        (None, None) => Ok(()),
        (Some(record), Some(actual)) => {
            if record.created.len() != actual.created.len()
                || record.returned.len() != actual.returned.len()
                || !actual.has_exact_index(steps)
            {
                return Err("source original changed its complete owned-value coverage");
            }
            for (record, actual) in record.created.iter().zip(&actual.created) {
                *steps = steps.saturating_add(1);
                let expected = lexical::temporary(record, &mut |index| view.site(index, steps))?;
                if !same_temporary(actual, &expected) {
                    return Err("source original changed an owned-value creation");
                }
            }
            for (record, actual) in record.returned.iter().zip(&actual.returned) {
                *steps = steps.saturating_add(1);
                if actual.value.0 != record.value
                    || actual.location != lexical::location(record.location)
                    || actual.exits_before != record.exits_before
                {
                    return Err("source original changed an owned-value transfer/exit order");
                }
            }
            Ok(())
        }
        _ => Err("source original changed absent-versus-present owned-value records"),
    }
}
fn same_temporary(a: &crate::body::ir::Temporary, b: &crate::body::ir::Temporary) -> bool {
    a.lifetime == b.lifetime
        && a.value == b.value
        && a.source == b.source
        && a.block == b.block
        && a.before == b.before
        && a.exits_before == b.exits_before
}

fn v4(view: &SourceOriginal<'_>) -> bool {
    view.rows().descriptor().profile().feature_predicates()
}
