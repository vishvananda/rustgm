//! A crate retains demanded opaque captures once; emitted tokens hold stable dense handles.
use super::{
    macros::{capture::Payload, matcher::Matched, model::Fragment},
    stream::{OpaqueSource, Stream},
};
use crate::{
    lex::token::{ExpansionId, Kind, LexError, Token},
    support::source::FileId,
    syntax::{
        interpolation::{Inputs, Splice},
        token_tree::{Span, error},
    },
};

struct Entry {
    fragment: Fragment,
    payload: Payload,
    file: FileId,
    original: Span,
}
#[derive(Default)]
pub struct Registry {
    entries: Vec<Entry>,
}
pub struct View<'a, F> {
    registry: &'a Registry,
    source: F,
}
impl Registry {
    #[must_use]
    pub fn view<'a, F>(&'a self, source: F) -> View<'a, F>
    where
        F: Fn(FileId) -> Option<&'a str>,
    {
        View {
            registry: self,
            source,
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Move demanded captures into the registry and rewrite only emitted token
    /// handles. Forwarded tokens keep their identity; repeated substitutions
    /// share a single payload. Unused/failed-alternative captures are not retained.
    /// Use a registry view to parse this stream after adoption.
    ///
    /// # Errors
    /// Rejects duplicate adoption, invalid capture references or arena overflow.
    pub fn adopt(
        &mut self,
        stream: &mut Stream,
        matched: &mut Matched,
        file: FileId,
    ) -> Result<(), LexError> {
        if stream.registered {
            return Err(error(0, "expansion stream is already registered"));
        }
        self.entries
            .len()
            .checked_add(stream.opaque.len())
            .filter(|n| *n <= ExpansionId::LIMIT)
            .ok_or_else(|| error(0, "too many persistent macro captures"))?;
        for opaque in &stream.opaque {
            match opaque.payload {
                OpaqueSource::Capture(id) => {
                    let value =
                        matched.captures.values.get(id.0 as usize).ok_or_else(|| {
                            error(opaque.original.start, "invalid demanded capture")
                        })?;
                    value.payload.nonterminal(value.fragment)?;
                }
                OpaqueSource::Forwarded { index, .. } if index as usize >= self.entries.len() => {
                    return Err(error(
                        opaque.original.start,
                        "missing persistent forwarded capture",
                    ));
                }
                OpaqueSource::Forwarded { .. } => {}
            }
        }
        for token in &stream.tokens {
            if let Kind::Interpolated(index) = token.kind
                && index.get() as usize >= stream.opaque.len()
            {
                return Err(error(token.start, "invalid expansion token handle"));
            }
        }
        let mut captures = vec![None; matched.captures.values.len()];
        let mut remap = Vec::with_capacity(stream.opaque.len());
        for opaque in &stream.opaque {
            let id = match opaque.payload {
                OpaqueSource::Forwarded { index, .. } => index,
                OpaqueSource::Capture(id) => {
                    if let Some(index) = captures[id.0 as usize] {
                        index
                    } else {
                        let value = &mut matched.captures.values[id.0 as usize];
                        let index = u32::try_from(self.entries.len())
                            .map_err(|_| error(0, "capture identity overflow"))?;
                        self.entries.push(Entry {
                            fragment: value.fragment,
                            payload: std::mem::replace(&mut value.payload, Payload::Transparent),
                            file,
                            original: opaque.original,
                        });
                        captures[id.0 as usize] = Some(index);
                        index
                    }
                }
            };
            remap.push(id);
        }
        for token in &mut stream.tokens {
            if let Kind::Interpolated(index) = &mut token.kind {
                *index = ExpansionId::new(remap[index.get() as usize])?;
            }
        }
        stream.registered = true;
        Ok(())
    }
}
impl<'a, F> Inputs for View<'a, F>
where
    F: Fn(FileId) -> Option<&'a str>,
{
    fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }
    fn get(&self, token: Token) -> Result<Option<Splice<'_>>, LexError> {
        let Kind::Interpolated(index) = token.kind else {
            return Ok(None);
        };
        let entry = self
            .registry
            .entries
            .get(index.get() as usize)
            .ok_or_else(|| error(token.start, "missing persistent macro capture"))?;
        let original_source = (self.source)(entry.file)
            .ok_or_else(|| error(token.start, "missing persistent capture source"))?;
        Ok(Some(Splice {
            fragment: entry.payload.nonterminal(entry.fragment)?,
            original_source,
            original: entry.original,
            destination: Span {
                start: token.start,
                end: token.end,
            },
        }))
    }
}
