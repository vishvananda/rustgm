//! One cold transaction owns each demanded list until all actual members resolve.
mod paired_impls;
mod receipt;
#[cfg(test)]
#[path = "../../../tests/invariants/derived_locations.rs"]
mod tests;
use super::{
    Body, Builtin, Call, CallGrammar, Engine, Error, ExpansionId, Invocation, NodeKey, Sources,
    State,
};
use crate::{
    expand::configured::{ConfiguredAttribute, Effect},
    lex::token::Punctuation,
    resolve::bindings::panic_contracts,
    support::source::OriginalImage,
    syntax::{
        attribute::Input,
        token_tree::{Tree, TreeRange},
    },
};
use std::{ops::Range, sync::Arc};

#[derive(Default)]
pub(super) struct Transactions {
    entries: Vec<Transaction>,
}
struct Transaction {
    outer: usize,
    owner: NodeKey,
    effect: usize,
    capture: u32,
    attribute: ConfiguredAttribute,
    trees: Vec<Tree>,
    spelling: Box<str>,
    image: Arc<OriginalImage>,
    children: Range<usize>,
    pending: usize,
    invocation: usize,
    contexts: Range<usize>,
    output: Option<super::NodeId>,
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn derive_outer(&self, index: usize) -> bool {
        self.derive_position(index).is_some()
    }
    fn derive_position(&self, index: usize) -> Option<usize> {
        self.attribute_effects
            .as_deref()?
            .derives
            .as_deref()?
            .entries
            .binary_search_by_key(&index, |entry| entry.outer)
            .ok()
    }
    fn derive_entry(&self, index: usize) -> Result<&Transaction, Error> {
        let position = self
            .derive_position(index)
            .ok_or_else(|| Error::unlocated("missing actual derive transaction"))?;
        Ok(&self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or_else(|| Error::unlocated("missing derive transaction owner"))?
            .entries[position])
    }
    pub(super) fn derive_consumed(&self, index: usize) -> Result<bool, Error> {
        let outer = if matches!(self.calls[index].grammar, CallGrammar::Derive) {
            self.calls[index]
                .visit
                .eager
                .ok_or_else(|| self.calls[index].diagnostic("derive member lost its actual list"))?
        } else {
            index
        };
        let Some(position) = self.derive_position(outer) else {
            return Ok(false);
        };
        let entry = &self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or_else(|| Error::unlocated("missing derive owner"))?
            .entries[position];
        receipt::current(&self.loader.package, entry, entry.output.is_some())?;
        Ok(entry.output.is_some())
    }
    pub(super) fn derive_completed(&mut self, outer: usize) -> Result<(), Error> {
        let position = self
            .derive_position(outer)
            .ok_or_else(|| Error::unlocated("derive completion has no actual list"))?;
        let entry = &mut self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.derives.as_deref_mut())
            .ok_or_else(|| Error::unlocated("missing derive completion owner"))?
            .entries[position];
        entry.pending = entry
            .pending
            .checked_sub(1)
            .ok_or_else(|| Error::unlocated("derive member counter underflow"))?;
        if entry.pending == 0 {
            self.wake(outer);
        }
        Ok(())
    }
    pub(super) fn begin_derive(&mut self, index: usize, definition: usize) -> Result<(), Error> {
        self.checked_derive_definition(definition, Builtin::Derive)?;
        if self.calls[index].visit.in_block {
            return Err(
                self.calls[index].diagnostic("block-owned source derives require a wider profile")
            );
        }
        let ranges = self.derive_ranges(index)?;
        let mut transaction = self.prepare_derive_transaction(index, ranges.len())?;
        self.calls
            .try_reserve(ranges.len())
            .map_err(|_| self.calls[index].diagnostic("derive member allocation failed"))?;
        self.ready
            .try_reserve(ranges.len())
            .map_err(|_| self.calls[index].diagnostic("derive queue allocation failed"))?;
        let parent = self.calls[index].visit.introduction;
        let context = self.derive_invocation(index, definition, parent)?;
        let first = self.calls.len();
        for range in ranges {
            self.queue_derive_member(index, context, range)?;
        }
        transaction.children = first..self.calls.len();
        self.attribute_effects
            .get_or_insert_default()
            .derives
            .get_or_insert_default()
            .entries
            .push(transaction);
        self.calls[index].selected = Some(definition);
        self.calls[index].state = State::InProgress;
        if first == self.calls.len() {
            self.wake(index);
        }
        Ok(())
    }
    fn prepare_derive_transaction(&self, index: usize, count: usize) -> Result<Transaction, Error> {
        let call = &self.calls[index];
        let (file, owner, effect) = (call.file, call.visit.node, call.capture as usize);
        if count > 1_000_000_usize.saturating_sub(self.calls.len()) {
            return Err(call.diagnostic("derive member count exceeds the macro work budget"));
        }
        let unit = self.loader.package.unit(file);
        let actual = &unit.configured.effects[effect];
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| call.diagnostic("derive requires its actual local provider"))?;
        source
            .macros
            .charge(
                count,
                size_of::<Transactions>()
                    + size_of::<super::attribute_effects::Effects>()
                    + 4 * size_of::<Transaction>()
                    + (actual
                        .attribute
                        .attribute
                        .span
                        .end
                        .saturating_sub(actual.attribute.attribute.span.start)
                        as usize)
                        .saturating_mul(4)
                    + unit.configured.ast.attributes[actual.capture as usize]
                        .forest
                        .payload_bytes()
                        .saturating_mul(4)
                    + ranges
                        .len()
                        .saturating_mul(4 * (size_of::<Call>() + size_of::<usize>())),
            )
            .map_err(|message| call.diagnostic(message))?;
        Ok(Transaction {
            outer: index,
            owner: NodeKey { file, node: owner },
            effect,
            capture: actual.capture,
            attribute: actual.attribute,
            spelling: unit
                .source
                .text()
                .get(
                    actual.attribute.attribute.span.start as usize
                        ..actual.attribute.attribute.span.end as usize,
                )
                .ok_or_else(|| call.diagnostic("derive capture is outside its actual source"))?
                .into(),
            trees: unit.configured.ast.attributes[actual.capture as usize]
                .forest
                .nodes()
                .to_vec(),
            image: Arc::clone(unit.source.original_image()),
            children: self.calls.len()..self.calls.len(),
            pending: count,
            invocation: self.invocations.len(),
            contexts: 0..0,
            output: None,
        })
    }
    fn queue_derive_member(
        &mut self,
        index: usize,
        context: ExpansionId,
        range: TreeRange,
    ) -> Result<(), Error> {
        let (file, effect, offset) = (
            self.calls[index].file,
            self.calls[index].capture as usize,
            self.calls[index].offset,
        );
        let (absolute, path) =
            super::decorated_calls::path_range(&mut self.loader.package, file, effect, range)?;
        let call = &self.calls[index];
        let child = self.calls.len();
        self.calls.push(Call {
            visit: super::Visit {
                eager: Some(index),
                introduction: context,
                ..call.visit
            },
            file,
            capture: call.capture,
            grammar: CallGrammar::Derive,
            absolute,
            path: path.save(),
            scope: call.scope,
            output: None,
            state: State::Unresolved,
            queued: true,
            selected: None,
            error: None,
            offset,
            eager: None,
            notified: false,
        });
        // Child membership is the contiguous range just prepared from this exact capture.
        self.lint_derive_call(child, index)?;
        self.ready.push_back(child);
        Ok(())
    }
    fn derive_ranges(&self, index: usize) -> Result<Vec<TreeRange>, Error> {
        let call = &self.calls[index];
        let unit = self.loader.package.unit(call.file);
        let effect = &unit.configured.effects[call.capture as usize];
        let attribute = effect.attribute.attribute;
        let Input::Delimited(group) = attribute.input else {
            return Err(call.diagnostic("derive requires a parenthesized path list"));
        };
        if attribute.unsafe_wrapper {
            return Err(call.diagnostic("derive cannot use an unsafe attribute wrapper"));
        }
        let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
        let range = forest
            .children(group)
            .ok_or_else(|| call.diagnostic("derive lost its actual argument list"))?;
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| call.diagnostic("missing derive provider"))?;
        source
            .macros
            .charge(
                (range.end - range.start) as usize,
                (range.end - range.start) as usize * 4 * size_of::<TreeRange>(),
            )
            .map_err(|message| call.diagnostic(message))?;
        let mut result = Vec::new();
        let mut start = range.start;
        for cursor in range.start..range.end {
            if forest.punctuation(cursor, Punctuation::Comma) {
                if cursor == start {
                    return Err(call.diagnostic("derive list requires an actual macro path"));
                }
                result.push(TreeRange { start, end: cursor });
                start = cursor + 1;
            }
        }
        if start != range.end {
            result.push(TreeRange {
                start,
                end: range.end,
            });
        }
        Ok(result)
    }
    fn checked_derive_definition(&self, definition: usize, expected: Builtin) -> Result<(), Error> {
        let entry = self
            .definitions
            .get(definition)
            .ok_or_else(|| Error::unlocated("unknown selected derive definition"))?;
        if !matches!(entry.body, Body::Builtin(role) if role == expected) {
            return Err(Error::unlocated(
                "derive changed its actual selected source role",
            ));
        }
        let key = self
            .paths
            .checked_source_macro(definition, entry)
            .map_err(Error::unlocated)?;
        let header =
            panic_contracts::published(&self.loader.package, key, entry.header.as_ref(), 1)?;
        if header.builtin != Some(expected) || header.edition != entry.edition {
            return Err(Error::unlocated(
                "derive changed its defining source contract",
            ));
        }
        Ok(())
    }
    fn derive_invocation(
        &mut self,
        index: usize,
        definition: usize,
        parent: ExpansionId,
    ) -> Result<ExpansionId, Error> {
        let call = &self.calls[index];
        let context = ExpansionId(
            u32::try_from(self.invocations.len() + 1)
                .map_err(|_| call.diagnostic("derive invocation overflow"))?,
        );
        self.loader
            .package
            .source_core()
            .ok_or_else(|| call.diagnostic("missing derive provider"))?
            .macros
            .charge(1, 4 * size_of::<Invocation>())
            .map_err(|message| call.diagnostic(message))?;
        self.paths
            .invocation(context, parent, &self.definitions[definition])
            .map_err(|message| call.diagnostic(message))?;
        let original = self.loader.package.unit(call.file).configured.effects
            [call.capture as usize]
            .attribute
            .attribute
            .span;
        self.invocations.push(Invocation {
            context,
            node: NodeKey {
                file: call.file,
                node: call.visit.node,
            },
            original,
            definition,
            rule: None,
            generated: original,
            parser_calls: 0,
            parent,
        });
        Ok(context)
    }
}
