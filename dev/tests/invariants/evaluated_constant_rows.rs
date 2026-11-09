//! Version thirteen's named constants: declaration tag 13, one header row per
//! constant declaration (`ty | value`, with K's term in place) and value code
//! 17. Every consumer re-validates each row from the provider's own exported
//! rows, and every forged row, byte or writer input is rejected by name.
use super::*;
use crate::{
    lower::instances::SourcePublication,
    module_packets::{exported_profile, raw},
    support::crates::{Scalar, generics::PayloadView},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Mutation = fn(&mut s::SourceRows) -> Option<()>;

const PROVIDER: &str = "pa29/tests/spec/named-constants/provider.rs";
const BODIES: s::Profile = s::Profile::Source2024ModuleBodies;
const BRANCHES: s::Profile = s::Profile::Source2024ModuleBranches;
const NON_CANONICAL: &str = "non-canonical source const value";
/// `ty`, the presence byte, then the term's tag, `ty` and sixteen bits bytes.
const ROW: usize = 26;

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
fn type_row(rows: &mut s::SourceRows, wanted: Scalar) -> Option<g::TypeRef> {
    let wanted = s::Type::Scalar(wanted);
    let row = if let Some(row) = rows.types.iter().position(|row| *row == wanted) {
        row
    } else {
        rows.types.push(wanted);
        rows.types.len() - 1
    };
    Some(g::TypeRef(u32::try_from(row).ok()?))
}
/// The first constant row of `ty`, as `(row, value)`.
fn constant(rows: &s::SourceRows, ty: Scalar) -> Option<(usize, s::ConstValue)> {
    rows.constants()
        .iter()
        .enumerate()
        .find_map(|(index, header)| match header.value {
            Some(s::ConstTerm::Value(value)) if scalar(rows, header.ty) == Some(ty) => {
                Some((index, value))
            }
            _ => None,
        })
}
fn set_bits(rows: &mut s::SourceRows, ty: Scalar, bits: u128) -> Option<()> {
    let (index, mut value) = constant(rows, ty)?;
    value.bits = bits;
    rows.cold_mut().constants[index].value = Some(s::ConstTerm::Value(value));
    Some(())
}
fn set_value(rows: &mut s::SourceRows, value: Option<s::ConstTerm>) -> Option<()> {
    let (index, _) = constant(rows, Scalar::U8)?;
    rows.cold_mut().constants[index].value = value;
    Some(())
}
/// The one code-17 value reading the constant declared `name`.
fn read_of(rows: &s::SourceRows, name: &str) -> Option<(usize, usize)> {
    let found: Vec<(usize, usize)> = rows
        .originals
        .iter()
        .enumerate()
        .flat_map(|(original, row)| {
            row.values
                .iter()
                .enumerate()
                .filter_map(move |(index, value)| match value.kind {
                    s::ValueKind::Constant(declaration) => Some((original, index, declaration)),
                    _ => None,
                })
        })
        .filter(|&(_, _, declaration)| {
            rows.declarations
                .get(declaration.0 as usize)
                .and_then(|row| row.name.as_deref())
                == Some(name)
        })
        .map(|(original, index, _)| (original, index))
        .collect();
    let [only] = found.as_slice() else {
        return None;
    };
    Some(*only)
}
/// Move the item-local constant `INNER` of `nested` under the first `parent`
/// declaration. It is numbered among its new parent's lexical children in row
/// order, and the later children of both parents shift to match, so the
/// parent's kind is the only change (amendment 8).
fn reparent(rows: &mut s::SourceRows, parent: s::DefinitionKind) -> Option<()> {
    let target = rows
        .declarations
        .iter()
        .position(|row| row.definition.kind == parent)?;
    let inner = rows.declarations.iter().position(|row| {
        row.definition.kind == s::DefinitionKind::Constant && row.name.as_deref() == Some("INNER")
    })?;
    let owner = Some(DeclarationRef(u32::try_from(target).ok()?));
    let previous = rows.declarations[inner].definition.parent;
    let ordinal = rows.declarations[..inner]
        .iter()
        .filter(|row| row.definition.parent == owner)
        .count();
    for row in &mut rows.declarations[inner + 1..] {
        if row.definition.parent == previous {
            row.ordinal = row.ordinal.checked_sub(1)?;
        } else if row.definition.parent == owner {
            row.ordinal = row.ordinal.checked_add(1)?;
        }
    }
    let inner = &mut rows.declarations[inner];
    inner.definition.parent = owner;
    inner.ordinal = u32::try_from(ordinal).ok()?;
    Some(())
}

/// Each forged row and the name validation rejects it by.
const MUTATIONS: [(&str, Mutation, &str); 12] = [
    (
        "u8 const with bits 0x100",
        |rows| set_bits(rows, Scalar::U8, 0x100),
        NON_CANONICAL,
    ),
    (
        "bool const with bits 2",
        |rows| set_bits(rows, Scalar::Bool, 2),
        NON_CANONICAL,
    ),
    (
        "char const with bits 0xD800",
        |rows| set_bits(rows, Scalar::Char, 0xD800),
        NON_CANONICAL,
    ),
    (
        "term tag 1 in a free const",
        |rows| set_value(rows, Some(s::ConstTerm::Parameter(s::SlotRef(0)))),
        "source const value over a const parameter requires its wider profile",
    ),
    (
        "presence 0 in a free const",
        |rows| set_value(rows, None),
        "source free constant requires its value",
    ),
    (
        "term ty a different row from the header ty",
        |rows| {
            let (index, mut value) = constant(rows, Scalar::Usize)?;
            value.ty = type_row(rows, Scalar::U8)?;
            rows.cold_mut().constants[index].value = Some(s::ConstTerm::Value(value));
            Some(())
        },
        "source constant value changed its declared type",
    ),
    (
        "header and term ty a reference row",
        |rows| {
            let (index, mut value) = constant(rows, Scalar::U8)?;
            let reference = rows
                .types
                .iter()
                .position(|row| matches!(row, s::Type::Reference { .. }))?;
            value.ty = g::TypeRef(u32::try_from(reference).ok()?);
            let header = &mut rows.cold_mut().constants[index];
            header.ty = value.ty;
            header.value = Some(s::ConstTerm::Value(value));
            Some(())
        },
        "source const value type is not a primitive row",
    ),
    (
        "code 17 naming a function declaration",
        |rows| {
            let (original, index) = read_of(rows, "SECRET")?;
            let function = rows.declarations.iter().position(|row| {
                row.definition.kind == s::DefinitionKind::Function
                    && row.name.as_deref() == Some("secret")
            })?;
            rows.originals[original].values[index].kind =
                s::ValueKind::Constant(DeclarationRef(u32::try_from(function).ok()?));
            Some(())
        },
        "source constant read names no validated constant declaration",
    ),
    (
        "code 17 typed u16 over a u8 constant",
        |rows| {
            let (original, index) = read_of(rows, "SECRET")?;
            let wide = type_row(rows, Scalar::U16)?;
            rows.originals[original].values[index].ty = wide;
            Some(())
        },
        "source constant read changed its declaration's type",
    ),
    (
        "tag 13 under a trait's Self declaration",
        |rows| reparent(rows, s::DefinitionKind::SelfType),
        "source associated constants require their wider profile",
    ),
    // Amendment 4's substitute for a struct parent. The existing lexical
    // forest rejects a type-parameter parent before any constant check
    // (amendment 7, a failed prediction).
    (
        "tag 13 under a type parameter declaration",
        |rows| reparent(rows, s::DefinitionKind::TypeParameter),
        "source lexical child has a foreign kind, gap or duplicate ordinal",
    ),
    // The writer's own check is behind validation, which no surplus row
    // passes: the identical ledger check names it first.
    (
        "a constant row with no tag-13 declaration",
        |rows| {
            let surplus = *rows.constants().first()?;
            rows.cold_mut().constants.push(surplus);
            Some(())
        },
        "source constant rows lack their declaration",
    ),
];

fn written(
    row: impl FnOnce(&mut Writer) -> std::result::Result<(), &'static str>,
) -> Result<Vec<u8>> {
    let mut out = Writer::default();
    row(&mut out)?;
    Ok(out.finish())
}
/// The constant column of `rows`, decoded under `profile` against `rows`'
/// own declaration table; a successful read consumes every byte.
fn decoded(
    bytes: &[u8],
    profile: s::Profile,
    rows: &s::SourceRows,
) -> std::result::Result<Vec<s::ConstantHeader>, &'static str> {
    let mut rows = rows.clone();
    rows.cold_mut().constants.clear();
    let mut reader = Reader::new(bytes);
    free_constants_wire::decode(
        &mut Input {
            reader: &mut reader,
            bytes: 0,
            base: 0,
            profile,
        },
        &mut rows,
    )?;
    if !reader.done() {
        return Err("constant column left trailing bytes");
    }
    Ok(rows.constants().to_vec())
}

