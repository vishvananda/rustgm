//! Version thirteen encodes body operations as self-delimiting typed IR nodes.
//! Every form of the typed body IR owns a stable code; operands carry their own
//! kind. A consumer maps exactly the vocabulary it can rebuild and rejects every
//! other allocated form by name, so no form is skipped, erased or guessed.
use super::{DeclarationRef, Input, Writer, g, id, range, s};

const MAX_ARITY: usize = 6;

/// One typed operand; each kind has one fixed payload width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operand {
    Value(u32),
    Block(u32),
    Local(u32),
    Projection(u32),
    Type(u32),
    Declaration(u32),
    Application(u32),
    Region(u32),
    Flag(u32),
    Slot(s::SlotRef),
    Range(g::Range),
    Bits([u8; 16]),
    Place(s::Place),
}

/// Stable value-form codes, in typed body IR order.
const VALUES: [&str; 32] = [
    "encoded Rvalue::Assembly is not reconstructable by this consumer",
    "encoded Rvalue::AssemblyOutput is not reconstructable by this consumer",
    "encoded Rvalue::Unit is not reconstructable by this consumer",
    "encoded Rvalue::Aggregate is not reconstructable by this consumer",
    "encoded Rvalue::Union is not reconstructable by this consumer",
    "encoded Rvalue::Repeat is not reconstructable by this consumer",
    "encoded Rvalue::Scalar is not reconstructable by this consumer",
    "encoded Rvalue::Text is not reconstructable by this consumer",
    "encoded Rvalue::Length is not reconstructable by this consumer",
    "encoded Rvalue::TextEquals is not reconstructable by this consumer",
    "encoded Rvalue::Read is not reconstructable by this consumer",
    "encoded Rvalue::Address is not reconstructable by this consumer",
    "encoded Rvalue::Borrow is not reconstructable by this consumer",
    "encoded Rvalue::Reborrow is not reconstructable by this consumer",
    "encoded Rvalue::IsVariant is not reconstructable by this consumer",
    "encoded Rvalue::Validate is not reconstructable by this consumer",
    "encoded Rvalue::Function is not reconstructable by this consumer",
    "encoded Rvalue::Constant is not reconstructable by this consumer",
    "encoded Rvalue::AssociatedConstant is not reconstructable by this consumer",
    "encoded Rvalue::SourceAssociated is not reconstructable by this consumer",
    "encoded Rvalue::InlineConstant is not reconstructable by this consumer",
    "encoded Rvalue::InlineInstance is not reconstructable by this consumer",
    "encoded Rvalue::ConstArgument is not reconstructable by this consumer",
    "encoded Rvalue::Reify is not reconstructable by this consumer",
    "encoded Rvalue::Constrain is not reconstructable by this consumer",
    "encoded Rvalue::Unsize is not reconstructable by this consumer",
    "encoded Rvalue::Unary is not reconstructable by this consumer",
    "encoded Rvalue::Binary is not reconstructable by this consumer",
    "encoded Rvalue::Cast is not reconstructable by this consumer",
    "encoded Rvalue::Invoke is not reconstructable by this consumer",
    "encoded Rvalue::Call is not reconstructable by this consumer",
    "encoded Rvalue::Intrinsic is not reconstructable by this consumer",
];
const ACTIONS: [&str; 13] = [
    "encoded Action::CheckInitialized is not reconstructable by this consumer",
    "encoded Action::Mention is not reconstructable by this consumer",
    "encoded Action::GuardStart is not reconstructable by this consumer",
    "encoded Action::GuardUse is not reconstructable by this consumer",
    "encoded Action::BindView is not reconstructable by this consumer",
    "encoded Action::Project is not reconstructable by this consumer",
    "encoded Action::Select is not reconstructable by this consumer",
    "encoded Action::StorageLive is not reconstructable by this consumer",
    "encoded Action::StorageDead is not reconstructable by this consumer",
    "encoded Action::StorageEnd is not reconstructable by this consumer",
    "encoded Action::Evaluate is not reconstructable by this consumer",
    "encoded Action::Store is not reconstructable by this consumer",
    "encoded Action::FutureTake is not reconstructable by this consumer",
];
const TERMINATORS: [&str; 6] = [
    "encoded Terminator::Jump is not reconstructable by this consumer",
    "encoded Terminator::FalseEdge is not reconstructable by this consumer",
    "encoded Terminator::Branch is not reconstructable by this consumer",
    "encoded Terminator::Return is not reconstructable by this consumer",
    "encoded Terminator::Unreachable is not reconstructable by this consumer",
    "encoded Terminator::Panic is not reconstructable by this consumer",
];
const PROJECTIONS: [&str; 13] = [
    "encoded ProjectionKind::CaptureBoundary is not reconstructable by this consumer",
    "encoded ProjectionKind::Dereference is not reconstructable by this consumer",
    "encoded ProjectionKind::RawDereference is not reconstructable by this consumer",
    "encoded ProjectionKind::Field is not reconstructable by this consumer",
    "encoded ProjectionKind::UnionField is not reconstructable by this consumer",
    "encoded ProjectionKind::Index is not reconstructable by this consumer",
    "encoded ProjectionKind::IndexParameter is not reconstructable by this consumer",
    "encoded ProjectionKind::SliceIndex is not reconstructable by this consumer",
    "encoded ProjectionKind::ConstantIndex is not reconstructable by this consumer",
    "encoded ProjectionKind::FromEnd is not reconstructable by this consumer",
    "encoded ProjectionKind::Subslice is not reconstructable by this consumer",
    "encoded ProjectionKind::Subarray is not reconstructable by this consumer",
    "encoded ProjectionKind::FutureField is not reconstructable by this consumer",
];

