//! Defining abstract calls consume exact source facts and a local Self premise.
use super::{Declarations, DefId, FunctionContext, NodeKey};
use crate::{
    sema::model::Operation,
    support::constness::Constness,
    types::{
        Kind, TypeId,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

#[derive(Clone, Copy)]
pub(crate) struct Call {
    pub site: NodeKey,
    pub target: TypeId,
    pub signature: TypeId,
    pub result: TypeId,
}

/// A scoped defining check, without a selected implementation or execution token.
pub(crate) struct DefiningCalls<'a> {
    facts: &'a Declarations,
    owner: NodeKey,
    trait_definition: DefId,
    self_type: TypeId,
}
impl<'a> DefiningCalls<'a> {
    pub(crate) fn new(
        facts: &'a Declarations,
        definition: DefId,
        owner: NodeKey,
        mut charge: impl FnMut(usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        if facts.source_bindings.is_none() {
            return Err(
                "abstract const-trait defining-call facts outside source provider are not implemented",
            );
        }
        charge(5)?;
        let FunctionContext::ConstTraitDefault { trait_definition } =
            facts.function_context(definition, owner)?
        else {
            return Err(
                "abstract const member checking outside its actual provided body is not implemented",
            );
        };
        charge(9)?;
        let mut predicate = facts.const_self_predicate(trait_definition)?;
        let Bound::Trait(application) = predicate.bound else {
            return Err("missing ordinary Self application");
        };
        let application_data = facts
            .types
            .trait_application(application)
            .ok_or("missing defining Self application")?;
        if facts
            .types
            .arguments(application_data.arguments)
            .is_none_or(|arguments| arguments.len() != 0)
        {
            return Err("generic abstract const-trait defining calls are not implemented");
        }
        predicate.bound = Bound::MaybeConstTrait(application);
        charge(1)?;
        let environment = facts
            .generics
            .body_environment(definition)
            .ok_or("abstract const call has no actual defining environment")?;
        let mut found = false;
        // This one local preparation scan is shared by every call in this body.
        let mut clauses = environment.trait_clauses();
        loop {
            charge(1)?;
            let Some(clause) = clauses.next() else {
                break;
            };
            found |= clause == Clause::Predicate(predicate);
        }
        if !found {
            return Err("abstract const call lacks its exact local conditional Self premise");
        }
        Ok(Self {
            facts,
            owner,
            trait_definition,
            self_type: predicate.ty,
        })
    }

    pub(crate) fn check(
        &self,
        call: Call,
        mut charge: impl FnMut(usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        charge(1)?;
        let Some(Kind::FunctionItem {
            item, arguments, ..
        }) = self.facts.types.kind(call.target)
        else {
            return Err("abstract const call lost its actual complete function item");
        };
        let definition = DefId(item.0);
        charge(3)?;
        let mut arguments = self
            .facts
            .types
            .arguments(arguments)
            .ok_or("abstract const call has malformed complete arguments")?;
        if arguments.next() != Some(crate::types::arguments::Argument::Type(self.self_type))
            || arguments.next().is_some()
        {
            return Err("abstract const call requires its exact original Self application");
        }
        charge(3)?;
        if self.facts.associated.trait_owner(definition) != Some(self.trait_definition) {
            return Err("abstract const call belongs to another actual trait member");
        }
        charge(1)?;
        let node = self
            .facts
            .node(call.site)
            .ok_or("abstract const call has no actual typed source site")?;
        let Some(Operation::Call {
            signature,
            source_use,
        }) = node.operation
        else {
            return Err("abstract const call changed its typed source operation");
        };
        if signature != call.signature || node.ty != Some(call.result) {
            return Err("abstract const call changed its opened signature or result");
        }
        charge(1)?;
        if source_use
            .ok_or("abstract const call has no defining source-use fact")?
            .effect_for(self.facts, self.owner)?
            != Constness::Conditional
        {
            return Err("abstract const call changed its defining conditional demand");
        }
        charge(1)?;
        if let Some(method) = self.facts.methods.calls.get(&call.site) {
            if method.method != definition || method.callable != call.target {
                return Err("abstract const call changed its exact selected method application");
            }
        } else {
            charge(1)?;
            let invocation = self
                .facts
                .invocations
                .get(&call.site)
                .ok_or("abstract const call has no exact selected callable fact")?;
            if invocation.kind.is_some()
                || invocation.receiver != call.target
                || invocation.dereferences != 0
                || invocation.borrow.is_some()
            {
                return Err("abstract const call changed its exact UFCS callable application");
            }
        }
        Ok(())
    }
}
