//! The two implicit calls keep separate inference terms and ordinary trait evidence.
use super::{Checker, Error, NodeId, TermId, Type, Work};
use crate::support::hash::HashMap;
use crate::{
    resolve::model::BuiltinIterationDeclaration,
    sema::iteration::{Call, ForPlan},
    support::{iteration::Trait, library_types::LibraryType},
    types::{
        ItemId, TypeId,
        infer::{Solved, arguments::Argument},
        lifetime_proof::ScopeId,
    },
};
#[derive(Default)]
pub(super) struct Pending {
    plans: HashMap<NodeId, Plan>,
}
struct Plan {
    into_iter: OpenCall,
    next: OpenCall,
    iterator: TermId,
    item: TermId,
}
struct OpenCall {
    callable: TermId,
    signature: TermId,
}
impl Checker<'_> {
    pub(super) fn for_iterator_work(
        &mut self,
        node: NodeId,
    ) -> Result<Option<super::Dependency>, Error> {
        if let Some(requirement) = self.for_collection_readiness(node)? {
            self.state.work.push(Work::ForIterator(node));
            return Ok(Some(super::Dependency {
                requirement,
                source: self.key(node),
            }));
        }
        self.for_iterator(node)?;
        Ok(None)
    }
    fn for_collection_readiness(
        &mut self,
        node: NodeId,
    ) -> Result<Option<super::Requirement>, Error> {
        let crate::syntax::ast::Kind::For { iterator, .. } = self.kind(node) else {
            return Err(self.error(node, "invalid iteration readiness continuation"));
        };
        let subject = self.term(iterator)?;
        if !self
            .facts
            .iteration
            .as_deref()
            .and_then(|r| r.ranges.as_deref())
            .is_some_and(|r| r.complete)
            && let Some(owner) = crate::sema::iteration::range_originals::requested_owner(
                self.report,
                &self.facts.types,
                &mut self.state.infer,
                subject,
            )
            .map_err(|e| self.error(node, e.to_string()))?
        {
            return Ok(Some(super::Requirement::ImplHeader(owner)));
        }
        let owner = crate::sema::iteration::collections::requested_owner(
            self.report,
            &self.facts.types,
            &mut self.state.infer,
            subject,
        )
        .map_err(|e| self.error(node, e.to_string()))?;
        let owner = owner.filter(|owner| {
            let mutable = self
                .report
                .resolution
                .builtin_traits
                .collections()
                .and_then(|c| c.declarations().find(|d| d.definition == *owner))
                .is_some_and(|d| d.kind.mutable());
            !self
                .facts
                .iteration
                .as_deref()
                .and_then(|r| r.collections.as_deref())
                .and_then(|r| r.group(mutable))
                .is_some_and(|g| g.status == crate::sema::iteration::collections::Status::Complete)
        });
        Ok(owner.map(super::Requirement::ImplHeader))
    }

    pub(super) fn for_iterator(&mut self, node: NodeId) -> Result<(), Error> {
        let crate::syntax::ast::Kind::For {
            pattern,
            iterator,
            body,
        } = self.kind(node)
        else {
            return Err(self.error(node, "invalid iteration continuation"));
        };
        let canonical = self
            .facts
            .iteration
            .as_deref()
            .ok_or_else(|| self.error(node, "missing canonical iteration contracts"))?
            .canonical;
        let subject = self.term(iterator)?;
        let into_iter = self.iteration_call(node, canonical.get(Trait::IntoIterator), subject)?;
        let iterator_type = self.child(node, into_iter.signature, 1)?;
        let next = self.iteration_call(node, canonical.get(Trait::Iterator), iterator_type)?;
        let option = self.child(node, next.signature, 1)?;
        let projected_item = self.child(node, option, 0)?;
        let assumptions = self.trait_assumptions(node)?;
        let item = if let Some(element) = crate::sema::traits::selection::range_element(
            self.facts,
            &mut self.state.infer,
            subject,
            assumptions.as_deref(),
        )
        .map_err(|e| self.error(node, e))?
        {
            // This equality is still retained beside the original complete
            // projection and both call obligations. It grants no selected body.
            self.shape_equation(node, projected_item, element)?;
            element
        } else {
            projected_item
        };
        self.state
            .call_types
            .iteration
            .get_or_insert_default()
            .plans
            .insert(
                node,
                Plan {
                    into_iter,
                    next,
                    iterator: iterator_type,
                    item,
                },
            );
        self.state.work.push(Work::Enter(body, None));
        self.state.work.push(Work::ForPattern(pattern, item));
        Ok(())
    }
    fn iteration_call(
        &mut self,
        node: NodeId,
        declaration: BuiltinIterationDeclaration,
        subject: TermId,
    ) -> Result<OpenCall, Error> {
        let Some(Type::FunctionItem {
            signature, item, ..
        }) = self.facts.definitions[declaration.method.0 as usize]
            .and_then(|ty| self.facts.types.kind(ty))
        else {
            return Err(self.error(node, "missing canonical iteration method signature"));
        };
        let bindings = [(
            self.operator_parameter(node, declaration.self_parameter)?,
            Argument::Type(subject),
        )];
        let signature = self
            .state
            .infer
            .instantiate(&mut self.facts.types, signature, &bindings)
            .map_err(|e| self.error(node, e))?;
        self.defer_normalization(node, ScopeId::ROOT, signature)?;
        let arguments = [Argument::Type(subject)];
        self.bound_obligations(node, declaration.method, &arguments, &bindings)?;
        let callable = self
            .state
            .infer
            .function_item(item, signature, &arguments)
            .map_err(|e| self.error(node, e))?;
        self.state
            .call_types
            .generic
            .push((node, declaration.method, callable));
        let signature = self.call_signature(node, signature)?;
        self.state.calls.push((node, signature));
        Ok(OpenCall {
            callable,
            signature,
        })
    }
    pub(super) fn publish_for_plans(&mut self, solved: &Solved) -> Result<(), Error> {
        let Some(pending) = self.state.call_types.iteration.take() else {
            return Ok(());
        };
        let option = self
            .report
            .resolution
            .library_types
            .get(LibraryType::Option)
            .ok_or_else(|| {
                self.error(self.state.body.node, "missing canonical iteration Option")
            })?;
        let option_item = ItemId(option.definition.0);
        let variants = self.facts.members.variants(option_item);
        let [none, some] = variants else {
            return Err(self.error(
                self.state.body.node,
                "invalid canonical iteration Option variants",
            ));
        };
        let (none, some) = (*none, *some);
        for (node, plan) in pending.plans {
            let call = |open: OpenCall| Call {
                callable: solved.types[open.callable.0 as usize],
                signature: solved.types[open.signature.0 as usize],
            };
            let into_iter = call(plan.into_iter);
            let next = call(plan.next);
            let iterator = solved.types[plan.iterator.0 as usize];
            let item = solved.types[plan.item.0 as usize];
            let Some(Type::Function {
                parameters,
                result: option,
                bound_regions: 0,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
            }) = self.facts.types.kind(next.signature)
            else {
                return Err(self.error(
                    node,
                    "iteration next signature is not an opened safe Rust call",
                ));
            };
            let [receiver] = self
                .facts
                .types
                .list(parameters)
                .ok_or_else(|| self.error(node, "missing iteration receiver"))?
            else {
                return Err(self.error(node, "iteration next signature has incorrect arity"));
            };
            if !matches!(self.facts.types.kind(*receiver), Some(Type::Reference {referent, mutable: true, ..}) if referent == iterator)
            {
                return Err(self.error(
                    node,
                    "iteration next receiver differs from selected iterator",
                ));
            }
            let Some(Type::Adt(owner, arguments)) = self.facts.types.kind(option) else {
                return Err(self.error(node, "iteration next does not return canonical Option"));
            };
            if owner != option_item
                || self
                    .facts
                    .types
                    .arguments(arguments)
                    .is_none_or(|mut args| {
                        args.next() != Some(crate::types::arguments::Argument::Type(item))
                            || args.next().is_some()
                    })
            {
                return Err(self.error(
                    node,
                    "iteration Option Item differs from the selected contract",
                ));
            }
            if !matches!(self.facts.types.kind(into_iter.signature), Some(Type::Function {result, ..}) if result == iterator)
            {
                return Err(self.error(node, "iteration result differs from the selected IntoIter"));
            }
            self.prepare_iteration_storage(node, iterator, option)?;
            let key = self.key(node);
            self.facts
                .iteration
                .as_deref_mut()
                .ok_or_else(|| Error::at(self.state.file, 0, "missing iteration plan owner"))?
                .plans
                .insert(
                    key,
                    ForPlan {
                        into_iter,
                        next,
                        item,
                        iterator,
                        option,
                        some,
                        none,
                    },
                );
        }
        Ok(())
    }
    fn prepare_iteration_storage(
        &mut self,
        node: NodeId,
        iterator: TypeId,
        option: TypeId,
    ) -> Result<(), Error> {
        // These synthesized call results become owned loop storage. Their
        // exact solved shapes must precede CFG construction and drop queries.
        for ty in [iterator, option] {
            self.facts
                .aggregates
                .prepare(&mut self.facts.types, ty)
                .map_err(|e| self.error(node, e))?;
        }
        Ok(())
    }
}