/// Stable `Rvalue::Binary` operator flags. Division, remainder and shifts require v14.
const OPERATORS: [s::BinaryOperator; 16] = [
    s::BinaryOperator::Equal,
    s::BinaryOperator::NotEqual,
    s::BinaryOperator::Less,
    s::BinaryOperator::LessEqual,
    s::BinaryOperator::Greater,
    s::BinaryOperator::GreaterEqual,
    s::BinaryOperator::BitAnd,
    s::BinaryOperator::BitOr,
    s::BinaryOperator::BitXor,
    s::BinaryOperator::Add,
    s::BinaryOperator::Sub,
    s::BinaryOperator::Mul,
    s::BinaryOperator::Div,
    s::BinaryOperator::Rem,
    s::BinaryOperator::Shl,
    s::BinaryOperator::Shr,
];
/// Last allocated flags reserve Div, Rem, Shl and Shr.
const CHECKED_OPERATORS: u32 = 16;
const FORMS: [s::OperandForm; 5] = [
    s::OperandForm::NotRequired,
    s::OperandForm::Neither,
    s::OperandForm::Left,
    s::OperandForm::Right,
    s::OperandForm::Both,
];

/// Reconstructable codes in this consumer, by family and profile.
const VALUE_FORMS: [u8; 18] = [
    0, 2, 3, 6, 7, 10, 12, 14, 16, 17, 18, 22, 23, 24, 27, 28, 29, 30,
];
const V14_VALUE_FORMS: [u8; 26] = [
    0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 22, 23, 24, 25, 26, 27, 28, 29, 30,
];
/// Stable `Rvalue::Invoke` call-trait flags.
const CALLABLES: [crate::support::call_traits::CallTrait; 3] = [
    crate::support::call_traits::CallTrait::FnOnce,
    crate::support::call_traits::CallTrait::FnMut,
    crate::support::call_traits::CallTrait::Fn,
];
/// Stable lexical unsafe-context flags of a pointer call, as a request's.
const SAFETIES: [s::CallSafety; 3] = [
    s::CallSafety::Safe,
    s::CallSafety::ExplicitUnsafeBlock,
    s::CallSafety::UnsafeFunction,
];
const ACTION_FORMS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const TERMINATOR_FORMS: [u8; 6] = [0, 1, 2, 3, 4, 5];
const PROJECTION_FORMS: [u8; 2] = [1, 3];
const V14_PROJECTION_FORMS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

struct Node {
    code: u8,
    arity: usize,
    operands: [Operand; MAX_ARITY],
}
impl Node {
    fn operands(&self) -> &[Operand] {
        &self.operands[..self.arity]
    }
}

/// A supported code whose operands differ from its schema is malformed; any
/// other allocated code is a known form this consumer cannot rebuild.
fn rejected(
    table: &[&'static str],
    supported: &[u8],
    code: u8,
    malformed: &'static str,
    unknown: &'static str,
) -> &'static str {
    if supported.contains(&code) {
        malformed
    } else {
        table.get(usize::from(code)).copied().unwrap_or(unknown)
    }
}

