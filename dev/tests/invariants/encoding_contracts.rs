//! The pinned bytecode is derived from decoded data, not formatted source text.
use super::*;
use std::cell::Cell;
type Result<T = ()> = std::result::Result<T, LexError>;
#[test]
fn complete_constructor_and_count_pairs_preserve_source_recipe_control_and_data() -> Result {
    let pay = |_, _| Ok(());
    let all = compile(
        "{0}|{0:?}|{0:o}|{0:x}|{0:X}|{0:p}|{0:b}|{0:e}|{0:E}",
        1,
        false,
        &pay,
    )?;
    assert_eq!(
        all.arguments
            .iter()
            .map(|arg| arg.constructor)
            .collect::<Vec<_>>(),
        [
            FormatConstructor::Display,
            FormatConstructor::Debug,
            FormatConstructor::Octal,
            FormatConstructor::LowerHex,
            FormatConstructor::UpperHex,
            FormatConstructor::Pointer,
            FormatConstructor::Binary,
            FormatConstructor::LowerExp,
            FormatConstructor::UpperExp
        ]
    );
    dynamic_count_recipe()?;
    constructor_family_layouts();
    complete_family_scaling()?;
    Ok(())
}
fn dynamic_count_recipe() -> Result {
    let pay = |_, _| Ok(());
    let named = compile_named("{value:width$.precision$}", 0, &[], true, false, &pay)?;
    assert_eq!(named.captured, ["value", "precision", "width"]);
    assert_eq!(
        named.arguments,
        [
            Argument {
                input: 0,
                constructor: FormatConstructor::Display
            },
            Argument {
                input: 2,
                constructor: FormatConstructor::Count
            },
            Argument {
                input: 1,
                constructor: FormatConstructor::Count
            }
        ]
    );
    assert_eq!(named.bytes, b"\xf7\x20\x00\x00\x78\x01\x00\x02\x00\x00");
    let star = compile("{:.*}|{}", 3, false, &pay)?;
    assert_eq!(
        star.arguments,
        [
            Argument {
                input: 1,
                constructor: FormatConstructor::Display
            },
            Argument {
                input: 0,
                constructor: FormatConstructor::Count
            },
            Argument {
                input: 2,
                constructor: FormatConstructor::Display
            }
        ]
    );
    assert_eq!(
        star.bytes,
        b"\xe5\x20\x00\x00\x70\x01\x00\x01|\xc8\x02\x00\x00"
    );
    let shared = compile("{0:0$}", 1, false, &pay)?;
    assert_eq!(
        shared.arguments,
        [
            Argument {
                input: 0,
                constructor: FormatConstructor::Display
            },
            Argument {
                input: 0,
                constructor: FormatConstructor::Count
            }
        ]
    );
    assert_eq!(shared.bytes, b"\xd3\x20\x00\x00\x68\x01\x00\x00");
    let mut chars = Characters::new("0$}");
    let (_, mut raw) = options::parse(&mut chars, &mut 0)?;
    let mut inputs = named::Inputs::new(1, &[], false, &pay)?;
    raw.bind(&mut inputs)?;
    let zero = raw.encoded(|_| Ok(0))?;
    let mut bytes = Vec::new();
    zero.emit(0, 0, &mut bytes);
    assert_eq!(bytes, b"\xd3\x20\x00\x00\x68\x00\x00");
    for text in [
        "{:*}",
        "{:.*}",
        "{:1$}",
        "{:65536}",
        "{:unknown$}",
        "{0:0$q}",
    ] {
        assert!(compile(text, 1, false, &pay).is_err(), "{text}");
    }
    Ok(())
}
fn constructor_family_layouts() {
    assert_eq!(size_of::<super::super::output::Output<'_>>(), 472);
    assert_eq!(size_of::<Options<'_>>(), 56);
    assert_eq!(size_of::<options::Encoded>(), 12);
    assert_eq!(size_of::<Template<'_>>(), 72);
    assert_eq!(size_of::<crate::support::library_types::LibraryValue>(), 12);
    assert_eq!(size_of::<crate::syntax::ast::Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::expand::stream::Origin>(), 32);
    eprintln!(
        "constructor family names={} slots={} pair={} Output={} rawOptions={} encodedOptions={} Template={} LibraryValue={}",
        size_of::<[crate::support::names::NameId; FormatConstructor::ALL.len()]>(),
        size_of::<[Option<u16>; FormatConstructor::PAIR_COUNT]>(),
        size_of::<Argument>(),
        size_of::<super::super::output::Output<'_>>(),
        size_of::<Options<'_>>(),
        size_of::<options::Encoded>(),
        size_of::<Template<'_>>(),
        size_of::<crate::support::library_types::LibraryValue>()
    );
}

fn complete_family_scaling() -> Result {
    let work = Cell::new(0usize);
    let bytes = Cell::new(0usize);
    let pay = |steps, storage| {
        work.set(work.get() + steps);
        bytes.set(bytes.get() + storage);
        Ok(())
    };
    let mut prior = None;
    for count in [0usize, 8, 32, 128] {
        let mut text = String::new();
        for n in 0..count {
            use std::fmt::Write;
            for suffix in ["", "?", "x", "o", "X", "p", "b", "e", "E"] {
                write!(&mut text, "{{value{n}:value{n}${suffix}}};")
                    .map_err(|_| error(0, "test template formatting failed"))?;
            }
        }
        work.set(0);
        bytes.set(0);
        let actual = compile_named(&text, 0, &[], true, false, &pay)?;
        assert_eq!(actual.captured.len(), count);
        assert_eq!(
            actual.arguments.len(),
            count * FormatConstructor::PAIR_COUNT
        );
        assert!(actual.arguments.capacity() >= actual.arguments.len());
        if let Some((old_work, old_bytes)) = prior {
            assert!(work.get() <= old_work * 6);
            assert!(bytes.get() <= old_bytes * 6);
        }
        if count > 0 {
            prior = Some((work.get(), bytes.get()));
        }
        eprintln!(
            "complete constructor inputs={count} work={} bytes={} pairs={} capacity={}",
            work.get(),
            bytes.get(),
            actual.arguments.len(),
            actual.arguments.capacity()
        );
    }
    let calls = Cell::new(0usize);
    compile("{0:0$}", 1, false, &|_, _| {
        calls.set(calls.get() + 1);
        Ok(())
    })?;
    for stopped in 0..calls.get() {
        let attempted = Cell::new(0usize);
        assert!(
            compile("{0:0$}", 1, false, &|_, _| {
                let current = attempted.get();
                attempted.set(current + 1);
                if current == stopped {
                    Err(error(0, "frozen pair payment failure"))
                } else {
                    Ok(())
                }
            })
            .is_err()
        );
        assert_eq!(attempted.get(), stopped + 1);
    }
    Ok(())
}

#[test]
fn named_and_captured_inputs_keep_first_use_pairs_and_paid_scaling() -> Result {
    assert_eq!(size_of::<crate::expand::stream::Origin>(), 32);
    assert_eq!(size_of::<crate::expand::stream::OriginKind>(), 2);
    assert_eq!(size_of::<crate::syntax::ast::Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::support::library_types::LibraryValue>(), 12);
    eprintln!(
        "named format owners Output={} Draft={} Template={} Characters={}",
        size_of::<super::super::output::Output<'_>>(),
        size_of::<super::super::Draft>(),
        size_of::<Template<'_>>(),
        size_of::<Characters<'_>>()
    );
    let work = Cell::new(0usize);
    let bytes = Cell::new(0usize);
    let pay = |steps, storage| {
        work.set(work.get() + steps);
        bytes.set(bytes.get() + storage);
        Ok(())
    };
    let named = compile_named(
        "FieldId({right:#034x})|{left:?}|{right:x}|{right}|{left:?}",
        2,
        &[Some("left"), Some("right")],
        false,
        false,
        &pay,
    )?;
    let positional = compile("FieldId({1:#034x})|{0:?}|{1:x}|{1}|{0:?}", 2, false, &pay)?;
    assert_eq!(named.bytes, positional.bytes);
    assert_eq!(named.arguments, positional.arguments);
    let captured = compile_named(
        "FieldId({right:#034x})|{left:?}|{right:x}|{right}|{left:?}",
        0,
        &[],
        true,
        false,
        &pay,
    )?;
    assert_eq!(captured.captured, ["right", "left"]);
    assert_eq!(captured.bytes, positional.bytes);
    assert_eq!(
        captured
            .arguments
            .iter()
            .map(|arg| arg.input)
            .collect::<Vec<_>>(),
        [0, 1, 0]
    );
    for (text, names, capture) in [
        ("{missing}", vec![], false),
        ("literal", vec![Some("unused")], true),
        ("{same}", vec![Some("same"), Some("same")], true),
        ("{same} {}", vec![Some("same"), None], true),
        ("{0}", vec![], true),
        ("{1name}", vec![None], true),
        ("{r#type}", vec![], true),
    ] {
        assert!(
            compile_named(text, names.len(), &names, capture, false, &pay).is_err(),
            "{text}"
        );
    }
    let mut prior = None;
    for count in [8usize, 32, 128] {
        let mut text = String::new();
        for n in 0..count {
            use std::fmt::Write;
            write!(&mut text, "{{value{n}}}|{{value{n}:?}}|{{value{n}:#034x}};").map_err(|_| {
                crate::syntax::token_tree::error(0, "test template formatting failed")
            })?;
        }
        work.set(0);
        bytes.set(0);
        let output = compile_named(&text, 0, &[], true, false, &pay)?;
        assert_eq!(output.captured.len(), count);
        assert_eq!(output.arguments.len(), count * 3);
        if let Some((old_work, old_bytes)) = prior {
            assert!(work.get() <= old_work * 6);
            assert!(bytes.get() <= old_bytes * 6);
        }
        eprintln!(
            "named capture count={count} work={} bytes={} Template={} Inputs={}",
            work.get(),
            bytes.get(),
            size_of::<Template<'_>>(),
            size_of::<named::Inputs<'_, '_>>()
        );
        prior = Some((work.get(), bytes.get()));
    }
    assert!(
        compile_named("{value}", 0, &[], true, false, &|_, _| Err(
            crate::syntax::token_tree::error(0, "payment stopped")
        ))
        .is_err()
    );
    Ok(())
}
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
        size_of::<Template<'_>>()
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
    for text in ["{:65536x}", "{:width$x}", "{:.*x}", "{named:x}", "{:8#x}"] {
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
        size_of::<Options<'_>>(),
        size_of::<Argument>(),
        size_of::<[crate::support::names::NameId; 4]>(),
        size_of::<super::super::output::Output<'_>>(),
        size_of::<crate::support::library_types::LibraryValue>(),
        size_of::<Template<'_>>()
    );
    Ok(())
}
