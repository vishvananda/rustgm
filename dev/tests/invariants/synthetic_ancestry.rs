//! Synthetic vocabulary edition and copied opaque bytes have different witnesses.
use super::{NodeKey, TestResult};
use crate::{
    expand::{
        config::Config,
        marks::{ExpansionId, Marks, SyntaxContextId, Transparency},
        stream::{Location, OriginKind, Stream},
    },
    resolve::{
        expansion::Report,
        unit::{FileId, Unit, editions},
    },
    support::{edition::Edition, names::Names, source::SourceFile},
    syntax::token_tree::Span,
};

pub(super) fn units() -> TestResult {
    for (definition, caller) in [
        (Edition::Rust2021, Edition::Rust2024),
        (Edition::Rust2024, Edition::Rust2021),
    ] {
        let make = |edition| {
            Unit::parse(
                SourceFile::new(b"fn r#gen() {} // \xce\xa9".to_vec())?,
                &Config::in_edition(edition),
                &mut Names::default(),
            )
            .map_err(|e| e.message.to_string())
        };
        let first = make(definition)?;
        let mut second = make(caller)?;
        let mut marks = Marks::default();
        let context = marks.apply(
            SyntaxContextId::ROOT,
            ExpansionId(1),
            Transparency::Opaque,
            SyntaxContextId::ROOT,
        )?;
        let mut stream = Stream::default();
        let range = stream
            .synthetic_image(
                "let",
                Location {
                    file: FileId(0),
                    span: Span { start: 3, end: 8 },
                },
                context,
                ExpansionId(1),
                definition,
            )
            .map_err(|e| e.message)?;
        assert!(editions::parent(&stream.origins, range).is_err());
        let base = second.source.append(&stream.source)?;
        let mut origin = stream.origins[0];
        origin.generated.start += base;
        origin.generated.end += base;
        second.origins.push(origin);
        let mut units = [first, second];
        let location = Location {
            file: FileId(1),
            span: origin.generated,
        };
        assert_eq!(
            editions::at(location, |id| units.get(id.0 as usize)),
            Ok(definition)
        );
        for damaged in [
            crate::expand::stream::Origin {
                introduction: ExpansionId(0),
                ..origin
            },
            crate::expand::stream::Origin {
                context: None,
                ..origin
            },
            crate::expand::stream::Origin {
                context: Some(SyntaxContextId::ROOT),
                ..origin
            },
            crate::expand::stream::Origin {
                original: Location {
                    file: FileId(2),
                    span: Span { start: 3, end: 8 },
                },
                ..origin
            },
            crate::expand::stream::Origin {
                original: Location {
                    file: FileId(0),
                    span: Span { start: 17, end: 18 },
                },
                ..origin
            },
        ] {
            units[1].origins[0] = damaged;
            assert!(editions::at(location, |id| units.get(id.0 as usize)).is_err());
        }
        units[1].origins[0] = origin;
        let fresh = units[1].fresh_instance().map_err(|e| e.message)?;
        assert!(fresh.origins.is_empty());
        assert_eq!(fresh.source.text(), fresh.source.original());
    }
    assert_eq!(size_of::<crate::expand::stream::Origin>(), 32);
    assert_eq!(size_of::<OriginKind>(), 2);
    assert_eq!(size_of::<Stream>(), 104);
    assert_eq!(size_of::<Unit>(), 688);
    eprintln!(
        "synthetic origin layouts Origin={} align={} Kind={} Location={} Stream={} Unit={}",
        size_of::<crate::expand::stream::Origin>(),
        align_of::<crate::expand::stream::Origin>(),
        size_of::<OriginKind>(),
        size_of::<Location>(),
        size_of::<Stream>(),
        size_of::<Unit>()
    );
    Ok(())
}

pub(super) fn replay(report: &mut Report, owner: NodeKey, effect: usize) -> TestResult {
    let file = owner.file.0 as usize;
    let index = report.package.units[file]
        .origins
        .iter()
        .position(|o| matches!(o.kind, OriginKind::Synthetic(_)))
        .ok_or("actual synthetic origin")?;
    let original = report.package.units[file].origins[index];
    let foreign = report
        .invocations
        .iter()
        .find(|i| i.original != original.original.span)
        .ok_or("other actual invocation")?
        .context;
    let OriginKind::Synthetic(edition) = original.kind else {
        return Err("synthetic origin".into());
    };
    let other = if edition == Edition::Rust2021 {
        Edition::Rust2024
    } else {
        Edition::Rust2021
    };
    for changed in [
        crate::expand::stream::Origin {
            kind: OriginKind::Synthetic(other),
            ..original
        },
        crate::expand::stream::Origin {
            introduction: foreign,
            ..original
        },
        crate::expand::stream::Origin {
            introduction: ExpansionId(0),
            ..original
        },
        crate::expand::stream::Origin {
            context: Some(SyntaxContextId::ROOT),
            ..original
        },
        crate::expand::stream::Origin {
            original: Location {
                file: FileId(u32::MAX),
                span: original.original.span,
            },
            ..original
        },
        crate::expand::stream::Origin {
            kind: OriginKind::Copied,
            ..original
        },
    ] {
        report.package.units[file].origins[index] = changed;
        assert!(report.expanded_derive(owner, effect).is_err());
        report.package.units[file].origins[index] = original;
        report.expanded_derive(owner, effect)?;
    }
    Ok(())
}