fn node_write(out: &mut Writer, code: u8, operands: &[Operand]) -> Result<(), &'static str> {
    out.u8(code)?;
    out.u8(u8::try_from(operands.len()).map_err(|_| "body IR node arity overflow")?)?;
    for &operand in operands {
        operand_write(out, operand)?;
    }
    Ok(())
}
fn operand_write(out: &mut Writer, operand: Operand) -> Result<(), &'static str> {
    let (kind, value) = match operand {
        Operand::Value(value) => (0, value),
        Operand::Block(value) => (1, value),
        Operand::Local(value) => (2, value),
        Operand::Projection(value) => (3, value),
        Operand::Type(value) => (4, value),
        Operand::Declaration(value) => (5, value),
        Operand::Application(value) => (6, value),
        Operand::Region(value) => (7, value),
        Operand::Flag(value) => (8, value),
        Operand::Range(value) => {
            out.u8(9)?;
            return range(out, value);
        }
        Operand::Bits(bits) => {
            out.u8(10)?;
            return out.bytes(&bits);
        }
        Operand::Place(s::Place::Local(value)) => (11, value),
        Operand::Place(s::Place::Projection(value)) => (12, value),
        Operand::Place(s::Place::Temporary(value)) => (13, value),
        Operand::Place(s::Place::Static(value)) => (14, value.0),
        Operand::Slot(value) => (15, value.0),
    };
    out.u8(kind)?;
    id(out, value)
}
fn node_read(read: &mut Input<'_, '_>) -> Result<Node, &'static str> {
    let code = read.reader.u8()?;
    let arity = usize::from(read.reader.u8()?);
    if arity > MAX_ARITY {
        return Err("body IR node arity exceeds its stable bound");
    }
    let mut operands = [Operand::Flag(0); MAX_ARITY];
    for slot in &mut operands[..arity] {
        *slot = operand_read(read)?;
    }
    Ok(Node {
        code,
        arity,
        operands,
    })
}
fn operand_read(read: &mut Input<'_, '_>) -> Result<Operand, &'static str> {
    Ok(match read.reader.u8()? {
        0 => Operand::Value(read.id()?),
        1 => Operand::Block(read.id()?),
        2 => Operand::Local(read.id()?),
        3 => Operand::Projection(read.id()?),
        4 => Operand::Type(read.id()?),
        5 => Operand::Declaration(read.id()?),
        6 => Operand::Application(read.id()?),
        7 => Operand::Region(read.id()?),
        8 => Operand::Flag(read.id()?),
        9 => Operand::Range(read.range()?),
        10 => Operand::Bits(read.reader.u128()?.to_le_bytes()),
        11 => Operand::Place(s::Place::Local(read.id()?)),
        12 => Operand::Place(s::Place::Projection(read.id()?)),
        13 => Operand::Place(s::Place::Temporary(read.id()?)),
        14 if read.profile.declaration_policies() => {
            Operand::Place(s::Place::Static(DeclarationRef(read.id()?)))
        }
        14 => return Err("encoded static places are not reconstructable by this consumer"),
        15 if read.profile.declaration_policies() => Operand::Slot(s::SlotRef(read.id()?)),
        _ => return Err("unknown body IR operand kind"),
    })
}

