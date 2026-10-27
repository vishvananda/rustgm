//! The pinned bytecode is derived from decoded data, not formatted source text.
use super::*;
use std::cell::Cell;
type Result<T = ()> = std::result::Result<T, LexError>;
#[test]
fn default_debug_templates_preserve_first_use_repeats_utf8_and_paid_failures() -> Result {
    let work = Cell::new(0usize);
    let bytes = Cell::new(0usize);
    let charge = |steps, storage| {
        work.set(work.get() + steps);
        bytes.set(bytes.get() + storage);
        Ok(())
    };
    let actual = compile("{:?} (1 << {:?})", 2, false, &charge)?;
    assert_eq!(actual.bytes, b"\xc0\x07 (1 << \xc0\x01)\x00");
    assert_eq!(
        actual.arguments,
        [
            Argument {
                input: 0,
                constructor: FormatConstructor::Debug
            },
            Argument {
                input: 1,
                constructor: FormatConstructor::Debug
            }
        ]
    );
    let actual = compile("{1:?} {0:?} {1:?}", 2, false, &charge)?;
    assert_eq!(
        actual.arguments,
        [
            Argument {
                input: 1,
                constructor: FormatConstructor::Debug
            },
            Argument {
                input: 0,
                constructor: FormatConstructor::Debug
            }
        ]
    );
    assert_eq!(actual.bytes, b"\xc0\x01 \xc0\x01 \xc8\x00\x00\x00");
    let actual = compile("{1}|{0:?}|{1}|{1:?}|{0:?}", 2, false, &charge)?;
    assert_eq!(
        actual.arguments,
        [
            Argument {
                input: 1,
                constructor: FormatConstructor::Display
            },
            Argument {
                input: 0,
                constructor: FormatConstructor::Debug
            },
            Argument {
                input: 1,
                constructor: FormatConstructor::Debug
            }
        ]
    );
    assert_eq!(
        actual.bytes,
        b"\xc0\x01|\xc0\x01|\xc8\x00\x00\x01|\xc8\x02\x00\x01|\xc8\x01\x00\x00"
    );
    let repeated = compile("{}|{0:}|{0}", 1, false, &charge)?;
    assert_eq!(
        repeated.arguments,
        [Argument {
            input: 0,
            constructor: FormatConstructor::Display
        }]
    );
    eprintln!(
        "format pair layouts Argument={} name_arrays={}/{} LibraryValue={} Output={} Template={}",
        size_of::<Argument>(),
        size_of::<[crate::support::names::NameId; 2]>(),
        size_of::<[crate::support::names::NameId; 3]>(),
        size_of::<crate::support::library_types::LibraryValue>(),
        size_of::<super::super::output::Output<'_>>(),
        size_of::<Template>()
    );
    let text = format!("{}{{:?}}", "é".repeat(40_000));
    let actual = compile(&text, 1, false, &charge)?;
    assert_eq!(actual.bytes[0], 0x80);
    assert_eq!(
        u16::from_le_bytes([actual.bytes[1], actual.bytes[2]]),
        65534
    );
    assert_eq!(actual.bytes.last(), Some(&0));
    assert!(work.get() > text.len());
    assert!(bytes.get() > text.len());
    for text in ["{}", "{:x}", "{2:?}", "{:?} }", "{:?}"] {
        assert!(compile(text, 2, false, &charge).is_err(), "{text}");
    }
    let attempts = Cell::new(0);
    assert!(
        compile("{:?}", 1, false, &|_, _| {
            attempts.set(attempts.get() + 1);
            Err(error(0, "frozen failed payment"))
        })
        .is_err()
    );
    assert_eq!(attempts.get(), 1);
    Ok(())
}
