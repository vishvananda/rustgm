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
        size_of::<[crate::support::names::NameId; 3]>(),
        size_of::<[crate::support::names::NameId; 4]>(),
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

#[test]
fn static_options_keep_exact_flags_counts_positions_and_trait_pairs() -> Result {
    let charge = |_, _| Ok(());
    assert_eq!(compile("{:}>", 1, false, &charge)?.bytes, b"\xc0\x01>\x00");
    let actual = compile("FieldId({:#034x})", 1, false, &charge)?;
    assert_eq!(
        actual.bytes,
        b"\x08FieldId(\xc3\x20\x00\x80\x69\x22\x00\x01)\x00"
    );
    assert_eq!(
        actual.arguments,
        [Argument {
            input: 0,
            constructor: FormatConstructor::LowerHex
        }]
    );
    let actual = compile("{0:#034x}|{0:x}|{0:?}|{0}", 1, false, &charge)?;
    assert_eq!(actual.arguments.len(), 3);
    assert_eq!(
        actual.bytes,
        b"\xc3\x20\x00\x80\x69\x22\x00\x01|\xc8\x00\x00\x01|\xc0\x01|\xc0\x00"
    );
    assert_eq!(
        compile("{:00x}", 1, false, &charge)?.bytes,
        b"\xc1\x20\x00\x00\x69\x00"
    );
    assert_eq!(compile("{:.x}", 1, false, &charge)?.bytes, b"\xc0\x00");
    assert_eq!(
        compile("{:*>8.3}", 1, false, &charge)?.bytes,
        b"\xc7\x2a\x00\x00\x38\x08\x00\x03\x00\x00"
    );
    assert_eq!(
        compile("{:🦀^7}", 1, false, &charge)?.bytes,
        b"\xc3\x80\xf9\x01\x48\x07\x00\x00"
    );
    assert_eq!(
        compile("{:#x?}", 1, false, &charge)?.bytes,
        b"\xc1\x20\x00\x80\x62\x00"
    );
    for text in [
        "{:65536x}",
        "{:width$x}",
        "{:0$x}",
        "{:.*x}",
        "{named:x}",
        "{:X}",
        "{:8#x}",
    ] {
        assert!(compile(text, 1, false, &charge).is_err(), "{text}");
    }
    let mut paid = Vec::new();
    for newline in [false, true] {
        let storage = Cell::new(0);
        let error = compile("unused", 1, newline, &|_, bytes| {
            storage.set(storage.get() + bytes);
            Ok(())
        })
        .err()
        .ok_or_else(|| error(0, "unused input accepted"))?;
        assert!(error.message.contains("unused argument"));
        paid.push(storage.get());
    }
    assert_eq!(paid[1] - paid[0], 1);
    eprintln!(
        "static options={} pair={} names={} Output={} LibraryValue={} Template={}",
        size_of::<Options>(),
        size_of::<Argument>(),
        size_of::<[crate::support::names::NameId; 4]>(),
        size_of::<super::super::output::Output<'_>>(),
        size_of::<crate::support::library_types::LibraryValue>(),
        size_of::<Template>()
    );
    Ok(())
}
