//! Captured identifiers inherit the literal's context, independently of constructors.
use super::{Engine, Error, ExpansionId, FileId, NodeId, Sources, builtin};
use crate::{
    expand::stream::{Location, Origin, OriginKind},
    support::retained::Growth,
    syntax::{ast::Kind, token_tree::Span},
};
impl<S: Sources> Engine<'_, S> {
    pub(super) fn format_capture_origins(
        &mut self,
        file: FileId,
        literal: NodeId,
        draft: &builtin::format_arguments::Draft,
        introduction: ExpansionId,
        default: Origin,
    ) -> Result<Option<Vec<Origin>>, Error> {
        if draft.captured.is_empty() {
            return Ok(None);
        }
        let package = &mut self.loader.package;
        let unit = package.unit(file);
        let anchor = unit.configured.ast.node(literal).span;
        if !matches!(unit.configured.ast.node(literal).kind, Kind::Literal(_)) {
            return Err(Error::at(
                file,
                anchor.start,
                "formatting capture lost its literal root",
            ));
        }
        let source = package
            .source_core()
            .ok_or_else(|| Error::at(file, anchor.start, "missing formatting capture provider"))?;
        let count = draft.captured.len().saturating_mul(2).saturating_add(1);
        let growth = Growth::vector::<Origin>(unit.origins.len(), unit.origins.capacity(), count);
        source
            .macros
            .charge(
                count.saturating_add(growth.work),
                count
                    .saturating_mul(size_of::<Origin>())
                    .saturating_add(growth.bytes),
            )
            .map_err(|message| Error::at(file, anchor.start, message))?;
        let context = crate::resolve::unit::contexts::at_counted(
            &package.units,
            file,
            anchor.start,
            |steps, _, _| source.macros.charge(steps, 0),
        )?;
        let edition = crate::resolve::unit::editions::at(Location { file, span: anchor }, |id| {
            package.units.get(id.0 as usize)
        })
        .map_err(|e| Error::at(file, e.offset, e.message))?;
        validate_names(package, file, anchor, draft)?;
        let mut origins = Vec::with_capacity(count);
        let mut start = 0;
        for &node in &draft.captured {
            let generated = draft.expansion.syntax.ast.node(node).span;
            if start < generated.start {
                origins.push(Origin {
                    generated: Span {
                        start,
                        end: generated.start,
                    },
                    ..default
                });
            }
            origins.push(Origin {
                generated,
                original: Location { file, span: anchor },
                context: Some(context),
                introduction,
                joint: None,
                kind: OriginKind::Captured(edition),
            });
            start = generated.end;
        }
        if start < default.generated.end {
            origins.push(Origin {
                generated: Span {
                    start,
                    end: default.generated.end,
                },
                ..default
            });
        }
        Ok(Some(origins))
    }
}

fn validate_names(
    package: &super::Package,
    file: FileId,
    anchor: Span,
    draft: &builtin::format_arguments::Draft,
) -> Result<(), Error> {
    let source = package
        .source_core()
        .ok_or_else(|| Error::at(file, anchor.start, "missing formatting capture provider"))?;
    let mut end = 0;
    // Validate every interval and canonical symbol before allocation or install.
    for &node in &draft.captured {
        let actual = draft
            .expansion
            .syntax
            .ast
            .nodes
            .get(node.get() as usize)
            .ok_or_else(|| Error::at(file, anchor.start, "missing formatting capture node"))?;
        let Kind::Name(span, Some(name)) = actual.kind else {
            return Err(Error::at(
                file,
                anchor.start,
                "formatting capture changed its ordinary name",
            ));
        };
        let spelling = draft
            .expansion
            .source
            .get(span.start as usize..span.end as usize)
            .filter(|_| span == actual.span && span.start >= end && span.start < span.end)
            .ok_or_else(|| Error::at(file, anchor.start, "formatting capture interval changed"))?;
        source
            .macros
            .charge(spelling.len().saturating_add(1), 0)
            .map_err(|message| Error::at(file, anchor.start, message))?;
        if name.0 as usize >= package.names.len() || package.names.get(name) != spelling {
            return Err(Error::at(
                file,
                anchor.start,
                "formatting capture symbol changed",
            ));
        }
        end = span.end;
    }
    Ok(())
}
