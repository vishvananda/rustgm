//! Lexical storage and materialization metadata remain exact structural inputs.
use super::{Body, Export, g, ports};
use crate::body::{scopes, storage};
use ports::Sites;
pub(super) fn emit(
    export: &mut Export<'_>,
    body: &Body,
    raw: &mut g::Original,
    sites: &mut Sites,
) -> Result<(), &'static str> {
    let storage = body
        .storage
        .as_ref()
        .ok_or("generic original has no actual storage scopes")?;
    for scope in &storage.scopes {
        export.reserve(4 * size_of::<g::Scope>())?;
        raw.scopes.push(g::Scope {
            parent: scope.parent.map(|p| p.0),
            site: sites.get(export, scope.source)?,
            kind: kind(scope.kind),
        });
    }
    for exit in &storage.exits {
        export.reserve(4 * size_of::<g::Exit>())?;
        raw.exits.push(g::Exit {
            location: match exit.location {
                storage::Location::At { block, before } => g::Location::At {
                    block: block.0,
                    before,
                },
                storage::Location::Edge { block, target } => g::Location::Edge {
                    block: block.0,
                    target: target.0,
                },
            },
            end: match exit.end {
                storage::End::Scopes(storage::Range { inner, outer }) => g::End::Scopes {
                    inner: inner.0,
                    outer: outer.map(|p| p.0),
                },
                storage::End::Temporaries { start, end } => {
                    g::End::Temporaries(g::Range { start, end })
                }
            },
        });
    }
    for temp in &body.temporaries {
        export.reserve(4 * size_of::<g::Temporary>())?;
        raw.temporaries.push(g::Temporary {
            lifetime: match temp.lifetime {
                scopes::Lifetime::Scope(scope) => g::Lifetime::Scope(scope.0),
                scopes::Lifetime::Static => g::Lifetime::Static,
            },
            value: temp.value.0,
            site: sites.get(export, temp.source)?,
            block: temp.block.0,
            before: temp.before,
            exits_before: temp.exits_before,
        });
    }
    Ok(())
}
fn kind(kind: scopes::Kind) -> g::ScopeKind {
    match kind {
        scopes::Kind::Body => g::ScopeKind::Body,
        scopes::Kind::Block => g::ScopeKind::Block,
        scopes::Kind::Statement => g::ScopeKind::Statement,
        scopes::Kind::Tail => g::ScopeKind::Tail,
        scopes::Kind::Condition => g::ScopeKind::Condition,
        scopes::Kind::IfThen => g::ScopeKind::IfThen,
        scopes::Kind::Iteration => g::ScopeKind::Iteration,
        scopes::Kind::Arm => g::ScopeKind::Arm,
        scopes::Kind::Guard => g::ScopeKind::Guard,
    }
}
