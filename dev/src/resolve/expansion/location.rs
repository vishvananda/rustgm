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

struct TokenSources<'a> {
    package: &'a super::Package,
    captures: &'a crate::expand::registry::Registry,
}
impl crate::expand::stringify::Sources for TokenSources<'_> {
    fn source(&self, file: super::FileId) -> &str {
        self.package.unit(file).source.text()
    }
    fn capture(
        &self,
        id: crate::lex::token::ExpansionId,
    ) -> Result<(super::FileId, &[crate::lex::token::Token]), crate::lex::token::LexError> {
        self.captures.tokens(id)
    }
    fn joint(
        &self,
        mut file: super::FileId,
        token: crate::lex::token::Token,
    ) -> Result<bool, crate::lex::token::LexError> {
        let mut span = super::Span {
            start: token.start,
            end: token.end,
        };
        for _ in 0..4096 {
            let Some(origin) = self.package.origin(file, span.start) else {
                let tail = self.source(file).get(span.end as usize..).ok_or_else(|| {
                    crate::syntax::token_tree::error(span.end, "invalid token spacing source")
                })?;
                return Ok(!tail.starts_with("//")
                    && !tail.starts_with("/*")
                    && tail
                        .chars()
                        .next()
                        .is_some_and(|ch| !crate::lex::is_whitespace(ch)));
            };
            if let Some(joint) = origin.joint {
                return Ok(joint);
            }
            let delimiter = matches!(
                token.kind,
                crate::lex::token::Kind::Punctuation(
                    crate::lex::token::Punctuation::OpenParen
                        | crate::lex::token::Punctuation::CloseParen
                        | crate::lex::token::Punctuation::OpenBrace
                        | crate::lex::token::Punctuation::CloseBrace
                        | crate::lex::token::Punctuation::OpenBracket
                        | crate::lex::token::Punctuation::CloseBracket
                )
            );
            if (origin.context.is_some() && !delimiter)
                || origin.generated.end - origin.generated.start
                    != origin.original.span.end - origin.original.span.start
            {
                return Ok(false);
            }
            span = super::Span {
                start: origin.original.span.start + span.start - origin.generated.start,
                end: origin.original.span.start + span.end - origin.generated.start,
            };
            file = origin.original.file;
        }
        Err(crate::syntax::token_tree::error(
            token.start,
            "token spacing provenance exceeds 4096",
        ))
    }
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn stringify(
        &self,
        file: super::FileId,
        input: &super::Input,
    ) -> Result<String, Error> {
        crate::expand::stringify::render(
            file,
            &input.tokens,
            &TokenSources {
                package: &self.loader.package,
                captures: &self.captures,
            },
        )
        .map_err(|e| Error::at(file, e.offset, e.message))
    }
}
