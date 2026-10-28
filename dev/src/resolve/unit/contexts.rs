//! Token hygiene follows retained spans independently of physical call ancestry.
use super::{FileId, Unit};
use crate::{expand::marks::SyntaxContextId, resolve::package::Error};

pub(crate) fn callee(
    units: &[Unit],
    file: FileId,
    mut node: crate::syntax::ast::NodeId,
) -> Result<SyntaxContextId, Error> {
    use crate::syntax::ast::Kind;
    let unit = units
        .get(file.0 as usize)
        .ok_or_else(|| Error::at(file, 0, "missing macro callee source"))?;
    loop {
        let value = unit.configured.ast.node(node);
        match value.kind {
            Kind::Macro { path, .. } => node = path,
            Kind::Path { segments, .. } => {
                node =
                    *unit.configured.ast.list(segments).last().ok_or_else(|| {
                        Error::at(file, value.span.start, "empty macro callee path")
                    })?;
            }
            Kind::Applied { base, .. } => node = base,
            Kind::Name(name, _) => return at(units, file, name.start),
            _ => {
                return Err(Error::at(
                    file,
                    value.span.start,
                    "invalid typed macro callee",
                ));
            }
        }
    }
}

pub(crate) fn at(units: &[Unit], file: FileId, offset: u32) -> Result<SyntaxContextId, Error> {
    at_counted(units, file, offset, |_, _, _| Ok(()))
}

/// Query-local owners charge each retained link and binary lookup before work.
pub(crate) fn at_counted(
    units: &[Unit],
    mut file: FileId,
    mut offset: u32,
    mut charge: impl FnMut(usize, usize, usize) -> Result<(), &'static str>,
) -> Result<SyntaxContextId, Error> {
    let mut captured = None;
    let finish = |actual, claimed: Option<SyntaxContextId>, source_file, source_offset| {
        if claimed.is_none_or(|expected| expected == actual) {
            Ok(actual)
        } else {
            Err(Error::at(
                source_file,
                source_offset,
                "formatting capture changed its literal context",
            ))
        }
    };
    for _ in 0..4096 {
        charge(1, 0, 0).map_err(|message| Error::at(file, offset, message))?;
        let unit = units
            .get(file.0 as usize)
            .ok_or_else(|| Error::at(file, offset, "missing token context source"))?;
        if (offset as usize) < unit.source.physical_len() {
            return finish(SyntaxContextId::ROOT, captured, file, offset);
        }
        let origins = &unit.origins;
        charge(
            1 + origins.len().checked_ilog2().unwrap_or(0) as usize,
            0,
            1,
        )
        .map_err(|message| Error::at(file, offset, message))?;
        let index = origins.partition_point(|origin| {
            origin.generated.end <= offset && origin.generated.start != offset
        });
        let Some(origin) = origins
            .get(index)
            .filter(|origin| origin.generated.start <= offset && offset <= origin.generated.end)
        else {
            // Physical EOF is a legitimate location for empty captures.
            if offset as usize == unit.source.physical_len() {
                return finish(SyntaxContextId::ROOT, captured, file, offset);
            }
            return Err(Error::at(file, offset, "missing retained token context"));
        };
        if matches!(origin.kind, crate::expand::stream::OriginKind::Captured(_)) {
            let context = origin.context.ok_or_else(|| {
                Error::at(file, offset, "formatting capture has no literal context")
            })?;
            let anchor = units
                .get(origin.original.file.0 as usize)
                .ok_or_else(|| Error::at(file, offset, "missing formatting capture source"))?;
            if origin.introduction.0 == 0
                || origin.original.span.start >= origin.original.span.end
                || anchor
                    .source
                    .text()
                    .get(origin.original.span.start as usize..origin.original.span.end as usize)
                    .is_none()
                || captured.is_some_and(|expected| expected != context)
            {
                return Err(Error::at(
                    file,
                    offset,
                    "formatting capture changed its source context",
                ));
            }
            captured = Some(context);
            file = origin.original.file;
            offset = origin.original.span.start;
            continue;
        }
        if let Some(context) = origin.context {
            return finish(context, captured, file, offset);
        }
        charge(1, 1, 0).map_err(|message| Error::at(file, offset, message))?;
        let length = origin
            .original
            .span
            .end
            .checked_sub(origin.original.span.start)
            .ok_or_else(|| Error::at(file, offset, "reversed token context origin"))?;
        offset = origin.original.span.start + (offset - origin.generated.start).min(length);
        file = origin.original.file;
    }
    Err(Error::at(
        file,
        offset,
        "token context ancestry exceeds 4096",
    ))
}

#[cfg(test)]
#[path = "../../../tests/invariants/context_origins.rs"]
mod tests;
