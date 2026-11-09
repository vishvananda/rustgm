//! Version thirteen's const slots and terms re-validate from the provider's
//! own exported rows: each value is canonical for its primitive type, each
//! slot is an integer, bool or char formal without a default, and each term
//! fills a const position. Every forged row is rejected by name.
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw},
    support::crates::{
        self, Scalar,
        generics::{self as g, source as s},
    },
};

const PROVIDER: &str = "pa29/tests/spec/const-parameters/provider.rs";
const NON_CANONICAL: &str = "non-canonical source const value";
type Mutation = fn(&mut s::SourceRows) -> Option<()>;

fn validate(
    interface: &crates::Interface,
    rows: s::SourceRows,
) -> std::result::Result<(), &'static str> {
    let descriptor = interface.source_descriptor().ok_or("source descriptor")?;
    let (_, attributes, modules) = raw(interface).map_err(|_| "raw rows")?;
    let availability = descriptor
        .availability()
        .map_err(|_| "availability")?
        .clone();
    g::Payload::from_source_module_bodies(
        interface.identity(),
        rows,
        attributes,
        modules,
        availability,
        0,
        &mut g::Work::default(),
    )
    .map(|_| ())
}
fn scalar(rows: &s::SourceRows, ty: g::TypeRef) -> Option<Scalar> {
    match rows.types.get(ty.0 as usize)? {
        s::Type::Scalar(scalar) => Some(*scalar),
        _ => None,
    }
}
/// Every value term of `ty`, as `(ref, value)`.
fn values(rows: &s::SourceRows, ty: Scalar) -> Vec<(usize, s::ConstValue)> {
    rows.consts()
        .iter()
        .enumerate()
        .filter_map(|(index, term)| {
            let s::ConstTerm::Value(value) = *term else {
                return None;
            };
            (scalar(rows, value.ty) == Some(ty)).then_some((index, value))
        })
        .collect()
}
/// The one value term of `ty` whose bits are `bits`.
fn value(rows: &s::SourceRows, ty: Scalar, bits: u128) -> Option<(usize, s::ConstValue)> {
    let found: Vec<_> = values(rows, ty)
        .into_iter()
        .filter(|(_, value)| value.bits == bits)
        .collect();
    let [only] = found.as_slice() else {
        return None;
    };
    Some(*only)
}
fn set_bits(rows: &mut s::SourceRows, ty: Scalar, from: u128, bits: u128) -> Option<()> {
    let (index, mut value) = value(rows, ty, from)?;
    value.bits = bits;
    rows.cold_mut().consts[index] = s::ConstTerm::Value(value);
    Some(())
}
/// The argument row of the one function application whose const argument
/// is the value `bits`, as `(argument row, term ref)`.
fn applied(rows: &s::SourceRows, bits: u128) -> Option<(usize, usize)> {
    let found: Vec<(usize, usize)> = rows
        .function_applications
        .iter()
        .flat_map(|application| application.arguments.start..application.arguments.end)
        .filter_map(|row| {
            let s::Argument::Const(term) = *rows.arguments.get(row as usize)? else {
                return None;
            };
            let s::ConstTerm::Value(value) = *rows.consts().get(term.0 as usize)? else {
                return None;
            };
            (value.bits == bits).then_some((row as usize, term.0 as usize))
        })
        .collect();
    let [only] = found.as_slice() else {
        return None;
    };
    Some(*only)
}
/// Retype the one function application's const value `bits` as `u8`.
fn retype_applied(rows: &mut s::SourceRows, bits: u128) -> Option<()> {
    let (_, term) = applied(rows, bits)?;
    let byte = rows
        .types
        .iter()
        .position(|row| matches!(row, s::Type::Scalar(Scalar::U8)))?;
    let s::ConstTerm::Value(mut value) = rows.consts()[term] else {
        return None;
    };
    value.ty = g::TypeRef(u32::try_from(byte).ok()?);
    rows.cold_mut().consts[term] = s::ConstTerm::Value(value);
    Some(())
}
fn const_slot(rows: &s::SourceRows) -> Option<usize> {
    rows.slots
        .iter()
        .position(|slot| matches!(slot.kind, s::ParameterKind::Const { .. }))
}
fn type_slot(rows: &s::SourceRows) -> Option<usize> {
    rows.slots
        .iter()
        .position(|slot| matches!(slot.kind, s::ParameterKind::Type { .. }))
}

