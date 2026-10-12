//! Source/environment builtins consume explicit provider inputs after eager expansion.
use super::{Engine, Error, FileId, Location, NodeId, Sources, builtin};
use crate::syntax::ast::MacroContext;
use std::path::Path;

impl<S: Sources> Engine<'_, S> {
    pub(super) fn resource_builtin(
        &mut self,
        file: FileId,
        target: NodeId,
        arguments: NodeId,
        grammar: MacroContext,
        kind: builtin::Builtin,
    ) -> Result<builtin::Expansion, Error> {
        let unit = self.loader.package.unit(file);
        let original = unit.configured.ast.node(target).span;
        if kind == builtin::Builtin::Env {
            let (name, message) =
                builtin::environment_arguments(&unit.configured.ast, arguments, |index| {
                    unit.literal(index)
                })
                .map_err(|e| Error::at(file, e.offset, e.message))?;
            let value = self
                .loader
                .environment(&name)
                .map_err(|e| Error::at(file, original.start, e))?
                .ok_or_else(|| {
                    Error::at(
                        file,
                        original.start,
                        message.unwrap_or_else(|| {
                            format!("environment variable {name:?} is not defined")
                        }),
                    )
                })?;
            return builtin::string(value.to_owned(), grammar, self.config)
                .map_err(|e| Error::at(file, original.start, e.message));
        }
        let (span, name) =
            builtin::string_argument(&unit.configured.ast, arguments, |index| unit.literal(index))
                .map_err(|e| Error::at(file, e.offset, e.message))?;
        let path = Path::new(&name);
        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            let physical = self.call_location(Location {
                file,
                span: original,
            })?;
            let source = &self.loader.package.file_paths[physical.file.0 as usize];
            if source == Path::new("<anon>") {
                return Err(Error::at(
                    file,
                    span.start,
                    "cannot resolve a relative include path from anonymous source",
                ));
            }
            source.parent().unwrap_or(Path::new("")).join(path)
        };
        let bytes = self
            .loader
            .resource(&path)
            .map_err(|e| Error::at(file, span.start, e))?
            .ok_or_else(|| {
                Error::at(
                    file,
                    span.start,
                    format!(
                        "included resource {:?} does not exist",
                        path.to_string_lossy()
                    ),
                )
            })?;
        let generated = if kind == builtin::Builtin::IncludeStr {
            let text = std::str::from_utf8(bytes)
                .map_err(|_| Error::at(file, span.start, "included string is not UTF-8"))?;
            builtin::string(text.to_owned(), grammar, self.config)
        } else {
            builtin::byte_string(bytes.to_vec(), grammar, self.config)
        };
        generated.map_err(|e| Error::at(file, original.start, e.message))
    }
}
