//! Version thirteen's const rows: declaration tag 12, slot tag 3, argument
//! tag 2 and type tag 14, each with its in-place term. Each is admitted only
//! under `body_nodes()`, and every other term tag is rejected by name.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Row<T> = std::result::Result<T, &'static str>;

// A const argument's term lives in the cold column, so no row grows.
const _: [(); 20] = [(); size_of::<s::Type>()];
const _: [(); 8] = [(); size_of::<s::Argument>()];
const _: [(); 12] = [(); size_of::<s::ParameterKind>()];
const _: [(); 40] = [(); size_of::<s::ParameterSlot>()];
// An array's length term lives in its own cold column: one Vec, 24 B.
const _: [(); 224] = [(); size_of::<s::ColdColumns>()];

const BODIES: s::Profile = s::Profile::Source2024ModuleBodies;
const BRANCHES: s::Profile = s::Profile::Source2024ModuleBranches;

fn written(row: impl FnOnce(&mut Writer) -> Row<()>) -> Result<Vec<u8>> {
    let mut out = Writer::default();
    row(&mut out)?;
    Ok(out.finish())
}
/// One row read under `profile`; a successful read consumes every byte.
fn read<T>(
    bytes: &[u8],
    profile: s::Profile,
    row: impl FnOnce(&mut Input<'_, '_>) -> Row<T>,
) -> Row<T> {
    let mut reader = Reader::new(bytes);
    let value = row(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile,
    })?;
    if !reader.done() {
        return Err("const row left trailing bytes");
    }
    Ok(value)
}
fn arguments(bytes: &[u8], profile: s::Profile) -> Row<(Vec<s::Argument>, Vec<s::ConstTerm>)> {
    let mut consts = Vec::new();
    let rows = read(bytes, profile, |read| {
        read.list(crates::MAX_COMPONENTS, 5, |read| {
            recipe_columns::argument_read(read, &mut consts)
        })
    })?;
    Ok((rows, consts))
}
fn types(bytes: &[u8], profile: s::Profile) -> Row<(Vec<s::Type>, Vec<s::ConstTerm>)> {
    let mut tables = recipe_columns::TypeTables::default();
    let rows = read(bytes, profile, |read| {
        read.list(crates::MAX_COMPONENTS, 2, |read| {
            recipe_columns::type_read(read, &mut tables)
        })
    })?;
    if !tables.ranges.is_empty() {
        return Err("array rows read a pattern range");
    }
    Ok((rows, tables.lengths))
}
fn types_written(rows: &[s::Type], lengths: &[s::ConstTerm]) -> Result<Vec<u8>> {
    written(|out| {
        recipe_columns::types_write(
            out,
            rows,
            &recipe_columns::TypeColumns {
                ranges: &[],
                lengths,
            },
        )
    })
}

#[test]
fn array_rows_write_their_length_terms_in_place_and_read_them_densely() -> Result {
    let array = |length| s::Type::Array {
        element: g::TypeRef(7),
        length: s::ArrayLengthRef(length),
    };
    let rows = [array(0), array(1)];
    let terms = [
        s::ConstTerm::Value(s::ConstValue {
            ty: g::TypeRef(3),
            bits: 4,
        }),
        s::ConstTerm::Parameter(s::SlotRef(2)),
    ];
    let bytes = types_written(&rows, &terms)?;
    // The count, then `14 | element | 0 | ty | bits` and `14 | element | 1 | slot`.
    let mut value = vec![14, 7, 0, 0, 0, 0, 3, 0, 0, 0, 4];
    value.extend([0; 15]);
    assert_eq!(bytes[..4], [2, 0, 0, 0]);
    assert_eq!(bytes[4..30], value[..]);
    assert_eq!(bytes[30..], [14, 7, 0, 0, 0, 1, 2, 0, 0, 0]);
    assert_eq!(types(&bytes, BODIES)?, (rows.to_vec(), terms.to_vec()));
    assert_eq!(
        types(&bytes, BRANCHES).err(),
        Some("unknown source function type recipe")
    );
    let order = "source array lengths are not in type-row order";
    for (rows, lengths, message) in [
        (&rows[..], &terms[..1], order),
        (&[array(1), array(0)][..], &terms[..], order),
        (
            &rows[..1],
            &terms[..],
            "source array length has no type row",
        ),
    ] {
        let error = types_written(rows, lengths).err();
        assert_eq!(
            error.map(|error| error.to_string()),
            Some(message.to_owned())
        );
    }
    Ok(())
}

