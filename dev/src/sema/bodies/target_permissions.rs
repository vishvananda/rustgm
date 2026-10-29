//! Direct calls keep their canonical signature; pointer views adjust safety contextually.
use super::{Checker, Constructor, DefId, Error, NodeId, TermId, calls::ActiveCall};
use crate::{
    sema::{
        header_attributes,
        target_features::{self, FeatureSet, Policy},
    },
    support::retained::Growth,
};
#[derive(PartialEq, Eq)]
enum Permission {
    Safe,
    UnsafeFunction,
    MissingFeatures,
}
impl Checker<'_> {
    pub(super) fn selected_target_function(
        &mut self,
        node: NodeId,
        mut callable: TermId,
    ) -> Result<Option<DefId>, Error> {
        if self.facts.target_feature_functions.is_empty() {
            return Ok(None);
        }
        loop {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(node, e))?;
            match self.head(node, callable)? {
                Some(Constructor::Reference { .. }) => callable = self.child(node, callable, 0)?,
                Some(
                    Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _),
                ) => {
                    let id = DefId(item.0);
                    return Ok(self
                        .facts
                        .target_feature_functions
                        .contains_key(&id)
                        .then_some(id));
                }
                _ => return Ok(None),
            }
        }
    }
    pub(super) fn retain_active_call(
        &mut self,
        node: NodeId,
        signature: TermId,
        target_function: Option<DefId>,
    ) -> Result<(), Error> {
        let active = &self.state.call_types.active;
        let growth = Growth::map::<NodeId, ActiveCall>(active.len(), active.capacity(), 1);
        self.state
            .infer
            .account_work(growth.work.saturating_add(1))
            .map_err(|e| self.error(node, e))?;
        if target_function.is_some() {
            header_attributes::charge_attribute_retention(self.facts, 1, growth.bytes)?;
        }
        self.state
            .call_types
            .active
            .try_reserve(1)
            .map_err(|_| self.error(node, "pending call allocation failed"))?;
        if self
            .state
            .call_types
            .active
            .insert(
                node,
                ActiveCall {
                    signature,
                    target_function,
                },
            )
            .is_some()
        {
            return Err(self.error(node, "duplicate pending call"));
        }
        Ok(())
    }
    pub(super) fn target_body_features(&mut self, node: NodeId) -> Result<FeatureSet, Error> {
        if !self.state.in_function {
            return Ok(FeatureSet::default());
        }
        let depth = self
            .state
            .closures
            .as_ref()
            .map_or(0, |scheduler| scheduler.stack.len());
        self.state
            .infer
            .account_work(depth.saturating_add(1))
            .map_err(|e| self.error(node, e))?;
        if self
            .state
            .closures
            .as_ref()
            .is_some_and(|scheduler| scheduler.stack.iter().any(|(_, state)| !state.in_function))
        {
            return Ok(FeatureSet::default());
        }
        let Some(owner) = self.state.environment_owner else {
            return Ok(FeatureSet::default());
        };
        let root = self
            .state
            .closures
            .as_ref()
            .and_then(|scheduler| scheduler.stack.first())
            .map_or(self.state.body, |(_, state)| state.body);
        let header = self
            .facts
            .declaration_header(owner)
            .map_err(|e| self.error(node, e))?
            .ok_or_else(|| self.error(node, "target-feature caller lacks its body declaration"))?;
        if header
            .body
            .and_then(crate::resolve::body_origin::BodyOrigin::source)
            != Some(root)
        {
            return Err(self.error(node, "target-feature caller body changed its actual owner"));
        }
        match target_features::query(self.report, self.facts, owner)? {
            Policy::Local(Some(row)) => Ok(row.features()),
            Policy::Local(None) => Ok(FeatureSet::default()),
            Policy::ImportedUnknown => {
                Err(self.error(node, "imported target-feature body policy is unavailable"))
            }
        }
    }
    fn target_function_permission(
        &mut self,
        node: NodeId,
        definition: DefId,
    ) -> Result<Permission, Error> {
        let (features, declared_unsafe) =
            match target_features::query(self.report, self.facts, definition)? {
                Policy::Local(Some(row)) => (row.features(), row.declared_unsafe()),
                Policy::Local(None) | Policy::ImportedUnknown => {
                    return Err(self.error(
                        node,
                        "selected target-feature policy lost its actual producer",
                    ));
                }
            };
        if declared_unsafe {
            return Ok(Permission::UnsafeFunction);
        }
        let available = self.target_body_features(node)?;
        header_attributes::charge_source_contract_query(self.facts, 2)?;
        Ok(if available.covers(features) {
            Permission::Safe
        } else {
            Permission::MissingFeatures
        })
    }
    pub(super) fn target_call_permission(
        &mut self,
        node: NodeId,
        target: Option<DefId>,
        unsafe_: bool,
    ) -> Result<(), Error> {
        if let Some(target) = target {
            return match self.target_function_permission(node, target)? {
                Permission::Safe => Ok(()),
                Permission::UnsafeFunction => self.require_unsafe_call(node),
                Permission::MissingFeatures => self.require_unsafe(
                    node,
                    "call requires an unsafe context or the callee's target features",
                ),
            };
        }
        if unsafe_ {
            self.require_unsafe_call(node)?;
        }
        Ok(())
    }
    pub(super) fn target_pointer_signature(
        &mut self,
        node: NodeId,
        callable: TermId,
        signature: TermId,
    ) -> Result<TermId, Error> {
        let Some(target) = self.selected_target_function(node, callable)? else {
            return Ok(signature);
        };
        if self.target_function_permission(node, target)? != Permission::Safe {
            return Ok(signature);
        }
        let Some(Constructor::Function {
            bound_regions,
            parameters,
            convention,
            ..
        }) = self.head(node, signature)?
        else {
            return Err(self.error(node, "target-feature callable has no signature"));
        };
        let count = parameters as usize + 1;
        let growth = Growth::vector::<TermId>(0, 0, count);
        header_attributes::charge_attribute_retention(
            self.facts,
            growth.work.saturating_add(count),
            growth.bytes,
        )?;
        let mut children = Vec::with_capacity(count);
        for index in 0..count {
            children.push(self.child(node, signature, index)?);
        }
        self.state
            .infer
            .construct(
                Constructor::Function {
                    bound_regions,
                    parameters,
                    unsafe_: false,
                    convention,
                },
                &children,
            )
            .map_err(|e| self.error(node, e))
    }
}

#[cfg(test)]
pub(in crate::sema) fn observe_active_calls(
    report: &crate::resolve::expansion::Report,
    source: &str,
) -> Result<[usize; 9], Box<dyn std::error::Error>> {
    let mut facts = crate::sema::declarations::analyze(report)?;
    let function = *facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == source
        })
        .ok_or("actual measured function")?;
    let mut checker = Checker::new(report, &mut facts);
    let progress = checker.start_function(function)?;
    checker.require_complete_body(progress)?;
    Ok([
        size_of::<ActiveCall>(),
        size_of::<(NodeId, ActiveCall)>(),
        checker.state.call_types.active.capacity(),
        crate::support::retained::map(&checker.state.call_types.active),
        checker.state.infer.steps,
        size_of::<super::calls::CallTypes>(),
        size_of::<super::State>(),
        checker.facts.target_feature_functions.len(),
        usize::from(
            checker
                .facts
                .must_use_contracts
                .as_deref()
                .is_some_and(|owner| owner.headers.is_some()),
        ),
    ])
}

#[cfg(test)]
#[path = "../../../tests/invariants/asm_cpu_contexts.rs"]
mod assembly_tests;
