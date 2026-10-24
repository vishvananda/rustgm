//! Borrowed and owned storage keep distinct creation and transfer coordinates.
use super::{Body, Sites, Stage, g, s};
use crate::lower::crate_originals::lexical;

pub(super) fn emit(
    stage: &mut Stage,
    body: &Body,
    raw: &mut s::Original,
    sites: &mut Sites,
) -> Result<(), &'static str> {
    let storage = body
        .storage
        .as_ref()
        .ok_or("source original has no actual lexical storage facts")?;
    for scope in &storage.scopes {
        stage.reserve(4 * size_of::<g::Scope>())?;
        raw.scopes.push(g::Scope {
            parent: scope.parent.map(|parent| parent.0),
            kind: lexical::kind(scope.kind),
            site: sites.body(stage, scope.source)?,
        });
    }
    for exit in &storage.exits {
        stage.reserve(4 * size_of::<g::Exit>())?;
        raw.exits.push(g::Exit {
            location: lexical::location(exit.location),
            end: lexical::end(exit.end),
        });
    }
    for temp in &body.temporaries {
        stage.reserve(4 * size_of::<g::Temporary>())?;
        let site = sites.body(stage, temp.source)?;
        raw.temporaries.push(lexical::temporary(temp, site));
    }
    if let Some(actual) = &body.owned {
        stage.reserve(size_of::<s::OwnedValues>())?;
        let mut owned = s::OwnedValues {
            created: Vec::new(),
            returned: Vec::new(),
        };
        for temp in &actual.created {
            stage.reserve(4 * size_of::<g::Temporary>())?;
            let site = sites.body(stage, temp.source)?;
            owned.created.push(lexical::temporary(temp, site));
        }
        for release in &actual.returned {
            stage.reserve(4 * size_of::<s::Release>())?;
            owned.returned.push(s::Release {
                value: release.value.0,
                location: lexical::location(release.location),
                exits_before: release.exits_before,
            });
        }
        raw.owned = Some(owned);
    }
    Ok(())
}
