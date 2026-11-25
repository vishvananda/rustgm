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

pub(super) fn place(
    view: &SourceOriginal<'_>,
    raw: s::Place,
    steps: &mut usize,
) -> Result<Place, &'static str> {
    match raw {
        s::Place::Static(declaration) => {
            Ok(Place::Static(view.static_storage(declaration, steps)?.0))
        }
        s::Place::Local(index) => {
            if index as usize >= view.raw().locals.len() {
                return Err("source place lost its actual local identity");
            }
            Ok(Place::Local(LocalId(index)))
        }
        s::Place::Temporary(index) => {
            require(view)?;
            if index as usize >= view.raw().values.len() {
                return Err("source snapshot lost its actual SSA value identity");
            }
            Ok(Place::Temporary(crate::body::ir::ValueId(index)))
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

pub(super) fn borrow(
    view: &SourceOriginal<'_>,
    raw: s::Place,
    kind: BorrowKind,
    steps: &mut usize,
) -> Result<Rvalue, &'static str> {
    require(view)?;
    if kind == BorrowKind::TwoPhase && !view.rows().descriptor().profile().body_nodes() {
        return Err("source two-phase borrows require the version-thirteen body IR profile");
    }
    if kind == BorrowKind::UniqueImmutable
        && !view.rows().descriptor().profile().declaration_policies()
    {
        return Err("source unique immutable borrows require the version-fourteen profile");
    }
    Ok(Rvalue::Borrow {
        place: place(view, raw, steps)?,
        kind,
    })
}

pub(super) fn project(
    view: &SourceOriginal<'_>,
    index: u32,
    steps: &mut usize,
) -> Result<Action, &'static str> {
    let Place::Projection(id) = place(view, s::Place::Projection(index), steps)? else {
        return Err("source projection statement changed its actual place kind");
    };
    Ok(Action::Project(id))
}

fn projection(
    view: &SourceOriginal<'_>,
    raw: s::Projection,
    steps: &mut usize,
) -> Result<Projection, &'static str> {
    let v14 = view.rows().descriptor().profile().declaration_policies();
    let kind = match raw.kind {
        s::ProjectionKind::Dereference { region, mutable } => ProjectionKind::Dereference {
            region: view.region(region, steps)?,
            mutable,
        },
        s::ProjectionKind::Field { owner, ordinal } => {
            ProjectionKind::Field(view.nominal_field(owner, ordinal, steps)?)
        }
        s::ProjectionKind::Tuple(ordinal) if v14 => {
            ProjectionKind::Field(crate::sema::members::Selection::Tuple(ordinal))
        }
        s::ProjectionKind::RawDereference { mutable } if v14 => {
            ProjectionKind::RawDereference { mutable }
        }
        s::ProjectionKind::UnionField { owner, ordinal } if v14 => {
            ProjectionKind::UnionField(view.nominal_field(owner, ordinal, steps)?)
        }
        s::ProjectionKind::Index { index, length } if v14 => ProjectionKind::Index {
            index: crate::body::ir::ValueId(index),
            length,
        },
        s::ProjectionKind::IndexParameter { index, length } if v14 => {
            ProjectionKind::IndexParameter {
                index: crate::body::ir::ValueId(index),
                length: view.const_argument(length, steps)?,
            }
        }
        s::ProjectionKind::SliceIndex { index } if v14 => ProjectionKind::SliceIndex {
            index: crate::body::ir::ValueId(index),
        },
        s::ProjectionKind::ConstantIndex(offset) if v14 => ProjectionKind::ConstantIndex(offset),
        s::ProjectionKind::FromEnd { offset, minimum } if v14 => {
            ProjectionKind::FromEnd { offset, minimum }
        }
        s::ProjectionKind::Subslice { prefix, suffix } if v14 => {
            ProjectionKind::Subslice { prefix, suffix }
        }
        s::ProjectionKind::Subarray { start, end } if v14 => {
            ProjectionKind::Subarray { start, end }
        }
        s::ProjectionKind::CaptureBoundary { mutable } if v14 => {
            ProjectionKind::CaptureBoundary { mutable }
        }
        _ => return Err("source extended projection requires the version-fourteen profile"),
    };
    Ok(Projection {
        base: place(view, raw.base, steps)?,
        ty: view.ty(raw.ty, steps)?,
        kind,
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
            (
                ProjectionKind::RawDereference { mutable: a },
                ProjectionKind::RawDereference { mutable: b },
            )
            | (
                ProjectionKind::CaptureBoundary { mutable: a },
                ProjectionKind::CaptureBoundary { mutable: b },
            ) => a == b,
            (ProjectionKind::Field(a), ProjectionKind::Field(b))
            | (ProjectionKind::UnionField(a), ProjectionKind::UnionField(b)) => a == b,
            (
                ProjectionKind::Index {
                    index: a,
                    length: al,
                },
                ProjectionKind::Index {
                    index: b,
                    length: bl,
                },
            ) => a == b && al == bl,
            (
                ProjectionKind::IndexParameter {
                    index: a,
                    length: al,
                },
                ProjectionKind::IndexParameter {
                    index: b,
                    length: bl,
                },
            ) => a == b && al == bl,
            (ProjectionKind::SliceIndex { index: a }, ProjectionKind::SliceIndex { index: b }) => {
                a == b
            }
            (ProjectionKind::ConstantIndex(a), ProjectionKind::ConstantIndex(b)) => a == b,
            (
                ProjectionKind::FromEnd {
                    offset: ao,
                    minimum: am,
                },
                ProjectionKind::FromEnd {
                    offset: bo,
                    minimum: bm,
                },
            ) => ao == bo && am == bm,
            (
                ProjectionKind::Subslice {
                    prefix: ap,
                    suffix: as_,
                },
                ProjectionKind::Subslice {
                    prefix: bp,
                    suffix: bs,
                },
            ) => ap == bp && as_ == bs,
            (
                ProjectionKind::Subarray {
                    start: as_,
                    end: ae,
                },
                ProjectionKind::Subarray { start: bs, end: be },
            ) => as_ == bs && ae == be,
            _ => false,
        };
        if !same || actual.base != expected.base || actual.ty != expected.ty {
            return Err("source original changed an actual dereference place/type/region");
        }
    }
    Ok(())
}

type StaticTypes =
    std::sync::Arc<crate::support::hash::HashMap<crate::body::ir::DefId, crate::types::TypeId>>;

pub(super) fn static_types(
    view: &SourceOriginal<'_>,
    steps: &mut usize,
) -> Result<Option<StaticTypes>, &'static str> {
    if view.rows().statics().is_none() {
        return Ok(None);
    }
    let mut types = crate::support::hash::HashMap::default();
    for declaration in view.raw().static_places() {
        let (definition, ty) = view.static_storage(declaration, steps)?;
        if types
            .insert(definition, ty)
            .is_some_and(|previous| previous != ty)
        {
            return Err("source body changes the type of its static allocation");
        }
    }
    Ok((!types.is_empty()).then(|| std::sync::Arc::new(types)))
}
