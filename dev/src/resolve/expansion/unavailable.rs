//! Declarations use item scope; an unavailable body never enters transcription.
use super::{Body, Context, Definition, Engine, Error, FileId, ItemKind, NodeKey, Sources, Visit};
use crate::syntax::item::Item;

impl<S: Sources> Engine<'_, S> {
    pub(super) fn define_scoped(
        &mut self,
        visit: Visit,
        file: FileId,
        item: Item,
    ) -> Result<(), Error> {
        if !matches!(item.kind, ItemKind::DeclarativeMacro { .. }) {
            return Err(Error::at(
                file,
                0,
                "scoped publication changed its declaration kind",
            ));
        }
        let offset = self
            .loader
            .package
            .unit(file)
            .configured
            .ast
            .node(visit.node)
            .span
            .start;
        let context = self.context(file, offset)?;
        let index = self.definitions.len();
        let (declared, name) = self.paths.scoped_item(
            &mut self.loader.package,
            Context {
                module: visit.module,
                file,
                scope: visit.path_scope,
                foreign: visit.foreign,
            },
            visit.node,
            visit.owner,
            item,
            (index, visit.introduction),
        )?;
        let edition_panic = declared.header.as_ref().is_some_and(|header| {
            header
                .declaration()
                .allows(crate::resolve::bindings::KnownFeature::EditionPanic)
        });
        let body = if let Some(role) = declared
            .header
            .as_ref()
            .and_then(|header| header.declaration().builtin)
        {
            // The shared producer already authenticated the sole scoped builtin role.
            Body::Builtin(role)
        } else {
            Body::ScopedDeclaration(declared.mode)
        };
        self.definitions.push(Definition {
            node: Some(NodeKey {
                file,
                node: visit.node,
            }),
            module: visit.module,
            name,
            body,
            context,
            introduction: visit.introduction,
            preceding_scope: self.head,
            local_inner_macros: false,
            edition: declared.edition,
            edition_panic,
            header: declared.header,
        });
        // Deliberately leave the legacy textual scope and export path untouched.
        Ok(())
    }

    pub(super) fn require_invocable(
        &self,
        definition: usize,
        file: FileId,
        offset: u32,
    ) -> Result<(), Error> {
        let actual = self
            .definitions
            .get(definition)
            .ok_or_else(|| Error::at(file, offset, "unknown selected macro definition"))?;
        if matches!(actual.body, Body::Builtin(role) if role.declaration_only()) {
            #[cfg(test)]
            crate::resolve::index::unavailable_macros::observation::note(
                crate::resolve::index::unavailable_macros::observation::Event::Denied(
                    self.scoped_snapshot(definition),
                ),
            );
            return Err(Error::at(
                file,
                offset,
                "formatting builtin invocation is not implemented",
            ));
        }
        if matches!(actual.body, Body::ScopedDeclaration(_)) {
            #[cfg(test)]
            crate::resolve::index::unavailable_macros::observation::note(
                crate::resolve::index::unavailable_macros::observation::Event::Denied(
                    self.scoped_snapshot(definition),
                ),
            );
            return Err(Error::at(
                file,
                offset,
                "scoped macro invocation is not implemented",
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn scoped_snapshot(
        &self,
        definition: usize,
    ) -> crate::resolve::index::unavailable_macros::observation::Snapshot {
        crate::resolve::index::unavailable_macros::observation::Snapshot {
            target: definition,
            owner: self.definitions[definition].node,
            captures: self.captures.len(),
            invocations: self.invocations.len(),
            marks: self.loader.package.marks.context_count(),
        }
    }
}
