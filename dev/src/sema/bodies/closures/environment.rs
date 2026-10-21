//! Selected captures build one typed environment and retain both lifetime views.
use super::{CallKind, Checker, Constructor, Entry, Error, NodeId, TypeId};
use crate::{
    resolve::unit::NodeKey,
    sema::{
        closures::captures::Mode,
        closures::{Environment, Execution},
        model::BodyRegionOrigin,
    },
    types::{Kind, RegionId, infer::Solved, substitute::Bound},
};
use std::collections::HashMap;

impl Checker<'_> {
    pub(super) fn collect_capture_environment(
        &mut self,
        entry: &mut Entry,
        parent: NodeKey,
        children: HashMap<NodeId, Option<&super::uses::Captures>>,
    ) -> Result<CallKind, Error> {
        let source = entry.source;
        let metadata = entry
            .payload
            .closure_mut()
            .map_err(|e| self.error(source, e))?;
        if metadata.bindings.is_empty() {
            return Ok(CallKind::Shared);
        }
        let captures =
            self.closure_uses(source, metadata.root.node, &metadata.bindings, children)?;
        let kind = captures.kind;
        for capture in captures.plan.captures() {
            let mut ty = captures.types[capture.path.index()];
            if capture.mode != Mode::Value {
                let region =
                    self.body_region_for(source, parent, BodyRegionOrigin::ExpressionBorrow)?;
                let region = metadata
                    .lifetimes
                    .opening
                    .region(region, 0)
                    .map_err(|e| self.error(source, e))?;
                ty = self
                    .state
                    .infer
                    .construct(
                        Constructor::Reference {
                            mutable: capture.mode != Mode::Shared,
                            region,
                        },
                        &[ty],
                    )
                    .map_err(|e| self.error(source, e))?;
            }
            metadata.fields.push(ty);
        }
        self.state.body_binders = metadata
            .lifetimes
            .count()
            .map_err(|e| self.error(source, e))?;
        self.prepare_universals(source)?;
        let environment = self
            .state
            .infer
            .construct(Constructor::Tuple, &metadata.fields)
            .map_err(|e| self.error(source, e))?;
        let target = self.child(source, metadata.self_ty, 1)?;
        self.equate(source, target, environment)?;
        metadata.captures = Some(Box::new(captures));
        Ok(kind)
    }

    pub(super) fn publish_capture_environment(
        &mut self,
        solved: &Solved,
        signature: TypeId,
    ) -> Result<Option<Execution>, Error> {
        let Some(mut scheduler) = self.state.closures.take() else {
            return Ok(None);
        };
        let result = self.complete_capture_environment(&mut scheduler, solved, signature);
        self.state.closures = Some(scheduler);
        result
    }
    fn complete_capture_environment(
        &mut self,
        scheduler: &mut super::Scheduler,
        solved: &Solved,
        signature: TypeId,
    ) -> Result<Option<Execution>, Error> {
        let Some(index) = scheduler.active else {
            return Ok(None);
        };
        let entry = &mut scheduler.entries[index];
        let source = entry.source;
        let metadata = entry
            .payload
            .closure_mut()
            .map_err(|e| self.error(source, e))?;
        let Some(captures) = metadata.captures.take() else {
            return Ok(None);
        };
        let mut body_fold = self.capture_body_fold(source, solved)?;
        let mut formation = metadata
            .lifetimes
            .closing(&self.state.call_types, false)
            .map_err(|e| self.error(source, e))?;
        let mut path_types = Vec::with_capacity(captures.types.len());
        let mut formation_types = Vec::with_capacity(captures.types.len());
        for term in &captures.types {
            let ty = solved.types[term.0 as usize];
            formation_types.push(
                formation
                    .apply(&mut self.facts.types, ty)
                    .map_err(|e| self.error(source, e))?,
            );
            path_types.push(
                body_fold
                    .apply(&mut self.facts.types, ty)
                    .map_err(|e| self.error(source, e))?,
            );
        }
        let fields = metadata
            .fields
            .iter()
            .map(|term| solved.types[term.0 as usize])
            .collect::<Vec<_>>();
        let mut field_types = Vec::with_capacity(fields.len());
        for ty in fields {
            field_types.push(
                body_fold
                    .apply(&mut self.facts.types, ty)
                    .map_err(|e| self.error(source, e))?,
            );
        }
        let self_ty = body_fold
            .apply(
                &mut self.facts.types,
                solved.types[metadata.self_ty.0 as usize],
            )
            .map_err(|e| self.error(source, e))?;
        let (self_ty, signature_work) = metadata
            .lifetimes
            .execution_self(&mut self.facts.types, self_ty, signature)
            .map_err(|e| self.error(source, e))?;
        self.facts.body_work.canonical_steps += signature_work;
        let mut binding_mutability = HashMap::new();
        for &(definition, _) in &metadata.bindings {
            let key = self.report.resolution.definitions[definition.0 as usize]
                .node
                .ok_or_else(|| self.error(source, "capture binding lacks source identity"))?;
            let mutable = match crate::sema::model::Declarations::kind(self.report, key) {
                crate::syntax::ast::Kind::Binding {
                    mutable, by_ref, ..
                } => mutable && !by_ref,
                // An explicit-lifetime receiver's `mut` qualifies its referent.
                // Its binding is immutable, as in declaration completion.
                crate::syntax::ast::Kind::Receiver { .. } => false,
                _ => return Err(self.error(source, "capture binding lacks mutability fact")),
            };
            binding_mutability.insert(definition, mutable);
        }
        let execution = if captures.plan.captures().is_empty() {
            None
        } else {
            Some(self.capture_execution(source, self_ty, signature, captures.kind)?)
        };
        self.facts.body_work.canonical_steps +=
            body_fold.steps + formation.steps + captures.work + captures.plan.work;
        self.facts.closures.environments.insert(
            metadata.identity,
            Environment {
                plan: captures.plan,
                path_types,
                formation_types,
                field_types,
                accesses: captures.accesses,
                adjusted_accesses: captures.adjusted_accesses,
                nested_accesses: captures.nested_accesses,
                mentions: captures.mentions,
                binding_mutability,
            },
        );
        Ok(execution)
    }
    fn capture_body_fold(&mut self, source: NodeId, solved: &Solved) -> Result<Bound, Error> {
        let mut equations = crate::types::substitute::RegionEquations::default();
        let work = equations
            .extend(solved.region_equalities.iter().copied())
            .map_err(|e| self.error(source, e))?;
        self.facts.body_work.canonical_steps += work;
        let mut fold = Bound::close_projection_regions(1, 0, HashMap::new())
            .map_err(|e| self.error(source, e))?;
        fold.exchange_projection_equations(&mut equations)
            .map_err(|e| self.error(source, e))?;
        Ok(fold)
    }
    fn capture_execution(
        &mut self,
        source: NodeId,
        self_ty: TypeId,
        signature: TypeId,
        receiver: CallKind,
    ) -> Result<Execution, Error> {
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            ..
        }) = self.facts.types.kind(signature)
        else {
            return Err(self.error(source, "missing capture execution signature"));
        };
        let mut universals = self.state.body_binders;
        let receiver_ty = if receiver == CallKind::Once {
            self_ty
        } else {
            let region = RegionId::bound(
                0,
                u16::try_from(universals).map_err(|e| self.error(source, e))?,
            )
            .map_err(|e| self.error(source, e))?;
            universals = universals
                .checked_add(1)
                .ok_or_else(|| self.error(source, "capture receiver lifetime exhausted"))?;
            self.facts
                .types
                .intern(Kind::Reference {
                    mutable: receiver == CallKind::Mutable,
                    region,
                    referent: self_ty,
                })
                .map_err(|e| self.error(source, e))?
        };
        let mut inputs = vec![receiver_ty];
        inputs.extend_from_slice(
            self.facts
                .types
                .list(parameters)
                .ok_or_else(|| self.error(source, "missing capture execution inputs"))?,
        );
        let parameters = self
            .facts
            .types
            .intern_list(&inputs)
            .map_err(|e| self.error(source, e))?;
        let signature = self
            .facts
            .types
            .intern(Kind::Function {
                bound_regions: universals,
                parameters,
                result,
                unsafe_,
                convention,
            })
            .map_err(|e| self.error(source, e))?;
        Ok(Execution {
            self_ty,
            signature,
            universals,
            receiver,
        })
    }
}