#[test]
fn constant_rows_keep_completed_values_and_reject_forged_rows() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "constant_rows")?;
    let (rows, _, _) = raw(&interface)?;
    validate(&interface, rows.clone())?;
    // One row per tag-13 declaration: nine module constants and three
    // item-local `INNER`s. Each holds its exact type's canonical bits.
    let declared: Vec<_> = rows.constant_rows().collect();
    assert_eq!(declared.len(), 12);
    assert_eq!(rows.constants().len(), 12);
    for (declaration, header) in &declared {
        let Some(s::ConstTerm::Value(value)) = header.value else {
            return Err("free constant without a value".into());
        };
        assert_eq!(value.ty, header.ty);
        let name = rows.declarations[declaration.0 as usize].name.as_deref();
        let expected = match name {
            Some("LANES") => (Scalar::Usize, 4),
            Some("FLAG") => (Scalar::Bool, 1),
            Some("LETTER") => (Scalar::Char, 0x78),
            Some("NEG") => (Scalar::I8, 0xFF),
            Some("HALF") => (Scalar::F32, u128::from(42.5_f32.to_bits())),
            Some("QUARTER") => (Scalar::F64, u128::from(42.25_f64.to_bits())),
            Some("BIG") => (Scalar::U128, u128::MAX),
            Some("NOTHING") => (Scalar::Unit, 0),
            Some("SECRET") => (Scalar::U8, 42),
            _ => (Scalar::U8, value.bits),
        };
        assert_eq!(
            (scalar(&rows, header.ty), value.bits),
            (Some(expected.0), expected.1),
            "{name:?}"
        );
    }
    let mut inner: Vec<u128> = declared
        .iter()
        .filter(|(declaration, _)| {
            rows.declarations[declaration.0 as usize].name.as_deref() == Some("INNER")
        })
        .filter_map(|(_, header)| match header.value {
            Some(s::ConstTerm::Value(value)) => Some(value.bits),
            _ => None,
        })
        .collect();
    inner.sort_unstable();
    assert_eq!(inner, [2, 40, 42]);
    // `lanes`, `neg`, `secret`, the three `INNER` readers and `lane_count`
    // read their constants through code 17.
    assert!(read_of(&rows, "SECRET").is_some() && read_of(&rows, "NEG").is_some());
    for (label, mutate, expected) in MUTATIONS {
        let mut changed = rows.clone();
        mutate(&mut changed).ok_or(label)?;
        assert_eq!(
            validate(&interface, changed).err(),
            Some(expected),
            "{label}"
        );
    }
    // Only a struct or enum parent passes the lexical forest and owns no
    // free item. The provider declares neither, so this input adds one unit
    // struct (amendments 7 and 8); its unmodified rows validate first.
    let source = format!("{source}// Generated struct-parent forgery input.\npub struct Holder;\n");
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "constant_parent")?;
    let (rows, _, _) = raw(&interface)?;
    validate(&interface, rows.clone())?;
    let mut changed = rows;
    reparent(&mut changed, s::DefinitionKind::Struct).ok_or("struct parent")?;
    assert_eq!(
        validate(&interface, changed).err(),
        Some("source constant header lost its free owner"),
        "tag 13 under a struct declaration"
    );
    Ok(())
}

