use super::*;
use crate::{
    expand::config::Config,
    support::{names::Names, source::SourceFile},
};

fn unit(edition: Edition) -> Result<Unit, String> {
    Unit::parse(
        SourceFile::new(b"fn r#gen() {}".to_vec())?,
        &Config::in_edition(edition),
        &mut Names::default(),
    )
    .map_err(|error| format!("{error:?}"))
}
fn origin(generated: Span, file: FileId, span: Span) -> Origin {
    Origin {
        generated,
        original: Location { file, span },
        context: None,
        joint: None,
    }
}

#[test]
fn original_and_fresh_physical_sources_keep_their_edition_without_line_index() -> Result<(), String>
{
    let source = unit(Edition::Rust2021)?;
    let fresh = source
        .fresh_instance()
        .map_err(|error| format!("{error:?}"))?;
    for source in [&source, &fresh] {
        assert_eq!(
            at(
                Location {
                    file: FileId(0),
                    span: Span { start: 5, end: 8 }
                },
                |_| Some(source)
            ),
            Ok(Edition::Rust2021)
        );
    }
    Ok(())
}

#[test]
fn exact_generated_subranges_follow_original_files_and_reject_gaps() -> Result<(), String> {
    let first = unit(Edition::Rust2021)?;
    let mut second = unit(Edition::Rust2024)?;
    let start = second.source.append("r#gen").map_err(str::to_owned)?;
    second.origins.push(origin(
        Span {
            start,
            end: start + 5,
        },
        FileId(0),
        Span { start: 3, end: 8 },
    ));
    let units = [first, second];
    let lookups = std::cell::Cell::new(0);
    let lookup = |file: FileId| {
        lookups.set(lookups.get() + 1);
        units.get(file.0 as usize)
    };
    let span = Span {
        start: start + 2,
        end: start + 5,
    };
    assert_eq!(
        at(
            Location {
                file: FileId(1),
                span
            },
            lookup
        ),
        Ok(Edition::Rust2021)
    );
    assert_eq!(lookups.get(), 2, "one lookup per ancestor, no package scan");
    for span in [
        Span {
            start: start - 1,
            end: start,
        },
        Span {
            start,
            end: start + 6,
        },
        Span { start, end: start },
    ] {
        assert!(
            at(
                Location {
                    file: FileId(1),
                    span
                },
                lookup
            )
            .is_err()
        );
    }
    assert!(
        at(
            Location {
                file: FileId(9),
                span
            },
            lookup
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn malformed_or_cyclic_provenance_never_falls_back_to_the_consumer() -> Result<(), String> {
    let mut source = unit(Edition::Rust2024)?;
    let start = source.source.append("gen").map_err(str::to_owned)?;
    let span = Span {
        start,
        end: start + 3,
    };
    source.origins.push(origin(span, FileId(0), span));
    let query = Location {
        file: FileId(0),
        span,
    };
    let lookups = std::cell::Cell::new(0);
    let error = at(query, |_| {
        lookups.set(lookups.get() + 1);
        Some(&source)
    })
    .err()
    .ok_or("expected bounded cycle failure")?;
    assert!(error.message.contains("4096"));
    assert_eq!(lookups.get(), 4096);
    source.origins[0].original.span.end += 1;
    assert!(at(query, |_| Some(&source)).is_err());
    source.origins.clear();
    assert!(at(query, |_| Some(&source)).is_err());
    Ok(())
}

#[test]
fn parsed_introducer_uses_its_origin_without_demanding_the_complete_syntax_range()
-> Result<(), String> {
    let first = unit(Edition::Rust2021)?;
    let mut second = unit(Edition::Rust2024)?;
    let start = second.source.append("gen r#gen").map_err(str::to_owned)?;
    second.origins.push(origin(
        Span {
            start,
            end: start + 3,
        },
        FileId(0),
        Span { start: 5, end: 8 },
    ));
    second.origins.push(origin(
        Span {
            start: start + 4,
            end: start + 9,
        },
        FileId(1),
        Span { start: 3, end: 8 },
    ));
    let units = [first, second];
    let lookups = std::cell::Cell::new(0);
    let lookup = |file: FileId| {
        lookups.set(lookups.get() + 1);
        units.get(file.0 as usize)
    };
    assert_eq!(
        introducer(
            FileId(1),
            Span {
                start,
                end: start + 9
            },
            lookup
        ),
        Ok(Edition::Rust2021)
    );
    assert_eq!(lookups.get(), 2);
    assert!(
        at(
            Location {
                file: FileId(1),
                span: Span {
                    start,
                    end: start + 9
                }
            },
            lookup
        )
        .is_err()
    );
    assert!(
        introducer(
            FileId(1),
            Span {
                start: u32::MAX,
                end: u32::MAX
            },
            lookup
        )
        .is_err()
    );
    Ok(())
}
