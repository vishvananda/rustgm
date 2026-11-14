//! Defining abstract calls consume exact source facts and a local Self premise.
mod conditional_arguments;
use super::{Declarations, DefId, FunctionContext, NodeKey};
use crate::{
    sema::model::Operation,
    support::constness::Constness,
    types::{
        GenericArgs, Kind, TypeId,
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
    environment: &'a crate::types::parameters::environment::Environment,
    premise: Premise,
}
enum Premise {
    Trait {
        subject: TypeId,
        application: crate::types::traits::ApplicationId,
    },
    Function {
        effect: Constness,
    },
}
impl<'a> DefiningCalls<'a> {
    pub(crate) fn new(
        facts: &'a Declarations,
        definition: DefId,
        owner: NodeKey,
        mut charge: impl FnMut(usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        charge(5)?;
        let context = facts.function_context(definition, owner)?;
        let trait_definition = match context {
            FunctionContext::ConstTraitDefault { trait_definition } => trait_definition,
            FunctionContext::ConstFunction | FunctionContext::Comptime => {
                charge(1)?;
                let environment = facts
                    .generics
                    .body_environment(definition)
                    .ok_or("const function member call has no actual defining environment")?;
                return Ok(Self {
                    facts,
                    owner,
                    environment,
                    premise: Premise::Function {
                        effect: if context == FunctionContext::Comptime {
                            Constness::Const
                        } else {
                            Constness::Conditional
                        },
                    },
                });
            }
            FunctionContext::Runtime => {
                return Err(
                    "abstract const member checking outside its actual provided body is not implemented",
                );
            }
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
        if application_data.item.0 != trait_definition.0 {
            return Err("defining Self application belongs to another trait");
        }
        predicate.bound = Bound::MaybeConstTrait(application);
        charge(1)?;
        let environment = facts
            .generics
            .body_environment(definition)
            .ok_or("abstract const call has no actual defining environment")?;
        let mut found = false;
        // This one local preparation scan is shared by every call in this body.
        let mut clauses = environment.trait_clauses()?;
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
            environment,
            premise: Premise::Trait {
                subject: predicate.ty,
                application,
            },
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
        self.check_premise(definition, arguments, &mut charge)?;
        let (source_use, field) = self.source_use(call, &mut charge)?;
        charge(1)?;
        let effect = match self.premise {
            Premise::Function { effect, .. } => effect,
            Premise::Trait { .. } => Constness::Conditional,
        };
        if source_use.effect_for(self.facts, self.owner)? != effect {
            return Err(if effect == Constness::Conditional {
                "abstract const call changed its defining conditional demand"
            } else {
                "abstract const call changed its defining unconditional const demand"
            });
        }
        charge(1)?;
        if field {
            return Ok(());
        }
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

    fn source_use(
        &self,
        call: Call,
        charge: &mut impl FnMut(usize) -> Result<(), &'static str>,
    ) -> Result<(crate::sema::bodies::SourceCallUse, bool), &'static str> {
        charge(1)?;
        if let Some(id) = crate::sema::protocol_plans::access::conversion_call(
            self.facts,
            self.owner,
            call.site,
            crate::sema::iteration::Call {
                callable: call.target,
                signature: call.signature,
            },
        )? {
            let plan = id.current(self.facts, self.owner, call.site)?;
            crate::sema::protocol_plans::access::retained_call(
                self.facts,
                plan,
                self.owner,
                crate::sema::iteration::Call {
                    callable: call.target,
                    signature: call.signature,
                },
            )
            .map_err(|_| "abstract conversion lost its actual retained call")?;
            if !matches!(self.facts.types.kind(call.signature), Some(Kind::Function { result, .. }) if result == call.result)
            {
                return Err("abstract conversion changed its actual call result");
            }
            return Ok((plan.source_use, true));
        }
        let node = self
            .facts
            .node(call.site)
            .ok_or("abstract const call has no actual typed source site")?;
        match node.operation {
            Some(Operation::Call {
                signature,
                source_use,
            }) => {
                if signature != call.signature || node.ty != Some(call.result) {
                    return Err("abstract const call changed its opened signature or result");
                }
                Ok((
                    source_use.ok_or("abstract const call has no defining source-use fact")?,
                    false,
                ))
            }
            Some(
                Operation::Field { plan: Some(id), .. } | Operation::MethodReceiver { plan: id },
            ) => {
                let plan = id.current(self.facts, self.owner, call.site)?;
                let Some((_, _, _, steps)) = plan.kind.access() else {
                    return Err("abstract const field changed its actual protocol");
                };
                charge(
                    steps
                        .end
                        .saturating_sub(steps.start)
                        .checked_ilog2()
                        .map_or(1, |n| n as usize + 2),
                )?;
                let step = crate::sema::protocol_plans::access::retained_call(
                    self.facts,
                    plan,
                    self.owner,
                    crate::sema::iteration::Call {
                        callable: call.target,
                        signature: call.signature,
                    },
                )
                .map_err(|_| "abstract const field lost its actual retained call")?;
                if !matches!(self.facts.types.kind(call.signature), Some(Kind::Function { result, .. }) if result == call.result)
                {
                    return Err("abstract const field changed its actual call result");
                }
                Ok((plan.source_use, step))
            }
            _ => Err("abstract const call changed its typed source operation"),
        }
    }

    fn check_premise(
        &self,
        definition: DefId,
        arguments: GenericArgs,
        charge: &mut impl FnMut(usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let predicate = conditional_arguments::select(self.facts, definition, arguments, charge)?;
        let Bound::MaybeConstTrait(application) = predicate.bound else {
            return Err("abstract const call lost its conditional request");
        };
        charge(3)?;
        match self.premise {
            Premise::Trait {
                subject,
                application: defining,
            } => {
                if predicate.ty != subject {
                    return Err(
                        "abstract const call changed its defining Self or trait application",
                    );
                }
                if application != defining {
                    // Other Self applications must be paid by this completed
                    // defining environment, including elaborated supertraits.
                    charge(2)?;
                    let unconditional = self.environment.assumes_predicate(
                        crate::types::parameters::predicates::Predicate {
                            bound: Bound::ConstTrait(application),
                            ..predicate
                        },
                    );
                    if !unconditional && !self.environment.assumes_predicate(predicate) {
                        return Err(
                            "abstract const call changed its defining Self or trait application",
                        );
                    }
                }
            }
            Premise::Function { effect } => {
                let unconditional = self.environment.assumes_predicate(
                    crate::types::parameters::predicates::Predicate {
                        bound: Bound::ConstTrait(application),
                        ..predicate
                    },
                );
                if !unconditional
                    && (effect == Constness::Const
                        || !self.environment.assumes_predicate(predicate))
                {
                    return Err(
                        "const function member lacks its current conditional trait premise",
                    );
                }
            }
        }
        Ok(())
    }
}