#[test]
fn constant_columns_round_trip_and_reject_forged_bytes_by_name() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "constant_wire")?;
    let (rows, _, _) = raw(&interface)?;
    let payload = interface.generic_payload().ok_or("generic payload")?;
    let PayloadView::Source(validated) = payload.view() else {
        return Err("source payload".into());
    };
    let bytes = written(|out| free_constants_wire::encode(out, validated.view()))?;
    assert_eq!(bytes.len(), ROW * rows.constants().len());
    assert_eq!(decoded(&bytes, BODIES, &rows)?, rows.constants());
    // The first row's presence byte, then its term's tag.
    let mut presence = bytes.clone();
    presence[4] = 2;
    assert_eq!(
        decoded(&presence, BODIES, &rows).err(),
        Some("invalid source constant value presence")
    );
    let mut term = bytes;
    term[5] = 2;
    assert_eq!(
        decoded(&term, BODIES, &rows).err(),
        Some("unknown source const term")
    );
    // Tag 13 is read only under the body-nodes profile.
    let declaration = rows
        .declarations
        .iter()
        .find(|row| row.definition.kind == s::DefinitionKind::Constant)
        .ok_or("constant declaration")?
        .clone();
    let row = written(|out| owner_rows::declaration_write(out, &declaration))?;
    assert_eq!(row[0], 13);
    let mut reader = Reader::new(&row);
    let read = owner_rows::declaration_read(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile: BRANCHES,
    });
    assert_eq!(read.err(), Some("unknown source declaration kind"));
    // The legacy branches writer has no code 17.
    let mut original = rows
        .originals
        .iter()
        .find(|original| {
            original
                .values
                .iter()
                .any(|value| matches!(value.kind, s::ValueKind::Constant(_)))
        })
        .ok_or("code-17 original")?
        .clone();
    original.texts = None;
    assert_eq!(
        written(|out| operation_columns::branch_write(out, &original))
            .err()
            .map(|error| error.to_string()),
        Some("source constant read requires the version-thirteen body IR profile".to_owned())
    );
    Ok(())
}

