//! Version thirteen's pointer-call value node, `30 | 4 | Type | Value | Range
//! | Flag`, decodes only with its exact operand kinds and a known safety flag.
//! Every honest node of the provider round-trips. A Range operand resized to a
//! four-byte Value and an unknown flag reject at the value node itself, with
//! the whole node consumed, so no frame or envelope supplies the text.
use super::super::Reader;
use super::*;
use crate::{lower::instances::SourcePublication, support::crates};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Row<T> = std::result::Result<T, &'static str>;

const PROVIDER: &str = "pa29/tests/spec/pointer-calls/provider.rs";
/// The provider's pointer calls, from `pointer-call-outcomes.json`.
const CALLS: usize = 11;
/// Offsets inside one written value: its type id, the node's code and arity,
/// the Type and Value operands, then the Range operand's kind byte and the
/// Flag operand's payload.
const NODE: usize = 4;
const RANGE: usize = NODE + 2 + 5 + 5;
const FLAG: usize = RANGE + 9 + 1;
const WIDTH: usize = FLAG + 4;

/// The provider's own pointer-call value rows.
fn calls() -> Result<Vec<s::Value>> {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = crate::module_packets::exported_profile(
        &source,
        SourcePublication::ModuleBodies,
        "pointer_wire",
    )?;
    let decoded = crates::decode(&crates::encode(&interface)?)?;
    let (rows, _, _) = crate::module_packets::raw(&decoded)?;
    Ok(rows
        .originals
        .into_iter()
        .flat_map(|original| original.values)
        .filter(|value| matches!(value.kind, s::ValueKind::PointerCall { .. }))
        .collect())
}
fn written(row: &s::Value) -> Result<Vec<u8>> {
    let mut out = Writer::default();
    value_write(&mut out, row)?;
    Ok(out.finish())
}
/// The decoded value and whether the reader consumed every byte.
fn read(bytes: &[u8]) -> (Row<s::Value>, bool) {
    let mut reader = Reader::new(bytes);
    let row = value_read(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile: s::Profile::Source2024ModuleBodies,
    });
    (row, reader.done())
}
#[test]
fn pointer_call_nodes_round_trip_and_resized_or_unknown_operands_reject_by_name() -> Result {
    let calls = calls()?;
    assert_eq!(calls.len(), CALLS);
    for row in calls {
        let bytes = written(&row)?;
        assert_eq!(bytes.len(), WIDTH);
        assert_eq!(bytes[NODE..NODE + 3], [30, 4, 4]);
        assert_eq!((bytes[RANGE], bytes[FLAG - 1]), (9, 8));
        assert_eq!(read(&bytes), (Ok(row), true));
        let mut resized = bytes[..RANGE].to_vec();
        resized.push(0);
        resized.extend_from_slice(&bytes[RANGE + 1..RANGE + 5]);
        resized.extend_from_slice(&bytes[FLAG - 1..]);
        assert_eq!(resized.len(), WIDTH - 4);
        assert_eq!(
            read(&resized),
            (Err("body IR value node has malformed operands"), true)
        );
        let mut unknown = bytes;
        unknown[FLAG..WIDTH].copy_from_slice(&3_u32.to_le_bytes());
        assert_eq!(
            read(&unknown),
            (Err("unknown source call safety context"), true)
        );
    }
    Ok(())
}