/// A guard's tested place is one place operand.
pub(super) fn place_write(out: &mut Writer, place: s::Place) -> Result<(), &'static str> {
    operand_write(out, Operand::Place(place))
}
pub(super) fn place_read(read: &mut Input<'_, '_>) -> Result<s::Place, &'static str> {
    match operand_read(read)? {
        Operand::Place(place) => Ok(place),
        _ => Err("source guard tested place is not a place operand"),
    }
}
pub(super) fn value_write(out: &mut Writer, row: &s::Value) -> Result<(), &'static str> {
    id(out, row.ty.0)?;
    match row.kind {
        s::ValueKind::Unit => node_write(out, 2, &[]),
        s::ValueKind::Aggregate { fields } => node_write(out, 3, &[Operand::Range(fields)]),
        // A record keeps its single-operand node; an enum names its variant.
        s::ValueKind::VariantAggregate { variant, fields } => node_write(
            out,
            3,
            &[Operand::Range(fields), Operand::Declaration(variant.0)],
        ),
        s::ValueKind::IsVariant { place, variant } => node_write(
            out,
            14,
            &[Operand::Place(place), Operand::Declaration(variant.0)],
        ),
        s::ValueKind::Scalar(bits) => node_write(out, 6, &[Operand::Bits(bits)]),
        s::ValueKind::Text(ordinal) => node_write(out, 7, &[Operand::Flag(ordinal)]),
        s::ValueKind::Read(place) => node_write(out, 10, &[Operand::Place(place)]),
        // Flag zero is the immediate kind; two-phase and unique borrows own 1 and 2.
        s::ValueKind::Borrow(place) => {
            node_write(out, 12, &[Operand::Place(place), Operand::Flag(0)])
        }
        s::ValueKind::TwoPhaseBorrow(place) => {
            node_write(out, 12, &[Operand::Place(place), Operand::Flag(1)])
        }
        s::ValueKind::UniqueBorrow(place) => {
            node_write(out, 12, &[Operand::Place(place), Operand::Flag(2)])
        }
        s::ValueKind::Function(application) => {
            node_write(out, 16, &[Operand::Application(application.0)])
        }
        s::ValueKind::Constant(declaration) => {
            node_write(out, 17, &[Operand::Declaration(declaration.0)])
        }
        s::ValueKind::AssociatedConstant {
            declaration,
            arguments,
        } => node_write(
            out,
            18,
            &[
                Operand::Declaration(declaration.0),
                Operand::Range(arguments),
            ],
        ),
        s::ValueKind::ConstArgument(slot) => node_write(out, 22, &[Operand::Slot(slot)]),
        s::ValueKind::Reify(value) => node_write(out, 23, &[Operand::Value(value)]),
        // The instruction ordinal names the original's assembly column row.
        s::ValueKind::Assembly {
            instruction,
            arguments,
        } => node_write(
            out,
            0,
            &[Operand::Flag(instruction), Operand::Range(arguments)],
        ),
        s::ValueKind::Constrain(value) => node_write(out, 24, &[Operand::Value(value)]),
        // Flag zero is a closed primitive `as` whose operation the types decide.
        s::ValueKind::ScalarCast(value) => {
            node_write(out, 28, &[Operand::Value(value), Operand::Flag(0)])
        }
        s::ValueKind::PointerCast(value) => {
            node_write(out, 28, &[Operand::Value(value), Operand::Flag(2)])
        }
        // Flag one is a fieldless enum's integer conversion.
        s::ValueKind::EnumCast(value) => {
            node_write(out, 28, &[Operand::Value(value), Operand::Flag(1)])
        }
        s::ValueKind::Binary {
            operator,
            operands,
            left,
            right,
        } => node_write(
            out,
            27,
            &[
                Operand::Value(left),
                Operand::Value(right),
                Operand::Flag(code(&OPERATORS, operator)?),
                Operand::Flag(code(&FORMS, operands)?),
            ],
        ),
        s::ValueKind::Invoke { .. }
        | s::ValueKind::Call { .. }
        | s::ValueKind::PointerCall { .. } => call_write(out, row.kind),
        kind => extended_value_write(out, kind),
    }
}
fn extended_value_write(out: &mut Writer, kind: s::ValueKind) -> Result<(), &'static str> {
    match kind {
        s::ValueKind::DirectEnumCast(value) => {
            node_write(out, 28, &[Operand::Value(value), Operand::Flag(3)])
        }
        s::ValueKind::AddressCast { kind, operand } => {
            let flag = match kind {
                s::AddressCast::PointerThin => 4,
                s::AddressCast::PointerToInteger => 5,
                s::AddressCast::IntegerToPointer => 6,
                s::AddressCast::FunctionPointer => 7,
            };
            node_write(out, 28, &[Operand::Value(operand), Operand::Flag(flag)])
        }
        s::ValueKind::Unary {
            operator,
            operand,
            literal,
        } => node_write(
            out,
            26,
            &[
                Operand::Value(operand),
                Operand::Flag(u32::from(operator == s::UnaryOperator::Not)),
                Operand::Flag(u32::from(literal)),
            ],
        ),
        s::ValueKind::Address(place) => node_write(out, 11, &[Operand::Place(place)]),
        s::ValueKind::Length(place) => node_write(out, 8, &[Operand::Place(place)]),
        s::ValueKind::Validate(place) => node_write(out, 15, &[Operand::Place(place)]),
        s::ValueKind::TextEquals { place, text } => {
            node_write(out, 9, &[Operand::Place(place), Operand::Flag(text)])
        }
        s::ValueKind::Union {
            owner,
            ordinal,
            value,
        } => node_write(
            out,
            4,
            &[
                Operand::Declaration(owner.0),
                Operand::Flag(ordinal),
                Operand::Value(value),
            ],
        ),
        s::ValueKind::Repeat { value, constant } => node_write(
            out,
            5,
            &[Operand::Value(value), Operand::Flag(u32::from(constant))],
        ),
        s::ValueKind::Unsize { value, conversion } => unsize_write(out, value, conversion),
        _ => Err("body IR extended value changed its form"),
    }
}
fn unsize_write(
    out: &mut Writer,
    value: u32,
    conversion: s::UnsizeConversion,
) -> Result<(), &'static str> {
    let (flag, first, second) = match conversion {
        s::UnsizeConversion::Slice(array) => {
            return node_write(
                out,
                25,
                &[
                    Operand::Value(value),
                    Operand::Flag(0),
                    Operand::Type(array.0),
                ],
            );
        }
        s::UnsizeConversion::Object { concrete, target } => (1, concrete, target),
        s::UnsizeConversion::Upcast { source, target } => (2, source, target),
        s::UnsizeConversion::Nominal { source, target } => (3, source, target),
        s::UnsizeConversion::Bound { source, target } => (4, source, target),
    };
    node_write(
        out,
        25,
        &[
            Operand::Value(value),
            Operand::Flag(flag),
            Operand::Type(first.0),
            Operand::Type(second.0),
        ],
    )
}
/// Call operators, direct calls and calls through a fn pointer.
fn call_write(out: &mut Writer, call: s::ValueKind) -> Result<(), &'static str> {
    match call {
        s::ValueKind::Invoke {
            kind,
            signature,
            arguments,
        } => node_write(
            out,
            29,
            &[
                Operand::Flag(code(&CALLABLES, kind)?),
                Operand::Type(signature.0),
                Operand::Range(arguments),
            ],
        ),
        s::ValueKind::Call {
            signature,
            callee,
            arguments,
        } => node_write(
            out,
            30,
            &[
                Operand::Type(signature.0),
                Operand::Value(callee),
                Operand::Range(arguments),
            ],
        ),
        // A call through a fn pointer adds its safety flag; the operand kinds,
        // not the count alone, distinguish the two shapes of code 30.
        s::ValueKind::PointerCall {
            signature,
            callee,
            arguments,
            safety,
        } => node_write(
            out,
            30,
            &[
                Operand::Type(signature.0),
                Operand::Value(callee),
                Operand::Range(arguments),
                Operand::Flag(code(&SAFETIES, safety)?),
            ],
        ),
        _ => Err("body IR call node changed its value form"),
    }
}
pub(super) fn value_read(read: &mut Input<'_, '_>) -> Result<s::Value, &'static str> {
    let ty = g::TypeRef(read.id()?);
    let node = node_read(read)?;
    let kind = match (node.code, node.operands()) {
        (2, []) => s::ValueKind::Unit,
        (3, &[Operand::Range(fields)]) => s::ValueKind::Aggregate { fields },
        (3, &[Operand::Range(fields), Operand::Declaration(variant)]) => {
            s::ValueKind::VariantAggregate {
                variant: DeclarationRef(variant),
                fields,
            }
        }
        (14, &[Operand::Place(place), Operand::Declaration(variant)]) => s::ValueKind::IsVariant {
            place,
            variant: DeclarationRef(variant),
        },
        (6, &[Operand::Bits(bits)]) => s::ValueKind::Scalar(bits),
        (7, &[Operand::Flag(ordinal)]) => s::ValueKind::Text(ordinal),
        (10, &[Operand::Place(place)]) => s::ValueKind::Read(place),
        (12, &[Operand::Place(place), Operand::Flag(0)]) => s::ValueKind::Borrow(place),
        (12, &[Operand::Place(place), Operand::Flag(1)]) => s::ValueKind::TwoPhaseBorrow(place),
        (12, &[Operand::Place(place), Operand::Flag(2)]) if read.profile.declaration_policies() => {
            s::ValueKind::UniqueBorrow(place)
        }
        (12, &[Operand::Place(_), Operand::Flag(2)]) => {
            return Err("encoded unique Rvalue::Borrow is not reconstructable");
        }
        (16, &[Operand::Application(application)]) => {
            s::ValueKind::Function(s::FunctionApplicationRef(application))
        }
        (17, &[Operand::Declaration(declaration)]) => {
            s::ValueKind::Constant(DeclarationRef(declaration))
        }
        (18, &[Operand::Declaration(declaration), Operand::Range(arguments)])
            if read.profile.declaration_policies() =>
        {
            s::ValueKind::AssociatedConstant {
                declaration: DeclarationRef(declaration),
                arguments,
            }
        }
        (18, _) if !read.profile.declaration_policies() => {
            return Err(
                "source associated constant reads require the version-fourteen declaration profile",
            );
        }
        (22, &[Operand::Slot(slot)]) if read.profile.declaration_policies() => {
            s::ValueKind::ConstArgument(slot)
        }
        (23, &[Operand::Value(value)]) => s::ValueKind::Reify(value),
        (0, &[Operand::Flag(instruction), Operand::Range(arguments)]) => s::ValueKind::Assembly {
            instruction,
            arguments,
        },
        (24, &[Operand::Value(value)]) => s::ValueKind::Constrain(value),
        (
            27,
            &[
                Operand::Value(left),
                Operand::Value(right),
                Operand::Flag(operator),
                Operand::Flag(form),
            ],
        ) => binary(left, right, operator, form, read.profile)?,
        (28, &[Operand::Value(value), Operand::Flag(0)]) => s::ValueKind::ScalarCast(value),
        (28, &[Operand::Value(value), Operand::Flag(1)]) => s::ValueKind::EnumCast(value),
        (28, &[Operand::Value(value), Operand::Flag(2)]) if read.profile.declaration_policies() => {
            s::ValueKind::PointerCast(value)
        }
        (29 | 30, operands) => callable_value(node.code, operands)?,
        (code, operands) => extended_value_read(code, operands, read.profile)?,
    };
    Ok(s::Value { ty, kind })
}
fn extended_value_read(
    code: u8,
    operands: &[Operand],
    profile: s::Profile,
) -> Result<s::ValueKind, &'static str> {
    let v14 = profile.declaration_policies();
    let kind = match (code, operands) {
        (28, &[Operand::Value(value), Operand::Flag(3)]) if v14 => {
            s::ValueKind::DirectEnumCast(value)
        }
        (28, &[Operand::Value(value), Operand::Flag(flag @ 4..=7)]) if v14 => {
            let kind = match flag {
                4 => s::AddressCast::PointerThin,
                5 => s::AddressCast::PointerToInteger,
                6 => s::AddressCast::IntegerToPointer,
                _ => s::AddressCast::FunctionPointer,
            };
            s::ValueKind::AddressCast {
                kind,
                operand: value,
            }
        }
        (
            26,
            &[
                Operand::Value(operand),
                Operand::Flag(op @ (0 | 1)),
                Operand::Flag(lit @ (0 | 1)),
            ],
        ) if v14 => s::ValueKind::Unary {
            operator: if op == 0 {
                s::UnaryOperator::Negate
            } else {
                s::UnaryOperator::Not
            },
            operand,
            literal: lit == 1,
        },
        (11, &[Operand::Place(place)]) if v14 => s::ValueKind::Address(place),
        (8, &[Operand::Place(place)]) if v14 => s::ValueKind::Length(place),
        (15, &[Operand::Place(place)]) if v14 => s::ValueKind::Validate(place),
        (9, &[Operand::Place(place), Operand::Flag(text)]) if v14 => {
            s::ValueKind::TextEquals { place, text }
        }
        (
            4,
            &[
                Operand::Declaration(owner),
                Operand::Flag(ordinal),
                Operand::Value(value),
            ],
        ) if v14 => s::ValueKind::Union {
            owner: DeclarationRef(owner),
            ordinal,
            value,
        },
        (5, &[Operand::Value(value), Operand::Flag(constant @ (0 | 1))]) if v14 => {
            s::ValueKind::Repeat {
                value,
                constant: constant == 1,
            }
        }
        (
            25,
            &[
                Operand::Value(value),
                Operand::Flag(0),
                Operand::Type(array),
            ],
        ) if v14 => s::ValueKind::Unsize {
            value,
            conversion: s::UnsizeConversion::Slice(g::TypeRef(array)),
        },
        (
            25,
            &[
                Operand::Value(value),
                Operand::Flag(flag @ 1..=4),
                Operand::Type(first),
                Operand::Type(second),
            ],
        ) if v14 => unsize_read(value, flag, g::TypeRef(first), g::TypeRef(second)),
        _ => {
            let forms: &[u8] = if v14 { &V14_VALUE_FORMS } else { &VALUE_FORMS };
            return Err(rejected(
                &VALUES,
                forms,
                code,
                "body IR value node has malformed operands",
                "unknown body IR value form",
            ));
        }
    };
    Ok(kind)
}
fn unsize_read(value: u32, flag: u32, first: g::TypeRef, second: g::TypeRef) -> s::ValueKind {
    let conversion = match flag {
        1 => s::UnsizeConversion::Object {
            concrete: first,
            target: second,
        },
        2 => s::UnsizeConversion::Upcast {
            source: first,
            target: second,
        },
        3 => s::UnsizeConversion::Nominal {
            source: first,
            target: second,
        },
        _ => s::UnsizeConversion::Bound {
            source: first,
            target: second,
        },
    };
    s::ValueKind::Unsize { value, conversion }
}

