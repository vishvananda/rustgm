//! Captured slices keep token contexts separate from their physical introduction.
use super::*;
use crate::{
    expand::{
        config::Config,
        marks::{ExpansionId, Marks, Transparency},
        stream::{Location, Origin},
    },
    support::{names::Names, source::SourceFile},
    syntax::token_tree::Span,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn unit() -> TestResult<Unit> {
    Ok(Unit::parse(
        SourceFile::new(b"m!();".to_vec())?,
        &Config::default(),
        &mut Names::default(),
    )
    .map_err(|error| format!("{error:?}"))?)
}

fn origin(generated: Span, parent: Span, context: Option<SyntaxContextId>) -> Origin {
    Origin {
        generated,
        original: Location {
            file: FileId(0),
            span: parent,
        },
        context,
        introduction: ExpansionId(2),
        joint: None,
    }
}

#[test]
fn opaque_subranges_keep_distinct_parent_marks_and_physical_root_identity() -> TestResult {
    let mut unit = unit()?;
    let mut marks = Marks::default();
    let root = SyntaxContextId::ROOT;
    let left = marks.apply(root, ExpansionId(1), Transparency::SemiOpaque, root)?;
    let right = marks.apply(root, ExpansionId(2), Transparency::Opaque, root)?;
    let parent = unit.source.append("x y")?;
    for (offset, context) in [(0, left), (2, right)] {
        unit.origins.push(origin(
            Span {
                start: parent + offset,
                end: parent + offset + 1,
            },
            Span { start: 0, end: 1 },
            Some(context),
        ));
    }
    let copy = unit.source.append("x y")?;
    unit.origins.push(origin(
        Span {
            start: copy,
            end: copy + 3,
        },
        Span {
            start: parent,
            end: parent + 3,
        },
        None,
    ));
    let units = [unit];
    assert_eq!(at(&units, FileId(0), 0)?, root);
    assert_eq!(at(&units, FileId(0), copy)?, left);
    assert_eq!(at(&units, FileId(0), copy + 2)?, right);
    assert_eq!(at(&units, FileId(0), parent)?, left);
    assert!(at(&units, FileId(0), copy + 4).is_err());
    Ok(())
}

#[test]
fn malformed_and_cyclic_context_origins_fail_without_physical_fallback() -> TestResult {
    let mut unit = unit()?;
    let start = unit.source.append("x")?;
    let span = Span {
        start,
        end: start + 1,
    };
    unit.origins
        .push(origin(span, Span { start: 3, end: 1 }, None));
    let mut units = [unit];
    assert_eq!(
        at(&units, FileId(0), start)
            .err()
            .ok_or("reversed origin accepted")?
            .message,
        "reversed token context origin"
    );
    units[0].origins[0].original.span = span;
    assert_eq!(
        at(&units, FileId(0), start)
            .err()
            .ok_or("cycle accepted")?
            .message,
        "token context ancestry exceeds 4096"
    );
    assert!(at(&units, FileId(1), start).is_err());
    Ok(())
}