/// The branch provider's first 57 lines, `n` named constants and one body
/// reading the first, as the declared scaling inputs.
fn scaled(prefix: &str, n: usize) -> std::result::Result<String, std::fmt::Error> {
    use std::fmt::Write as _;
    let mut source = format!("{prefix}// Generated scaling provider.\n");
    for index in 0..n {
        writeln!(source, "pub const C{index:02}: u8 = {index};")?;
    }
    source.push_str("pub fn first() -> u8 { C00 }\n");
    Ok(source)
}
#[test]
fn constant_packets_and_validation_grow_by_a_constant_per_constant() -> Result {
    let branches = crate::test_source::fixture("pa29/tests/spec/source-branches/provider.rs")?;
    let mut prefix = String::new();
    for line in branches.lines().take(57) {
        prefix.push_str(line);
        prefix.push('\n');
    }
    let mut bytes = Vec::new();
    let mut work = Vec::new();
    for n in [8, 16, 32] {
        let interface = exported_profile(
            &scaled(&prefix, n)?,
            SourcePublication::ModuleBodies,
            "constant_scaling",
        )?;
        let encoded = crates::encode(&interface)?;
        bytes.push(encoded.len());
        let (rows, attributes, modules) = raw(&interface)?;
        let availability = interface
            .source_descriptor()
            .ok_or("source descriptor")?
            .availability()?
            .clone();
        let mut steps = g::Work::default();
        g::Payload::from_source_module_bodies(
            interface.identity(),
            rows,
            attributes,
            modules,
            availability,
            0,
            &mut steps,
        )?;
        work.push(steps.steps);
        // A consumer seeds exactly the decoded packet's constant rows.
        let decoded = crates::decode(&encoded)?;
        let descriptor = decoded.source_descriptor().ok_or("decoded descriptor")?;
        assert_eq!(descriptor.rows().constant_rows().count(), n);
    }
    let first = bytes[1] - bytes[0];
    assert_eq!(first % 8, 0);
    let per = first / 8;
    assert_eq!(bytes[2] - bytes[1], 16 * per);
    assert!(per >= ROW, "{per}");
    assert!(
        work[2] - work[1] <= 2 * (work[1] - work[0]) + 64,
        "{work:?}"
    );
    Ok(())
}
