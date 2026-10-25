//! Item-scoped macros retain normal matching and their actual definition site.
use super::{
    Body, Compiled, Context, Definition, Engine, Error, FileId, ItemKind, NodeKey, Sources, Visit,
};
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
            // Only actual scoped declarations own this cold allocation. Keep
            // every ordinary macro body and definition at their existing size.
            if let Some(source) = self.loader.package.source_core() {
                source
                    .macros
                    .charge(1, size_of::<Compiled>())
                    .map_err(|message| Error::at(file, offset, message))?;
            }
            Body::Scoped {
                compiled: Box::new(Compiled::new(declared.matcher)),
                mode: declared.mode,
            }
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

    /// A selected call is denied before any capture, mark or invocation work.
    pub(super) fn require_call_definition(
        &self,
        call: &super::Call,
        definition: usize,
    ) -> Result<(), Error> {
        #[cfg(test)]
        if matches!(self.definitions[definition].body, Body::Scoped { .. })
            || matches!(self.definitions[definition].body, Body::Builtin(role) if !role.executable())
        {
            crate::resolve::index::unavailable_macros::observation::note(
                crate::resolve::index::unavailable_macros::observation::Event::Selected(
                    self.scoped_snapshot(definition),
                ),
            );
        }
        self.require_invocable(definition, call.grammar.kind(), call.file, call.offset)?;
        Ok(())
    }

    pub(super) fn require_invocable(
        &self,
        definition: usize,
        kind: crate::expand::builtin::BuiltinKind,
        file: FileId,
        offset: u32,
    ) -> Result<(), Error> {
        let actual = self
            .definitions
            .get(definition)
            .ok_or_else(|| Error::at(file, offset, "unknown selected macro definition"))?;
        let declared_role = actual
            .header
            .as_ref()
            .and_then(|header| header.declaration().builtin);
        let actual_kind = declared_role.map_or_else(
            || match actual.body {
                Body::Builtin(role) => role.kind(),
                _ => crate::expand::builtin::BuiltinKind::Bang,
            },
            crate::expand::builtin::Builtin::kind,
        );
        if kind != actual_kind {
            return Err(self.unavailable_error(
                definition,
                file,
                offset,
                "macro invocation has a different actual macro kind",
            ));
        }
        if declared_role.is_some_and(|role| !role.executable())
            || matches!(actual.body, Body::Builtin(role) if !role.executable())
        {
            return Err(self.unavailable_error(
                definition,
                file,
                offset,
                "builtin attribute execution is not implemented",
            ));
        }
        Ok(())
    }

    fn unavailable_error(
        &self,
        definition: usize,
        file: FileId,
        offset: u32,
        message: &'static str,
    ) -> Error {
        #[cfg(test)]
        crate::resolve::index::unavailable_macros::observation::note(
            crate::resolve::index::unavailable_macros::observation::Event::Denied(
                self.scoped_snapshot(definition),
            ),
        );
        // The ordinary diagnostic frame names the actual selected declaration.
        let mut error = Error::at(file, offset, message);
        let span = error.span;
        error.expansions.push(super::diagnostics::frame(
            &self.loader.package,
            &self.definitions[definition],
            super::Location { file, span },
        ));
        error
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
