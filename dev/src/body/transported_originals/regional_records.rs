//! Version-nine places retain real body-local regions and ordinary borrow proof.
use super::ImportedOriginals;
use crate::{
    body::ir::{
        Action, Body, BorrowKind, LocalId, Place, Projection, ProjectionId, ProjectionKind, Rvalue,
    },
    sema::crates::SourceOriginal,
    support::crates::generics::source as s,
};

pub(super) fn require(view: &SourceOriginal<'_>) -> Result<(), &'static str> {
    if !view.rows().descriptor().profile().conversion_roles() || !view.numbered() {
        return Err("borrowed ordinary source body requires its actual version-nine numbering");
    }
    Ok(())
}

pub(super) fn allocation(raw: &s::Original) -> usize {
    raw.projections
        .len()
        .saturating_mul(size_of::<Projection>())
}

pub(super) fn place(view: &SourceOriginal<'_>, raw: s::Place) -> Result<Place, &'static str> {
    match raw {
        s::Place::Local(index) => {
            if index as usize >= view.raw().locals.len() {
                return Err("source place lost its actual local identity");
            }
            Ok(Place::Local(LocalId(index)))
        }
        s::Place::Projection(index) => {
            require(view)?;
            if index as usize >= view.raw().projections.len() {
                return Err("source place lost its actual projection identity");
            }
            Ok(Place::Projection(ProjectionId(index)))
        }
    }
}

pub(super) fn borrow(view: &SourceOriginal<'_>, raw: s::Place) -> Result<Rvalue, &'static str> {
    require(view)?;
    Ok(Rvalue::Borrow {
        place: place(view, raw)?,
        kind: BorrowKind::Immediate,
    })
}

pub(super) fn project(view: &SourceOriginal<'_>, index: u32) -> Result<Action, &'static str> {
    let Place::Projection(id) = place(view, s::Place::Projection(index))? else {
        return Err("source projection statement changed its actual place kind");
    };
    Ok(Action::Project(id))
}

fn projection(
    view: &SourceOriginal<'_>,
    raw: s::Projection,
    steps: &mut usize,
) -> Result<Projection, &'static str> {
    Ok(Projection {
        base: place(view, raw.base)?,
        ty: view.ty(raw.ty, steps)?,
        kind: ProjectionKind::Dereference {
            region: view.region(raw.region, steps)?,
            mutable: raw.mutable,
        },
    })
}

/// The aggregate original preflight pays `allocation` before this allocation.
pub(super) fn projections(
    view: &SourceOriginal<'_>,
    work: &mut ImportedOriginals,
) -> Result<Vec<Projection>, &'static str> {
    let rows = &view.raw().projections;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    require(view)?;
    work.charge(rows.len())?;
    let mut result = Vec::with_capacity(rows.len());
    for &raw in rows {
        let actual = projection(view, raw, &mut work.steps)?;
        work.charge(0)?;
        result.push(actual);
    }
    Ok(result)
}

pub(super) fn validate(
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let rows = &view.raw().projections;
    if rows.len() != body.projections.len() {
        return Err("source original changed its complete projection domain");
    }
    if !rows.is_empty() {
        require(view)?;
    }
    for (&raw, actual) in rows.iter().zip(&body.projections) {
        *steps = steps.saturating_add(1);
        let expected = projection(view, raw, steps)?;
        let same = match (actual.kind, expected.kind) {
            (
                ProjectionKind::Dereference {
                    region: a,
                    mutable: am,
                },
                ProjectionKind::Dereference {
                    region: b,
                    mutable: bm,
                },
            ) => a == b && am == bm,
            _ => false,
        };
        if !same || actual.base != expected.base || actual.ty != expected.ty {
            return Err("source original changed an actual dereference place/type/region");
        }
    }
    Ok(())
}
