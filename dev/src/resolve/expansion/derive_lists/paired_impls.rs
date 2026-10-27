//! Complete a whole list before consuming its configured effect or visiting impls.
use super::{Body, Builtin, Effect, Engine, Error, ExpansionId, Range, Sources, receipt};
use crate::{
    expand::{
        builtin::derive_output,
        marks::{SyntaxContextId, Transparency},
    },
    syntax::{ast::Kind, token_tree::error},
};

const MEMBERS: [Builtin; 9] = derive_output::MEMBERS;
type Selected = [Option<(usize, usize)>; MEMBERS.len()];

impl<S: Sources> Engine<'_, S> {
    pub(in crate::resolve::expansion) fn finish_derive(
        &mut self,
        index: usize,
    ) -> Result<(), Error> {
        let entry = self.derive_entry(index)?;
        if entry.output.is_some() {
            return self.completed(index);
        }
        if entry.pending != 0 {
            return Ok(());
        }
        receipt::current(&self.loader.package, entry, false)?;
        let children = entry.children.clone();
        let parent = self.invocations[entry.invocation].context;
        let (file, effect) = (entry.owner.file, entry.effect);
        let selected = self.derive_selection(children)?;
        let start = self.invocations.len();
        let contexts = self.derive_contexts(index, selected, parent)?;
        let generated = self.derive_generated(index, contexts, selected)?;
        let (span, root) = self.loader.package.units[file.0 as usize]
            .append_derived(
                &generated.stream.source,
                generated.syntax,
                &generated.stream.origins,
            )
            .map_err(|failure| Error::at(file, failure.offset, failure.message))?;
        for invocation in &mut self.invocations[start..] {
            invocation.generated = span;
        }
        let position = self
            .derive_position(index)
            .ok_or_else(|| self.calls[index].diagnostic("derive transaction disappeared"))?;
        let entry = &mut self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.derives.as_deref_mut())
            .ok_or_else(|| Error::unlocated("missing derive output owner"))?
            .entries[position];
        entry.contexts = start..self.invocations.len();
        entry.output = Some(root);
        self.loader.package.units[file.0 as usize]
            .configured
            .effects[effect]
            .attribute
            .effect = Effect::ExpandedDerive;
        self.invocations[entry.invocation].generated = span;
        self.invocations[entry.invocation].parser_calls = 1;
        self.resume_derived_owner(index, root, parent)
    }
    fn derive_selection(&self, children: Range<usize>) -> Result<Selected, Error> {
        let mut selected = [None; MEMBERS.len()];
        for child in children {
            let definition = self.calls[child].selected.ok_or_else(|| {
                self.calls[child].diagnostic("derive member has no actual selected definition")
            })?;
            let Body::Builtin(role) = self.definitions[definition].body else {
                return Err(self.calls[child]
                    .diagnostic("this source derive has no executable member consumer"));
            };
            let slot = MEMBERS
                .iter()
                .position(|&member| member == role)
                .ok_or_else(|| {
                    self.calls[child]
                        .diagnostic("this source derive has no executable member consumer")
                })?;
            self.checked_derive_definition(definition, role)?;
            if selected[slot].replace((child, definition)).is_some() {
                return Err(
                    self.calls[child].diagnostic("conflicting derived trait implementations")
                );
            }
        }
        Ok(selected)
    }
    fn derive_contexts(
        &mut self,
        index: usize,
        selected: Selected,
        parent: ExpansionId,
    ) -> Result<[(SyntaxContextId, ExpansionId); MEMBERS.len()], Error> {
        let file = self.calls[index].file;
        let mut contexts = [(SyntaxContextId::ROOT, ExpansionId(0)); MEMBERS.len()];
        for (slot, selected) in selected.iter().enumerate() {
            if let Some(&(child, definition)) = selected.as_ref() {
                let context = self.derive_invocation(child, definition, parent)?;
                let call_site = self
                    .loader
                    .package
                    .syntax_context(file, self.calls[index].offset)?;
                let raw = self
                    .loader
                    .package
                    .marks
                    .apply(
                        self.definitions[definition].context,
                        context,
                        Transparency::Opaque,
                        call_site,
                    )
                    .map_err(|message| self.calls[index].diagnostic(message))?;
                contexts[slot] = (raw, context);
            }
        }
        Ok(contexts)
    }
    fn derive_generated(
        &self,
        index: usize,
        contexts: [(SyntaxContextId, ExpansionId); MEMBERS.len()],
        selected: Selected,
    ) -> Result<crate::expand::builtin::delegation::Delegated, Error> {
        let entry = self.derive_entry(index)?;
        let (file, owner, effect) = (entry.owner.file, entry.owner.node, entry.effect);
        let unit = self.loader.package.unit(file);
        let mut node = owner;
        while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
            node = value;
        }
        let Kind::Item(item) = unit.configured.ast.node(node).kind else {
            return Err(self.calls[index].diagnostic("source derive requires an actual item owner"));
        };
        let source = self
            .loader
            .package
            .source_core()
            .ok_or_else(|| self.calls[index].diagnostic("missing derive provider"))?;
        let original = unit.configured.effects[effect].attribute.attribute.span;
        let charge = |steps, bytes| {
            source
                .macros
                .charge(steps, bytes)
                .map_err(|message| error(original.start, message))
        };
        let generated = derive_output::build(
            &derive_output::Site {
                file,
                source: unit.source.text(),
                ast: &unit.configured.ast,
                active: &unit.configured.active,
                item: unit.configured.ast.items[item as usize],
                original,
                members: std::array::from_fn(|slot| {
                    selected[slot].map(|(_, definition)| derive_output::MemberContext {
                        context: contexts[slot],
                        edition: self.definitions[definition].edition,
                    })
                }),
                const_: matches!(
                    self.definitions[self.calls[index]
                        .selected
                        .ok_or_else(|| self.calls[index]
                            .diagnostic("derive list lost its selected definition"))?]
                    .body,
                    Body::Builtin(Builtin::DeriveConst)
                ),
                prior_copy: self.completed_copy(index)?,
            },
            self.config,
            &charge,
        )
        .map_err(|failure| Error::at(file, failure.offset, failure.message))?;
        source
            .macros
            .charge(1, generated.payload_bytes().saturating_mul(4))
            .map_err(|message| self.calls[index].diagnostic(message))?;
        Ok(generated)
    }
    fn resume_derived_owner(
        &mut self,
        index: usize,
        root: super::super::NodeId,
        parent: ExpansionId,
    ) -> Result<(), Error> {
        let call = &self.calls[index];
        let (visit, scope) = (call.visit, call.scope);
        let tail = self.lint_call_tail(index)?;
        self.work.push(super::super::Work::Restore(self.head));
        self.head = scope;
        self.push_visit_with_tail(
            super::super::Visit {
                node: root,
                owner: None,
                introduction: parent,
                depth: visit.depth + 1,
                ..visit
            },
            tail,
        )?;
        self.scan()?;
        self.work.push(super::super::Work::Restore(self.head));
        self.head = scope;
        self.push_visit_with_tail(visit, tail)?;
        self.scan()?;
        self.completed(index)
    }
    pub(in crate::resolve::expansion) fn attach_derives(&mut self) -> Result<(), Error> {
        let Some(transactions) = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
        else {
            return Ok(());
        };
        let Some(crate::resolve::bindings::Source::Core(source)) =
            self.loader.package.source.as_deref()
        else {
            return Err(Error::unlocated("derive installation lost its provider"));
        };
        source
            .macros
            .charge(
                transactions.entries.len(),
                transactions
                    .entries
                    .len()
                    .saturating_mul(8 * size_of::<(super::super::NodeId, super::super::NodeId)>()),
            )
            .map_err(Error::unlocated)?;
        let mut outputs: Vec<_> = transactions
            .entries
            .iter()
            .map(|entry| {
                entry
                    .output
                    .map(|root| (entry.owner.file, entry.owner.node, root))
                    .ok_or_else(|| Error::unlocated("derive transaction is incomplete"))
            })
            .collect::<Result<_, _>>()?;
        source
            .macros
            .charge(
                outputs
                    .len()
                    .saturating_mul(outputs.len().checked_ilog2().unwrap_or(0) as usize + 1),
                0,
            )
            .map_err(Error::unlocated)?;
        outputs.sort_unstable_by_key(|(file, owner, root)| (file.0, owner.get(), root.get()));
        let mut start = 0;
        while start < outputs.len() {
            let file = outputs[start].0;
            let end = start + outputs[start..].partition_point(|(next, _, _)| *next == file);
            let mut items: Vec<_> = outputs[start..end]
                .iter()
                .map(|(_, owner, root)| (*owner, *root))
                .collect();
            let charge = |steps, bytes| {
                source
                    .macros
                    .charge(steps, bytes)
                    .map_err(|message| error(0, message))
            };
            self.loader.package.units[file.0 as usize]
                .attach_derived(&mut items, &charge)
                .map_err(|failure| Error::at(file, failure.offset, failure.message))?;
            start = end;
        }
        Ok(())
    }
}