/// Each forged row and the name validation rejects it by.
const MUTATIONS: [(&str, Mutation, &str); 13] = [
    (
        "sign-extended i8",
        |rows| set_bits(rows, Scalar::I8, 0xFF, u128::MAX),
        NON_CANONICAL,
    ),
    (
        "bool bits 2",
        |rows| set_bits(rows, Scalar::Bool, 1, 2),
        NON_CANONICAL,
    ),
    (
        "char surrogate",
        |rows| set_bits(rows, Scalar::Char, 0x7A, 0xD800),
        NON_CANONICAL,
    ),
    (
        "bits above usize width",
        |rows| set_bits(rows, Scalar::Usize, 3, 1 << 64 | 3),
        NON_CANONICAL,
    ),
    (
        "term type not primitive",
        |rows| {
            // The first non-scalar row precedes every authored K row.
            let (index, mut value) = value(rows, Scalar::Usize, 3)?;
            let row = rows
                .types
                .iter()
                .position(|row| !matches!(row, s::Type::Scalar(_)))?;
            value.ty = g::TypeRef(u32::try_from(row).ok()?);
            rows.cold_mut().consts[index] = s::ConstTerm::Value(value);
            Some(())
        },
        "source const value type is not a primitive row",
    ),
    (
        "float slot type",
        |rows| {
            let slot = const_slot(rows)?;
            let s::ParameterKind::Const { ty } = rows.slots[slot].kind else {
                return None;
            };
            let float = rows
                .types
                .iter()
                .position(|row| matches!(row, s::Type::Scalar(Scalar::F32 | Scalar::F64)));
            match float {
                Some(float) => {
                    rows.slots[slot].kind = s::ParameterKind::Const {
                        ty: g::TypeRef(u32::try_from(float).ok()?),
                    };
                }
                None => rows.types[ty.0 as usize] = s::Type::Scalar(Scalar::F64),
            }
            Some(())
        },
        "source const parameter type is outside the initial profile",
    ),
    (
        "term names a type slot",
        |rows| {
            let slot = s::SlotRef(u32::try_from(type_slot(rows)?).ok()?);
            // The last rigid term: a Self type's meets its identity check first.
            let term = rows
                .consts()
                .iter()
                .rposition(|term| matches!(term, s::ConstTerm::Parameter(_)))?;
            rows.cold_mut().consts[term] = s::ConstTerm::Parameter(slot);
            Some(())
        },
        "source const term names a non-const slot",
    ),
    (
        "const slot default",
        |rows| {
            let slot = const_slot(rows)?;
            let s::ParameterKind::Const { ty } = rows.slots[slot].kind else {
                return None;
            };
            rows.slots[slot].default = Some(ty);
            Some(())
        },
        "source const parameter slot has a default",
    ),
    (
        "const argument in a type slot",
        |rows| {
            // `Mixed<N, T>` writes its const argument first; move it into T.
            // Take the last pair, since a Self type's meets its identity
            // check first.
            let position = rows
                .arguments
                .windows(2)
                .rposition(|pair| matches!(pair, [s::Argument::Const(_), s::Argument::Type(_)]))?;
            rows.arguments.swap(position, position + 1);
            Some(())
        },
        "source nominal argument changed its formal kind",
    ),
    (
        "application value of another type",
        |rows| retype_applied(rows, 9),
        "source const argument changed its parameter type",
    ),
    (
        "inherited formal's value of another type",
        |rows| retype_applied(rows, 5),
        "source const argument changed its parameter type",
    ),
    (
        "type argument in an application's const formal",
        |rows| {
            let (row, _) = applied(rows, 9)?;
            let byte = rows
                .types
                .iter()
                .position(|row| matches!(row, s::Type::Scalar(Scalar::U8)))?;
            rows.arguments[row] = s::Argument::Type(g::TypeRef(u32::try_from(byte).ok()?));
            Some(())
        },
        "source application changed its actual formal kind",
    ),
    (
        "unknown term ref",
        |rows| {
            let argument = rows
                .arguments
                .iter_mut()
                .rev()
                .find(|argument| matches!(argument, s::Argument::Const(_)))?;
            *argument = s::Argument::Const(s::ConstRef(u32::MAX));
            Some(())
        },
        "source const argument has no term row",
    ),
];
#[test]
fn const_rows_keep_canonical_values_and_reject_forged_slots_and_terms() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "const_slots")?;
    let (rows, _, _) = raw(&interface)?;
    validate(&interface, rows.clone())?;
    // One dense term per const argument row, each a value or a const slot.
    let arguments = rows
        .arguments
        .iter()
        .filter(|argument| matches!(argument, s::Argument::Const(_)))
        .count();
    assert_eq!(rows.consts().len(), arguments);
    // Constant's masked two's-complement bits: i8 -1 is 0xFF, true is 1 and
    // 'z' is 0x7A. The usize values are `three`'s 3, `five`'s 5 and `nine`'s
    // 9; 3 appears once, as `three`'s argument.
    for (ty, bits) in [
        (Scalar::I8, vec![0xFF]),
        (Scalar::Bool, vec![1]),
        (Scalar::Char, vec![0x7A]),
        (Scalar::Usize, vec![3, 5, 9]),
    ] {
        let mut found: Vec<u128> = values(&rows, ty)
            .into_iter()
            .map(|(_, value)| value.bits)
            .collect();
        found.sort_unstable();
        found.dedup();
        assert_eq!(found, bits, "{ty:?}");
    }
    assert!(value(&rows, Scalar::Usize, 3).is_some());
    // `wrap::<9>` fills its own formal, and `Tag::<5>::new` its impl's.
    assert!(applied(&rows, 9).is_some() && applied(&rows, 5).is_some());
    for (label, mutate, expected) in MUTATIONS {
        let mut changed = rows.clone();
        mutate(&mut changed).ok_or(label)?;
        assert_eq!(
            validate(&interface, changed).err(),
            Some(expected),
            "{label}"
        );
    }
    Ok(())
}
