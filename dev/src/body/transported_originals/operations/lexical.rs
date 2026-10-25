//! Physical scope/end/temporary records preserve the source body's exact order.
use super::{BlockId, Body, BodySite, Temporary, ValueId, g, scopes, storage};
#[derive(Clone, Copy)]
pub(in crate::body::transported_originals) struct Records<'a> {
    pub scopes: &'a [g::Scope],
    pub exits: &'a [g::Exit],
    pub temporaries: &'a [g::Temporary],
}
pub(super) fn fill(
    record: &g::Original,
    body: &mut Body,
    site: &impl Fn(u32) -> Result<BodySite, &'static str>,
) -> Result<(), &'static str> {
    fill_records(
        Records {
            scopes: &record.scopes,
            exits: &record.exits,
            temporaries: &record.temporaries,
        },
        body,
        &mut |index| site(index),
    )
}
pub(in crate::body::transported_originals) fn fill_records(
    record: Records<'_>,
    body: &mut Body,
    site: &mut impl FnMut(u32) -> Result<BodySite, &'static str>,
) -> Result<(), &'static str> {
    let scopes = record
        .scopes
        .iter()
        .map(|scope| {
            Ok(storage::Scope {
                parent: scope.parent.map(scopes::Id),
                source: site(scope.site)?,
                kind: scope_kind(scope.kind),
            })
        })
        .collect::<Result<_, &'static str>>()?;
    let exits = record
        .exits
        .iter()
        .map(|exit| storage::Exit {
            location: location(exit.location),
            end: match exit.end {
                g::End::Scopes { inner, outer } => storage::End::Scopes(storage::Range {
                    inner: scopes::Id(inner),
                    outer: outer.map(scopes::Id),
                }),
                g::End::Temporaries(range) => storage::End::Temporaries {
                    start: range.start,
                    end: range.end,
                },
            },
        })
        .collect();
    body.storage = Some(storage::Facts {
        scope_work: record.scopes.len(),
        scopes,
        exits,
    });
    body.temporaries = record
        .temporaries
        .iter()
        .map(|temp| temporary(temp, site))
        .collect::<Result<_, &'static str>>()?;
    Ok(())
}
pub(in crate::body::transported_originals) fn temporary(
    temp: &g::Temporary,
    site: &mut impl FnMut(u32) -> Result<BodySite, &'static str>,
) -> Result<Temporary, &'static str> {
    Ok(Temporary {
        lifetime: match temp.lifetime {
            g::Lifetime::Scope(scope) => scopes::Lifetime::Scope(scopes::Id(scope)),
            g::Lifetime::Static => scopes::Lifetime::Static,
        },
        value: ValueId(temp.value),
        source: site(temp.site)?,
        block: BlockId(temp.block),
        before: temp.before,
        exits_before: temp.exits_before,
    })
}
pub(in crate::body::transported_originals) fn location(record: g::Location) -> storage::Location {
    match record {
        g::Location::At { block, before } => storage::Location::At {
            block: BlockId(block),
            before,
        },
        g::Location::Edge { block, target } => storage::Location::Edge {
            block: BlockId(block),
            target: BlockId(target),
        },
    }
}
pub(in crate::body::transported_originals) fn scope_kind(kind: g::ScopeKind) -> scopes::Kind {
    match kind {
        g::ScopeKind::Body => scopes::Kind::Body,
        g::ScopeKind::Block => scopes::Kind::Block,
        g::ScopeKind::Statement => scopes::Kind::Statement,
        g::ScopeKind::Tail => scopes::Kind::Tail,
        g::ScopeKind::Condition => scopes::Kind::Condition,
        g::ScopeKind::IfThen => scopes::Kind::IfThen,
        g::ScopeKind::Iteration => scopes::Kind::Iteration,
        g::ScopeKind::Arm => scopes::Kind::Arm,
        g::ScopeKind::Guard => scopes::Kind::Guard,
    }
}
