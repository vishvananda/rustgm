//! Formatting selects an actual inherent constructor through ordinary call checking.
#[cfg(test)]
#[path = "../../../tests/invariants/formatting_calls.rs"]
mod tests;
use super::{Checker, Constructor, DefId, Error, NodeId, Requirement, TermId, Type, TypeId};
use crate::{
    sema::{
        methods::selection::{PathCaller, PathProgress, PathQuery},
        model::BodyRegionOrigin,
        traits::implementations::Availability,
    },
    support::{bindings::LangItem, library_types::LibraryValue, names::NameId},
    syntax::ast::Kind,
    types::{ItemId, arguments::Argument},
};

pub(super) struct Pending {
    node: NodeId,
    owner: ItemId,
    constructor: NameId,
    prefix: TypeId,
    subject: TermId,
    headers: bool,
    query: Option<PathQuery>,
    selected: Option<DefId>,
}
impl Checker<'_> {
    pub(super) fn formatting_readiness(
        &mut self,
        node: NodeId,
    ) -> Result<Option<Requirement>, Error> {
        let Kind::LibraryValue {
            kind: LibraryValue::FormatLiteral { constructor },
            value: Some(_),
        } = self.kind(node)
        else {
            return Ok(None);
        };
        if self.state.call_types.formatting.is_none() {
            self.report.prepare_formatting_value(
                self.key(node),
                size_of::<Pending>() + size_of::<Box<Pending>>(),
            )?;
            let owner = self.formatting_owner(node, constructor)?;
            if self.facts.aggregates.get(owner).is_none()
                || !self
                    .facts
                    .lifetimes
                    .parameters
                    .contains_key(&DefId(owner.0))
            {
                return Ok(Some(Requirement::Nominal(owner)));
            }
            let pending = self.formatting_prefix(node, owner, constructor)?;
            self.state.call_types.formatting = Some(Box::new(pending));
        }
        let mut pending = self.state.call_types.formatting.take().ok_or_else(|| {
            self.error(node, "missing actual formatting constructor continuation")
        })?;
        if pending.node != node || pending.constructor != constructor {
            return Err(self.error(node, "formatting continuation belongs to another value"));
        }
        let dependency = self.poll_formatting(node, &mut pending)?;
        self.state.call_types.formatting = Some(pending);
        Ok(dependency)
    }
    fn formatting_owner(&self, node: NodeId, constructor: NameId) -> Result<ItemId, Error> {
        if constructor.0 as usize >= self.report.package.names.len()
            || self.report.package.names.get(constructor) != "from_str"
        {
            return Err(self.error(
                node,
                "formatting changed its canonical constructor selector",
            ));
        }
        let declaration = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(LangItem::FormatArguments))
            .ok_or_else(|| {
                self.error(
                    node,
                    "formatting requires the actual format_arguments owner",
                )
            })?;
        Ok(ItemId(declaration.definition.0))
    }
    fn formatting_prefix(
        &mut self,
        node: NodeId,
        owner: ItemId,
        constructor: NameId,
    ) -> Result<Pending, Error> {
        let declaration = crate::sema::compiler_items::formatting::declaration(
            self.report,
            self.facts,
            LangItem::FormatArguments,
        )
        .map_err(|message| self.error(node, message))?
        .ok_or_else(|| self.error(node, "missing actual formatting nominal"))?;
        if declaration.definition.0 != owner.0 {
            return Err(self.error(node, "formatting nominal changed its actual owner"));
        }
        let region = self.body_region(node, BodyRegionOrigin::ItemArgument)?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Lifetime(region)])
            .map_err(|message| self.error(node, message))?;
        let prefix = self
            .facts
            .types
            .intern(Type::Adt(owner, arguments))
            .map_err(|message| self.error(node, message))?;
        let subject = self.known(node, prefix)?;
        Ok(Pending {
            node,
            owner,
            constructor,
            prefix,
            subject,
            headers: false,
            query: None,
            selected: None,
        })
    }
    fn poll_formatting(
        &mut self,
        node: NodeId,
        pending: &mut Pending,
    ) -> Result<Option<Requirement>, Error> {
        if !pending.headers {
            pending.headers = true;
            if matches!(self.availability, Availability::Declarations) {
                return Ok(Some(Requirement::Inherent(
                    pending.owner,
                    pending.constructor,
                )));
            }
        }
        if pending.selected.is_none() {
            if pending.query.is_none() {
                let scope = self
                    .report
                    .resolution
                    .node_scopes
                    .get(&(self.state.module, node))
                    .copied()
                    .ok_or_else(|| self.error(node, "missing actual formatting caller scope"))?;
                let caller = PathCaller {
                    availability: self.availability,
                    environment: self.trait_query_environment(node)?,
                    traits: self.trait_assumptions(node)?,
                };
                pending.query = Some(PathQuery::inherent_type(
                    self.report,
                    self.facts,
                    self.key(node),
                    pending.prefix,
                    (pending.owner, pending.constructor),
                    crate::sema::methods::names::Use::path(self.report, self.key(node), scope)?,
                    caller,
                )?);
            }
            let key = self.key(node);
            let progress = pending
                .query
                .as_mut()
                .ok_or_else(|| self.error(node, "missing formatting selection query"))?
                .poll(self.report, self.facts, key)
                .map_err(|message| self.error(node, message))?;
            match progress {
                PathProgress::Pending(dependency) => return Ok(Some(dependency.into())),
                PathProgress::Ready(Some(method)) => pending.selected = Some(method),
                PathProgress::Ready(None) => {
                    return Err(self.error(node, "no applicable formatting constructor"));
                }
            }
            pending.query = None;
        }
        let method = pending
            .selected
            .ok_or_else(|| self.error(node, "missing formatting constructor"))?;
        if self
            .facts
            .definitions
            .get(method.0 as usize)
            .copied()
            .flatten()
            .is_none()
        {
            return Ok(Some(Requirement::Definition(method)));
        }
        Ok(None)
    }
    pub(super) fn formatting_literal(
        &mut self,
        node: NodeId,
        value: Option<NodeId>,
    ) -> Result<TermId, Error> {
        let value = value.ok_or_else(|| self.error(node, "missing actual formatting literal"))?;
        let pending = self.state.call_types.formatting.take().ok_or_else(|| {
            self.error(node, "formatting has no checked constructor continuation")
        })?;
        if pending.node != node {
            return Err(self.error(node, "formatting constructor belongs to another value"));
        }
        let method = pending
            .selected
            .ok_or_else(|| self.error(node, "unselected formatting constructor"))?;
        let owner =
            self.facts.associated.member_owner(method).ok_or_else(|| {
                self.error(node, "formatting constructor has no actual impl owner")
            })?;
        self.state
            .call_types
            .method_owners
            .insert(node, (owner, pending.subject));
        self.complete_method_arguments(node, method)?;
        let ty = self.facts.definitions[method.0 as usize]
            .ok_or_else(|| self.error(node, "missing actual formatting callable"))?;
        let callable = self.generic_function(node, method, ty)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        if !matches!(
            self.head(node, signature)?,
            Some(Constructor::Function {
                parameters: 1,
                convention: crate::types::callable::Convention::RUST,
                ..
            })
        ) {
            return Err(self.error(
                node,
                "formatting constructor requires its one-input Rust signature",
            ));
        }
        let input = self.child(node, signature, 0)?;
        self.coerce(value, input)?;
        let result = self.child(node, signature, 1)?;
        self.equate(node, result, pending.subject)?;
        self.retain_language_method(node, method, callable, pending.subject);
        let target_function = self.selected_target_function(node, callable)?;
        self.retain_active_call(node, signature, target_function)?;
        self.call(node, node, crate::syntax::ast::List { start: 0, end: 0 })
    }
}
