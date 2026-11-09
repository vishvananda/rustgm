//! Version thirteen's nominal header representation byte 3 is SIMD. It is
//! read only under `body_nodes()`, every byte from 4 up stays unknown, and
//! neither the representation nor the header row grows.
use super::*;
use crate::support::crates::{RecordRepr, owner_catalog::DeclarationRef};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Row<T> = std::result::Result<T, &'static str>;

// The representation stays one byte, so the header row keeps its layout.
const _: [(); 1] = [(); size_of::<RecordRepr>()];
const _: [(); 104] = [(); size_of::<s::NominalHeader>()];

const BODIES: s::Profile = s::Profile::Source2024ModuleBodies;
const BRANCHES: s::Profile = s::Profile::Source2024ModuleBranches;
const UNKNOWN: &str = "unknown source nominal representation";

fn header(repr: RecordRepr) -> s::NominalHeader {
    s::NominalHeader {
        contract: s::HeaderContract {
            declaration: DeclarationRef(1),
            declaration_site: 2,
            must_use: None,
            inline: None,
            cold: false,
            caller_tracking: false,
            target_features: None,
        },
        declaration: DeclarationRef(1),
        scope: DeclarationRef(0),
        self_type: g::TypeRef(3),
        fields: g::Range { start: 0, end: 1 },
        inputs: g::Range { start: 0, end: 0 },
        declared_inputs: 0,
        repr,
    }
}
fn written(repr: RecordRepr) -> Result<Vec<u8>> {
    let mut out = Writer::default();
    nominal_rows::nominal_write(&mut out, &header(repr))?;
    Ok(out.finish())
}
/// One header row read under `profile`; a successful read consumes every byte.
fn read(bytes: &[u8], profile: s::Profile) -> Row<s::NominalHeader> {
    let mut reader = Reader::new(bytes);
    let row = nominal_rows::nominal_read(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile,
    })?;
    if !reader.done() {
        return Err("nominal header row left trailing bytes");
    }
    Ok(row)
}

#[test]
fn simd_headers_write_byte_three_and_read_it_only_under_body_nodes() -> Result {
    let bytes = written(RecordRepr::Simd)?;
    // The representation is the row's last byte, after the same fields as
    // every other representation's row.
    let rust = written(RecordRepr::Rust)?;
    assert_eq!(bytes.split_last(), Some((&3, &rust[..rust.len() - 1])));
    assert_eq!(read(&bytes, BODIES)?, header(RecordRepr::Simd));
    assert_eq!(read(&bytes, BRANCHES).err(), Some(UNKNOWN));
    for value in [4, 5, u8::MAX] {
        let mut forged = bytes.clone();
        if let Some(last) = forged.last_mut() {
            *last = value;
        }
        assert_eq!(read(&forged, BODIES).err(), Some(UNKNOWN), "{value}");
    }
    // The earlier representations keep their bytes and gates.
    for (repr, byte, older) in [
        (RecordRepr::Rust, 0, true),
        (RecordRepr::C, 1, true),
        (RecordRepr::Transparent, 2, false),
    ] {
        let row = written(repr)?;
        assert_eq!(row.last(), Some(&byte));
        assert_eq!(read(&row, BODIES)?, header(repr));
        assert_eq!(read(&row, BRANCHES).is_ok(), older, "{repr:?}");
    }
    Ok(())
}
