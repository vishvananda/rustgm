//! `#[global_allocator]` and `#[alloc_error_handler]` run once on their actual
//! configured item: the item stays, and rustc's generated constant is placed
//! after it, as a derive's impls are.
use super::{Body, Builtin, Engine, Error, NodeKey, Sources, State};
use crate::{
    expand::{
        builtin::allocator_output::{self, Role},
        configured::Effect,
        marks::Transparency,
    },
    syntax::{
        ast::{Kind, NodeId},
        attribute::{Input, Style},
        item::ItemKind,
        token_tree::error,
    },
};

/// One installed allocator expansion: its call, its kept owner and the root
/// of its generated items.
pub(super) struct Expanded {
    call: usize,
    owner: NodeKey,
    output: NodeId,
}

impl<S: Sources> Engine<'_, S> {
    /// Validate the attribute's form and item as rustc does, then install
    /// and visit the generated items and the kept owner.
    pub(super) fn expand_allocator(
        &mut self,
        index: usize,
        definition: usize,
    ) -> Result<(), Error> {
        let role = match self.definitions[definition].body {
            Body::Builtin(Builtin::GlobalAllocator) => Role::GlobalAllocator,
            Body::Builtin(Builtin::AllocErrorHandler) => Role::ErrorHandler,
            _ => {
                return Err(self.calls[index].diagnostic("allocator attribute has no actual role"));
            }
        };
        let name = self.allocator_owner(index, role)?;
        let call = &self.calls[index];
        let (file, owner, effect, parent) = (
            call.file,
            call.visit.node,
            call.capture as usize,
            call.visit.introduction,
        );
        let context = self.derive_invocation(index, definition, parent)?;
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
        let ledger = super::derive_lists::required_ledger(
            &self.loader.package,
            "allocator attribute requires its actual local provider",
        )
        .map_err(|message| self.calls[index].diagnostic(message))?;
        let unit = self.loader.package.unit(file);
        let original = unit.configured.effects[effect].attribute.attribute.span;
        let charge = |steps, bytes| {
            ledger
                .charge(steps, bytes)
                .map_err(|message| error(original.start, message))
        };
        let generated = allocator_output::build(
            &allocator_output::Site {
                file,
                source: unit.source.text(),
                original,
                name,
                crate_id: super::transported_macros::imported(&self.paths.state.facts, definition)
                    .map_or_else(
                        || {
                            self.loader
                                .package
                                .macro_crate(self.definitions[definition].module)
                        },
                        |imported| imported.crate_id,
                    ),
                context: (raw, context),
                edition: self.definitions[definition].edition,
                role,
            },
            self.config,
            &charge,
        )
        .map_err(|failure| Error::at(file, failure.offset, failure.message))?;
        ledger
            .charge(1, generated.payload_bytes().saturating_mul(4))
            .map_err(|message| self.calls[index].diagnostic(message))?;
        self.allocators
            .try_reserve(1)
            .map_err(|_| self.calls[index].diagnostic("allocator expansion allocation failed"))?;
        let unit = &mut self.loader.package.units[file.0 as usize];
        let (span, root) = unit
            .append_derived(
                &generated.stream.source,
                generated.syntax,
                &generated.stream.origins,
            )
            .map_err(|failure| Error::at(file, failure.offset, failure.message))?;
        unit.configured.effects[effect].attribute.effect = Effect::ExpandedAllocator;
        let invocation = self
            .invocations
            .last_mut()
            .filter(|invocation| invocation.context == context)
            .ok_or_else(|| Error::unlocated("allocator expansion lost its invocation"))?;
        invocation.generated = span;
        invocation.parser_calls = 1;
        self.allocators.push(Expanded {
            call: index,
            owner: NodeKey { file, node: owner },
            output: root,
        });
        self.calls[index].selected = Some(definition);
        self.calls[index].state = State::InProgress;
        self.resume_derived_owner(index, root, context)
    }
    /// The registered item's name, after rustc's checks of the attribute's
    /// input and of its item.
    fn allocator_owner(
        &self,
        index: usize,
        role: Role,
    ) -> Result<crate::syntax::token_tree::Span, Error> {
        let call = &self.calls[index];
        let unit = self.loader.package.unit(call.file);
        let attribute = unit.configured.effects[call.capture as usize]
            .attribute
            .attribute;
        if attribute.input != Input::Empty
            || attribute.unsafe_wrapper
            || attribute.style != Style::Outer
        {
            return Err(call.diagnostic(match role {
                Role::GlobalAllocator => "malformed `global_allocator` attribute input",
                Role::ErrorHandler => "malformed `alloc_error_handler` attribute input",
            }));
        }
        let mut node = call.visit.node;
        while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
            node = value;
        }
        let item = match unit.configured.ast.node(node).kind {
            Kind::Item(item) => Some(unit.configured.ast.items[item as usize]),
            _ => None,
        };
        let at = |message| {
            let mut failure = Error::at(
                call.file,
                unit.configured.ast.node(node).span.start,
                message,
            );
            failure.introduction = call.visit.introduction;
            failure
        };
        let name = match (role, item.map(|item| (item.kind, item.name))) {
            (Role::GlobalAllocator, Some((ItemKind::Static { .. }, Some(name))))
            | (Role::ErrorHandler, Some((ItemKind::Function { .. }, Some(name)))) => name,
            (Role::GlobalAllocator, _) => return Err(at("allocators must be statics")),
            (Role::ErrorHandler, _) => return Err(at("alloc_error_handler must be a function")),
        };
        let owner = unit
            .owner_indices
            .get(call.visit.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|owner| unit.configured.owners.get(owner.get() as usize - 1))
            .ok_or_else(|| call.diagnostic("allocator attribute lost its actual owner"))?;
        if role == Role::GlobalAllocator
            && unit
                .retained_attributes(owner.attributes.clone())
                .any(|(attribute, forest)| {
                    attribute.name(forest, unit.source.text()) == Some("thread_local")
                })
        {
            return Err(at("allocators cannot be `#[thread_local]`"));
        }
        Ok(name)
    }
    /// Whether this call installed an allocator expansion, so its configured
    /// effect is no longer `Keep`.
    pub(super) fn allocator_expanded(&self, index: usize) -> bool {
        self.allocators
            .iter()
            .any(|expanded| expanded.call == index)
    }
    /// Each installed expansion's kept owner and generated root, for
    /// placement after its owner.
    pub(super) fn allocator_outputs(&self) -> impl Iterator<Item = (NodeKey, NodeId)> + '_ {
        self.allocators
            .iter()
            .map(|expanded| (expanded.owner, expanded.output))
    }
}
