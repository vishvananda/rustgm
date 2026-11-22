//! Selected captures build one typed environment and retain both lifetime views.
use super::{CallKind, Checker, Constructor, Entry, Error, NodeId, TypeId};
use crate::support::hash::HashMap;
use crate::{
    resolve::unit::NodeKey,
    sema::{
        closures::captures::Mode,
        closures::{Environment, Execution},
        model::BodyRegionOrigin,
    },
    types::{Kind, RegionId, infer::Solved, substitute::Bound},
};

impl Checker<'_> {
    /// A closure's reads retain creator premises without an independent body
    /// trait goal, as rustc's closures share their creator's where-clauses.
    /// Instances discard this child-owned alias context and reprove their own
    /// reads.
    pub(super) fn retain_closure_copy_context(
        &mut self,
        body: crate::sema::closures::Body,
    ) -> Result<(), Error> {
        let node = body.source.node;
        if self.facts.source_bindings.is_none() {
            return Ok(());
        }
        let Some(owner) = body.owner else {
            return Ok(());
        };
        let Some(environment) = self.caller_declaration_environment(node, owner)? else {
            return Ok(());
        };
        let inherited = self
            .facts
            .lifetimes
            .functions
            .get(&owner)
            .map_or(0, |scheme| scheme.total);
        if environment.body_aliases().is_some()
            || (environment.requires_region_mapping() && body.inherited < inherited)
        {
            return Err(self.error(node, "closure Copy premises have a foreign lifetime scope"));
        }
        self.prepare_universals(node)?;
        let mut aliases = self.state.call_types.universal_arguments();
        // A captured execution borrowing its receiver opens one more universal,
        // the receiver's lifetime, after the body's own. No creator premise can
        // name it; its alias only completes the execution's slot vector.
        if let Some(execution) = body.execution
            && execution.universals != body.universals
        {
            if execution.universals != body.universals.saturating_add(1)
                || aliases.len() != body.universals as usize
                || execution.receiver == CallKind::Once
            {
                return Err(self.error(node, "closure execution changed its creator's universals"));
            }
            aliases.push(self.body_region(node, BodyRegionOrigin::UniversalAlias)?);
        }
        let growth = environment
            .body_alias_finish_bound(aliases.len())
            .map_err(|e| self.error(node, e))?;
        self.pay_closure_qualification(node, growth.work, growth.bytes)?;
        let environment = environment
            .as_ref()
            .clone()
            .with_body_aliases(aliases)
            .map_err(|e| self.error(node, e))?;
        self.prepare_source_copy_context(node, Some(std::sync::Arc::new(environment)))
    }

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
        let expected = metadata.expected_kind.unwrap_or(CallKind::Shared);
        if metadata.bindings.is_empty() {
            return Ok(expected);
        }
        let mut captures =
            self.closure_uses(source, metadata.root.node, &metadata.bindings, children)?;
        // A contextual FnOnce receiver owns its captures. Never grant a stronger
        // call permission than the body's checked uses allow.
        captures.kind = captures.kind.max(expected);
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
        let (_, parent) = scheduler
            .stack
            .last_mut()
            .ok_or_else(|| self.error(source, "missing capture formation parent"))?;
        let mut formation = metadata
            .lifetimes
            .closing_in_parent(&self.state.call_types, &parent.call_types)
            .map_err(|e| self.error(source, e))?;
        let mut closing = metadata
            .lifetimes
            .closing(&self.state.call_types, false)
            .map_err(|e| self.error(source, e))?;
        let formation_types = super::bindings::parent_paths(
            self.facts,
            &self.report.resolution.definitions,
            parent,
            &captures,
            solved,
            &mut formation,
            &mut closing,
        )
        .map_err(|e| self.error(source, e))?;
        let path_types = captures
            .types
            .iter()
            .map(|term| body_fold.apply(&mut self.facts.types, solved.types[term.0 as usize]))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| self.error(source, e))?;
        let field_types = metadata
            .fields
            .iter()
            .map(|term| {
                body_fold
                    .apply(&mut self.facts.types, solved.types[term.0 as usize])
                    .map_err(|e| self.error(source, e))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let formation_fields = retain_formations(
            self.facts,
            parent.body,
            crate::resolve::unit::NodeKey {
                node: source,
                ..parent.body
            },
            &captures.plan,
            &formation_types,
            &field_types,
            &mut closing,
        )
        .map_err(|e| self.error(source, e))?;
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
        let binding_mutability = self.capture_binding_mutability(source, &metadata.bindings)?;
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
                formation_fields,
                accesses: captures.accesses,
                adjusted_accesses: captures.adjusted_accesses,
                nested_accesses: captures.nested_accesses,
                mentions: captures.mentions,
                binding_mutability,
            },
        );
        Ok(execution)
    }
    fn capture_binding_mutability(
        &self,
        source: NodeId,
        bindings: &[(crate::resolve::model::DefId, super::TermId)],
    ) -> Result<HashMap<crate::resolve::model::DefId, bool>, Error> {
        let mut binding_mutability = HashMap::default();
        for &(definition, _) in bindings {
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
        Ok(binding_mutability)
    }
    fn capture_body_fold(&mut self, source: NodeId, solved: &Solved) -> Result<Bound, Error> {
        let mut equations = crate::types::substitute::RegionEquations::default();
        let work = equations
            .extend(solved.region_equalities.iter().copied())
            .map_err(|e| self.error(source, e))?;
        // A captured slot comes from the creator. A use in this closure may
        // require it to equal another region, including static, but the
        // execution receiver still stores the creator's original input.
        let mut anchors = HashMap::default();
        for (slot, &alias) in self.state.call_types.universal_aliases().iter().enumerate() {
            let slot = u16::try_from(slot)
                .map_err(|_| self.error(source, "too many capture lifetimes"))?;
            let slot = RegionId::bound(0, slot).map_err(|e| self.error(source, e))?;
            anchors.insert(slot, slot);
            anchors.insert(alias, slot);
        }
        self.facts.body_work.canonical_steps += work + anchors.len();
        let mut fold =
            Bound::close_projection_regions(1, 0, anchors).map_err(|e| self.error(source, e))?;
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

/// Captured values use the same checked lifetime conversion as an ordinary move.
/// The parent storage and exported field may use distinct universal aliases.
fn retain_formations(
    facts: &mut crate::sema::model::Declarations,
    body: NodeKey,
    source: NodeKey,
    plan: &crate::sema::closures::captures::Plan,
    places: &[TypeId],
    fields: &[TypeId],
    closing: &mut Bound,
) -> Result<Vec<TypeId>, &'static str> {
    let mut values = Vec::with_capacity(fields.len());
    for (capture, &field) in plan.captures().iter().zip(fields) {
        let place = *places
            .get(capture.path.index())
            .ok_or("missing formation place")?;
        let expected = closing.apply(&mut facts.types, field)?;
        let actual = if capture.mode == Mode::Value {
            place
        } else {
            let Some(Kind::Reference {
                region, mutable, ..
            }) = facts.types.kind(expected)
            else {
                return Err("borrowed capture field is not a reference");
            };
            if mutable != (capture.mode != Mode::Shared) {
                return Err("borrowed capture field has different permission");
            }
            facts.types.intern(Kind::Reference {
                region,
                mutable,
                referent: place,
            })?
        };
        values.push(actual);
        if actual != expected {
            facts.retain_coercion(crate::sema::model::Coercion {
                body,
                source,
                actual,
                expected,
                kind: crate::sema::model::AdjustmentKind::Equality,
                reference: None,
            });
        }
    }
    facts.body_work.canonical_steps += closing.steps;
    Ok(values)
}
