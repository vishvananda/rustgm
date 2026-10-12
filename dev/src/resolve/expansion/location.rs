//! Physical builtin locations follow complete captured spans and template calls.
use super::{Engine, Error, Location, Sources};
impl<S: Sources> Engine<'_, S> {
    pub(super) fn module_path(&self, mut module: super::ModuleId) -> String {
        let package = &self.loader.package;
        let mut parts = Vec::new();
        while let Some(parent) = package.modules[module.0 as usize].parent {
            parts.push(package.modules[module.0 as usize].name);
            module = parent;
        }
        let mut path = package.crate_name.clone();
        for name in parts.into_iter().rev() {
            path.push_str("::");
            path.push_str(package.names.get(name));
        }
        path
    }

    pub(super) fn call_location(&self, mut location: Location) -> Result<Location, Error> {
        for _ in 0..4096 {
            let Some(first) = self
                .loader
                .package
                .origin(location.file, location.span.start)
            else {
                return Ok(location);
            };
            let context = if location.span.end <= first.generated.end {
                first.context
            } else {
                let origins = &self.loader.package.unit(location.file).origins;
                let start =
                    origins.partition_point(|origin| origin.generated.end <= location.span.start);
                origins[start..]
                    .iter()
                    .take_while(|origin| origin.generated.start < location.span.end)
                    .find_map(|origin| origin.context)
            };
            if let Some(context) = context.filter(|context| context.0 != 0) {
                let call = self
                    .invocations
                    .get(context.0 as usize - 1)
                    .ok_or_else(|| {
                        Error::at(
                            location.file,
                            location.span.start,
                            "invalid builtin expansion context",
                        )
                    })?;
                location = Location {
                    file: call.node.file,
                    span: call.original,
                };
                continue;
            }
            let delta = location.span.start - first.generated.start;
            let extent = first.original.span.end - first.original.span.start;
            let end_delta = location.span.end - first.generated.start;
            location = Location {
                file: first.original.file,
                span: super::Span {
                    start: first.original.span.start + delta.min(extent),
                    end: first.original.span.start + end_delta.min(extent),
                },
            };
        }
        Err(Error::at(
            location.file,
            location.span.start,
            "builtin source provenance exceeds 4096",
        ))
    }
}