fn callable_value(code: u8, operands: &[Operand]) -> Result<s::ValueKind, &'static str> {
    Ok(match (code, operands) {
        (
            29,
            &[
                Operand::Flag(kind),
                Operand::Type(signature),
                Operand::Range(arguments),
            ],
        ) => s::ValueKind::Invoke {
            kind: *CALLABLES
                .get(kind as usize)
                .ok_or("body IR invoke node has an unknown call trait")?,
            signature: g::TypeRef(signature),
            arguments,
        },
        (
            30,
            &[
                Operand::Type(signature),
                Operand::Value(callee),
                Operand::Range(arguments),
            ],
        ) => s::ValueKind::Call {
            signature: g::TypeRef(signature),
            callee,
            arguments,
        },
        (
            30,
            &[
                Operand::Type(signature),
                Operand::Value(callee),
                Operand::Range(arguments),
                Operand::Flag(safety),
            ],
        ) => s::ValueKind::PointerCall {
            signature: g::TypeRef(signature),
            callee,
            arguments,
            safety: *SAFETIES
                .get(safety as usize)
                .ok_or("unknown source call safety context")?,
        },
        _ => return Err("body IR value node has malformed operands"),
    })
}
fn code<T: Copy + PartialEq>(table: &[T], form: T) -> Result<u32, &'static str> {
    table
        .iter()
        .position(|entry| *entry == form)
        .and_then(|index| u32::try_from(index).ok())
        .ok_or("body IR form has no stable code")
}
fn binary(
    left: u32,
    right: u32,
    operator: u32,
    form: u32,
    profile: s::Profile,
) -> Result<s::ValueKind, &'static str> {
    let Some(&operands) = FORMS.get(form as usize) else {
        return Err("body IR binary node has an unknown operand form");
    };
    match OPERATORS.get(operator as usize) {
        Some(&operator) if !operator.requires_v14() || profile.declaration_policies() => {
            Ok(s::ValueKind::Binary {
                operator,
                operands,
                left,
                right,
            })
        }
        _ if operator < CHECKED_OPERATORS => Err(
            "encoded checked division or shift Rvalue::Binary is not reconstructable by this consumer",
        ),
        _ => Err("body IR binary node has an unknown operator"),
    }
}
pub(super) fn statement_write(out: &mut Writer, row: &s::Statement) -> Result<(), &'static str> {
    id(out, row.site)?;
    match row.action {
        s::Action::CheckInitialized(place) => node_write(out, 0, &[Operand::Place(place)]),
        s::Action::Mention(place) => node_write(out, 1, &[Operand::Place(place)]),
        s::Action::Project(projection) => node_write(out, 5, &[Operand::Projection(projection)]),
        s::Action::StorageLive(local) => node_write(out, 7, &[Operand::Local(local)]),
        s::Action::StorageDead(local) => node_write(out, 8, &[Operand::Local(local)]),
        s::Action::StorageEnd { first, last } => {
            node_write(out, 9, &[Operand::Local(first), Operand::Local(last)])
        }
        s::Action::Evaluate(value) => node_write(out, 10, &[Operand::Value(value)]),
        s::Action::Store { place, value } => {
            node_write(out, 11, &[Operand::Place(place), Operand::Value(value)])
        }
        // A guard ordinal indexes its original's guard table.
        s::Action::GuardStart(guard) => node_write(out, 2, &[Operand::Flag(guard)]),
        s::Action::GuardUse(guard) => node_write(out, 3, &[Operand::Flag(guard)]),
        s::Action::BindView { view, place } => {
            node_write(out, 4, &[Operand::Local(view), Operand::Place(place)])
        }
        s::Action::Select { projection, anchor } => node_write(
            out,
            6,
            &[Operand::Projection(projection), Operand::Place(anchor)],
        ),
    }
}
pub(super) fn statement_read(read: &mut Input<'_, '_>) -> Result<s::Statement, &'static str> {
    let site = read.id()?;
    let node = node_read(read)?;
    let action = match (node.code, node.operands()) {
        (0, &[Operand::Place(place)]) => s::Action::CheckInitialized(place),
        (1, &[Operand::Place(place)]) => s::Action::Mention(place),
        (5, &[Operand::Projection(projection)]) => s::Action::Project(projection),
        (7, &[Operand::Local(local)]) => s::Action::StorageLive(local),
        (8, &[Operand::Local(local)]) => s::Action::StorageDead(local),
        (9, &[Operand::Local(first), Operand::Local(last)]) => {
            s::Action::StorageEnd { first, last }
        }
        (10, &[Operand::Value(value)]) => s::Action::Evaluate(value),
        (11, &[Operand::Place(place), Operand::Value(value)]) => s::Action::Store { place, value },
        (2, &[Operand::Flag(guard)]) => s::Action::GuardStart(guard),
        (3, &[Operand::Flag(guard)]) => s::Action::GuardUse(guard),
        (4, &[Operand::Local(view), Operand::Place(place)]) => s::Action::BindView { view, place },
        (6, &[Operand::Projection(projection), Operand::Place(anchor)]) => {
            s::Action::Select { projection, anchor }
        }
        (code, _) => {
            return Err(rejected(
                &ACTIONS,
                &ACTION_FORMS,
                code,
                "body IR statement node has malformed operands",
                "unknown body IR statement form",
            ));
        }
    };
    Ok(s::Statement { site, action })
}
pub(super) fn block_write(out: &mut Writer, row: &g::Block) -> Result<(), &'static str> {
    range(out, row.statements)?;
    id(out, row.site)?;
    match row.terminator {
        g::Terminator::Jump(block) => node_write(out, 0, &[Operand::Block(block)]),
        g::Terminator::FalseEdge { real, imaginary } => {
            node_write(out, 1, &[Operand::Block(real), Operand::Block(imaginary)])
        }
        g::Terminator::Branch { condition, yes, no } => node_write(
            out,
            2,
            &[
                Operand::Value(condition),
                Operand::Block(yes),
                Operand::Block(no),
            ],
        ),
        g::Terminator::Return(value) => node_write(out, 3, &[Operand::Value(value)]),
        g::Terminator::Unreachable => node_write(out, 4, &[]),
        g::Terminator::Panic => node_write(out, 5, &[]),
    }
}
pub(super) fn block_read(read: &mut Input<'_, '_>) -> Result<g::Block, &'static str> {
    let statements = read.range()?;
    let site = read.id()?;
    let node = node_read(read)?;
    let terminator = match (node.code, node.operands()) {
        (0, &[Operand::Block(block)]) => g::Terminator::Jump(block),
        (1, &[Operand::Block(executed), Operand::Block(imaginary)]) => g::Terminator::FalseEdge {
            real: executed,
            imaginary,
        },
        (
            2,
            &[
                Operand::Value(condition),
                Operand::Block(yes),
                Operand::Block(no),
            ],
        ) => g::Terminator::Branch { condition, yes, no },
        (3, &[Operand::Value(value)]) => g::Terminator::Return(value),
        (4, []) => g::Terminator::Unreachable,
        (5, []) => g::Terminator::Panic,
        (code, _) => {
            return Err(rejected(
                &TERMINATORS,
                &TERMINATOR_FORMS,
                code,
                "body IR terminator node has malformed operands",
                "unknown body IR terminator form",
            ));
        }
    };
    Ok(g::Block {
        statements,
        site,
        terminator,
    })
}
pub(super) fn projection_write(out: &mut Writer, row: &s::Projection) -> Result<(), &'static str> {
    operand_write(out, Operand::Place(row.base))?;
    id(out, row.ty.0)?;
    let u64_bits = |value: u64| Operand::Bits(u128::from(value).to_le_bytes());
    match row.kind {
        s::ProjectionKind::CaptureBoundary { mutable } => {
            node_write(out, 0, &[Operand::Flag(u32::from(mutable))])
        }
        s::ProjectionKind::Dereference { region, mutable } => node_write(
            out,
            1,
            &[Operand::Region(region.0), Operand::Flag(u32::from(mutable))],
        ),
        s::ProjectionKind::RawDereference { mutable } => {
            node_write(out, 2, &[Operand::Flag(u32::from(mutable))])
        }
        s::ProjectionKind::Field { owner, ordinal } => node_write(
            out,
            3,
            &[Operand::Declaration(owner.0), Operand::Flag(ordinal)],
        ),
        s::ProjectionKind::Tuple(ordinal) => node_write(out, 3, &[Operand::Flag(ordinal)]),
        s::ProjectionKind::UnionField { owner, ordinal } => node_write(
            out,
            4,
            &[Operand::Declaration(owner.0), Operand::Flag(ordinal)],
        ),
        s::ProjectionKind::Index { index, length } => {
            node_write(out, 5, &[Operand::Value(index), u64_bits(length)])
        }
        s::ProjectionKind::IndexParameter { index, length } => {
            node_write(out, 6, &[Operand::Value(index), Operand::Slot(length)])
        }
        s::ProjectionKind::SliceIndex { index } => node_write(out, 7, &[Operand::Value(index)]),
        s::ProjectionKind::ConstantIndex(offset) => node_write(out, 8, &[u64_bits(offset)]),
        s::ProjectionKind::FromEnd { offset, minimum } => {
            node_write(out, 9, &[u64_bits(offset), u64_bits(minimum)])
        }
        s::ProjectionKind::Subslice { prefix, suffix } => {
            node_write(out, 10, &[u64_bits(prefix), u64_bits(suffix)])
        }
        s::ProjectionKind::Subarray { start, end } => {
            node_write(out, 11, &[u64_bits(start), u64_bits(end)])
        }
    }
}
pub(super) fn projection_read(read: &mut Input<'_, '_>) -> Result<s::Projection, &'static str> {
    let Operand::Place(base) = operand_read(read)? else {
        return Err("body IR projection base is not a place");
    };
    let ty = g::TypeRef(read.id()?);
    let node = node_read(read)?;
    let bits_u64 = |bits: [u8; 16]| {
        u64::try_from(u128::from_le_bytes(bits))
            .map_err(|_| "body IR projection u64 operand overflow")
    };
    let v14 = read.profile.declaration_policies();
    let kind = match (node.code, node.operands()) {
        (0, &[Operand::Flag(mutable @ (0 | 1))]) if v14 => s::ProjectionKind::CaptureBoundary {
            mutable: mutable == 1,
        },
        (1, &[Operand::Region(region), Operand::Flag(mutable @ (0 | 1))]) => {
            s::ProjectionKind::Dereference {
                region: s::RegionRef(region),
                mutable: mutable == 1,
            }
        }
        (2, &[Operand::Flag(mutable @ (0 | 1))]) if v14 => s::ProjectionKind::RawDereference {
            mutable: mutable == 1,
        },
        (3, &[Operand::Declaration(owner), Operand::Flag(ordinal)]) => s::ProjectionKind::Field {
            owner: DeclarationRef(owner),
            ordinal,
        },
        (3, &[Operand::Flag(ordinal)]) if v14 => s::ProjectionKind::Tuple(ordinal),
        (4, &[Operand::Declaration(owner), Operand::Flag(ordinal)]) if v14 => {
            s::ProjectionKind::UnionField {
                owner: DeclarationRef(owner),
                ordinal,
            }
        }
        (5, &[Operand::Value(index), Operand::Bits(length)]) if v14 => s::ProjectionKind::Index {
            index,
            length: bits_u64(length)?,
        },
        (6, &[Operand::Value(index), Operand::Slot(length)]) if v14 => {
            s::ProjectionKind::IndexParameter { index, length }
        }
        (7, &[Operand::Value(index)]) if v14 => s::ProjectionKind::SliceIndex { index },
        (8, &[Operand::Bits(offset)]) if v14 => s::ProjectionKind::ConstantIndex(bits_u64(offset)?),
        (9, &[Operand::Bits(offset), Operand::Bits(minimum)]) if v14 => {
            s::ProjectionKind::FromEnd {
                offset: bits_u64(offset)?,
                minimum: bits_u64(minimum)?,
            }
        }
        (10, &[Operand::Bits(prefix), Operand::Bits(suffix)]) if v14 => {
            s::ProjectionKind::Subslice {
                prefix: bits_u64(prefix)?,
                suffix: bits_u64(suffix)?,
            }
        }
        (11, &[Operand::Bits(start), Operand::Bits(end)]) if v14 => s::ProjectionKind::Subarray {
            start: bits_u64(start)?,
            end: bits_u64(end)?,
        },
        (code, _) => {
            let forms: &[u8] = if v14 {
                &V14_PROJECTION_FORMS
            } else {
                &PROJECTION_FORMS
            };
            return Err(rejected(
                &PROJECTIONS,
                forms,
                code,
                "body IR projection node has malformed operands",
                "unknown body IR projection form",
            ));
        }
    };
    Ok(s::Projection { base, ty, kind })
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/called_pointer_wire.rs"]
mod tests;