#[test]
fn const_declarations_slots_and_arguments_round_trip_only_under_body_nodes() -> Result {
    let declaration = s::Declaration {
        definition: s::Definition {
            kind: s::DefinitionKind::ConstParameter,
            parent: Some(DeclarationRef(1)),
        },
        ordinal: 0,
        name: Some("N".to_owned()),
        public: false,
        sites: 1,
        local: None,
    };
    let bytes = written(|out| owner_rows::declaration_write(out, &declaration))?;
    assert_eq!(bytes[0], 12);
    assert_eq!(
        read(&bytes, BODIES, owner_rows::declaration_read)?,
        declaration
    );
    assert_eq!(
        read(&bytes, BRANCHES, owner_rows::declaration_read).err(),
        Some("unknown source declaration kind")
    );
    let mut slot = s::ParameterSlot {
        owner: DeclarationRef(1),
        ordinal: 0,
        index: 0,
        definition: Some(DeclarationRef(2)),
        kind: s::ParameterKind::Const { ty: g::TypeRef(3) },
        default: None,
    };
    let bytes = written(|out| owner_rows::slot_write(out, &slot))?;
    // owner, ordinal and index, the definition option, then `3 | ty`.
    assert_eq!(bytes[12..], [1, 2, 0, 0, 0, 3, 3, 0, 0, 0]);
    assert_eq!(read(&bytes, BODIES, owner_rows::slot_read)?, slot);
    assert_eq!(
        read(&bytes, BRANCHES, owner_rows::slot_read).err(),
        Some("unknown source parameter kind")
    );
    slot.default = Some(g::TypeRef(3));
    assert_eq!(
        written(|out| owner_rows::slot_write(out, &slot))
            .err()
            .map(|error| error.to_string()),
        Some("source const parameter slot has a default".to_owned())
    );
    let rows = [
        s::Argument::Type(g::TypeRef(0)),
        s::Argument::Const(s::ConstRef(0)),
        s::Argument::Const(s::ConstRef(1)),
    ];
    // i8 -1 travels as Constant's masked two's-complement bits.
    let terms = [
        s::ConstTerm::Value(s::ConstValue {
            ty: g::TypeRef(3),
            bits: 0xFF,
        }),
        s::ConstTerm::Parameter(s::SlotRef(4)),
    ];
    let bytes = written(|out| recipe_columns::arguments_write(out, &rows, &terms))?;
    let mut value = vec![2, 0, 3, 0, 0, 0, 0xFF];
    value.extend([0; 15]);
    let parameter = [2, 1, 4, 0, 0, 0];
    assert_eq!(bytes[4..9], [0, 0, 0, 0, 0]);
    assert_eq!(bytes[9..31], value[..]);
    assert_eq!(bytes[31..], parameter);
    // The dense column holds exactly one term per const argument read.
    assert_eq!(arguments(&bytes, BODIES)?, (rows.to_vec(), terms.to_vec()));
    assert_eq!(
        arguments(&bytes, BRANCHES).err(),
        Some("unknown source generic argument kind")
    );
    let order = "source const terms are not in argument-row order";
    let reordered = [
        s::Argument::Type(g::TypeRef(0)),
        s::Argument::Const(s::ConstRef(1)),
        s::Argument::Const(s::ConstRef(0)),
    ];
    for (rows, consts, message) in [
        (&rows, &terms[..1], order),
        (&reordered, &terms[..], order),
        (
            &rows,
            &[terms[0], terms[1], terms[1]][..],
            "source const term has no argument row",
        ),
    ] {
        let error = written(|out| recipe_columns::arguments_write(out, rows, consts)).err();
        assert_eq!(
            error.map(|error| error.to_string()),
            Some(message.to_owned())
        );
    }
    Ok(())
}

#[test]
fn const_terms_reject_every_unknown_tag_by_name() -> Result {
    let term = s::ConstTerm::Value(s::ConstValue {
        ty: g::TypeRef(0),
        bits: 3,
    });
    let bytes = written(|out| recipe_columns::const_term_write(out, term))?;
    assert_eq!(read(&bytes, BODIES, recipe_columns::const_term_read)?, term);
    for tag in [2, 3, 0x80, 0xFF] {
        let mut forged = bytes.clone();
        forged[0] = tag;
        assert_eq!(
            read(&forged, BODIES, recipe_columns::const_term_read).err(),
            Some("unknown source const term"),
            "term tag {tag}"
        );
    }
    Ok(())
}
