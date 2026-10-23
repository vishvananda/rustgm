//! Instantiate lifetime-only aggregate uses and selected fields from their typed templates.
use super::{Checker, Constructor, DefId, Error, Kind, NodeId, Target, TermId, TypeId};
use crate::{
    sema::{
        members::FieldId,
        model::{
            Adjustment, AdjustmentKind, BodyRegion, BodyRegionOrigin, Coercion, ReferenceAdjustment,
        },
    },
    types::{ItemId, RegionId},
};

impl Checker<'_> {
    pub(super) fn publish_adjustments(
        &mut self,
        solved: &crate::types::infer::Solved,
        body_key: crate::resolve::unit::NodeKey,
    ) -> Result<(), Error> {
        let mut defining = (!solved.defining_opaques.is_empty())
            .then(|| {
                crate::types::substitute::Bound::defining_results(
                    &self.facts.types,
                    solved.defining_opaques.clone(),
                )
            })
            .transpose()
            .map_err(|e| self.error(body_key.node, e))?;
        self.publish_unsizing_relations(solved, body_key, defining.as_mut())?;
        self.publish_pointer_adjustments(solved, body_key);
        for (index, adjustment) in self.state.adjustments.iter().enumerate() {
            let key = self.key(adjustment.node);
            let target = solved.types[adjustment.expected.0 as usize];
            let actual = solved.types[adjustment.actual.0 as usize];
            let kind = adjustment.kind;
            // A coercion can create a distinct nominal lifetime instance with
            // no expression node of its own. Its runtime value needs fields too.
            if target != actual && self.facts.generics.has_nominal_requirements {
                self.facts
                    .aggregates
                    .prepare(&mut self.facts.types, target)
                    .map_err(|e| self.error(adjustment.node, e))?;
            }
            let (mut proof_actual, mut proof_expected) = self
                .state
                .discarded_adjustments
                .get(&index)
                .map_or((actual, target), |(a, b)| {
                    (solved.types[a.0 as usize], solved.types[b.0 as usize])
                });
            if !self.state.discarded_adjustments.contains_key(&index)
                && let Some(view) = self
                    .state
                    .pointer_adjustments
                    .get(&index)
                    .map(|pending| pending.view)
                    .or_else(|| {
                        self.state
                            .unsizing_adjustments
                            .get(&index)
                            .map(|pending| pending.view)
                    })
                    .or_else(|| self.state.callable_adjustments.get(&index).copied())
            {
                proof_actual = solved.types[view.0 as usize];
            }
            if let Some(fold) = &mut defining {
                proof_actual = fold
                    .apply(&mut self.facts.types, proof_actual)
                    .map_err(|e| self.error(adjustment.node, e))?;
                proof_expected = fold
                    .apply(&mut self.facts.types, proof_expected)
                    .map_err(|e| self.error(adjustment.node, e))?;
            }
            self.facts.retain_coercion(Coercion {
                body: body_key,
                source: key,
                actual: proof_actual,
                expected: proof_expected,
                kind,
                reference: self
                    .state
                    .reference_adjustments
                    .get(&index)
                    .filter(|_| !self.state.pointer_adjustments.contains_key(&index))
                    .map(|(conversion, _)| *conversion),
            });
            if !self.state.pointer_adjustments.contains_key(&index)
                && let Some((_, borrow)) = self.state.reference_adjustments.get(&index)
            {
                // Runtime reborrows have a fresh region. Keep its relation to
                // the proof destination as well as the source-place conversion.
                // A discarded repeat uses the freshened proof type here, not
                // the annotation's otherwise unobserved destination lifetime.
                self.facts.retain_coercion(Coercion {
                    body: body_key,
                    source: key,
                    actual: solved.types[self
                        .state
                        .unsizing_adjustments
                        .get(&index)
                        .and_then(|pending| pending.borrow_view)
                        .unwrap_or(*borrow)
                        .0 as usize],
                    expected: proof_expected,
                    kind: AdjustmentKind::Equality,
                    reference: None,
                });
            }
        }
        self.publish_runtime_adjustments(solved)
    }
    /// Runtime selections remain distinct from the source-side lifetime proof views.
    fn publish_runtime_adjustments(
        &mut self,
        solved: &crate::types::infer::Solved,
    ) -> Result<(), Error> {
        for (index, adjustment) in self.state.adjustments.iter().enumerate() {
            let key = self.key(adjustment.node);
            let target = solved.types[adjustment.expected.0 as usize];
            let kind = adjustment.kind;
            let facts = self.facts.node_mut(key).ok_or_else(|| {
                crate::sema::model::Declarations::error(self.report, key, "invalid adjustment node")
            })?;
            if kind == AdjustmentKind::Equality && facts.ty == Some(target) && !adjustment.reset {
                continue;
            }
            facts.adjustment = (kind != AdjustmentKind::Equality || facts.ty != Some(target))
                .then_some(Adjustment { kind, target });
            if let Some(pending) = self.state.pointer_adjustments.get(&index) {
                self.facts
                    .pointer_adjustments
                    .insert(key, solved.types[pending.intermediate.0 as usize]);
            } else {
                self.facts.pointer_adjustments.remove(&key);
            }
            if let Some(pending) = self.state.unsizing_adjustments.get(&index) {
                self.facts.unsizing_adjustments.insert(
                    key,
                    pending.conversion.try_map(|term| {
                        solved.types.get(term.0 as usize).copied().ok_or_else(|| {
                            crate::sema::model::Declarations::error(
                                self.report,
                                key,
                                "missing unsizing proof type",
                            )
                        })
                    })?,
                );
            } else {
                self.facts.unsizing_adjustments.remove(&key);
            }
            if let Some((conversion, borrow)) = self.state.reference_adjustments.get(&index) {
                self.facts.reference_adjustments.insert(
                    key,
                    ReferenceAdjustment {
                        conversion: *conversion,
                        borrow: solved.types[borrow.0 as usize],
                    },
                );
            } else {
                self.facts.reference_adjustments.remove(&key);
            }
        }
        Ok(())
    }

    pub(super) fn inferred_regions(&mut self, node: NodeId, term: TermId) -> Result<TermId, Error> {
        let first = self
            .facts
            .body_regions
            .frontier(self.state.body)
            .map_err(|_| self.error(node, "body region identity space exhausted"))?;
        let fresh = if self.facts.aggregates.has_projected_templates() {
            let facts = &mut *self.facts;
            self.state.infer.fresh_regions_with(
                &mut facts.types,
                &facts.lifetimes.callables,
                term,
                first,
                &mut |types, item| {
                    facts
                        .lifetimes
                        .rigid_inputs
                        .get_or_insert_with(Default::default)
                        .demand(
                            types,
                            &facts.aggregates,
                            item,
                            facts.definitions.get(item.0 as usize).copied().flatten(),
                        )
                        .map_err(|_| crate::types::infer::Error::InvalidIdentity)
                },
            )
        } else {
            self.state.infer.fresh_regions(
                &mut self.facts.types,
                &self.facts.lifetimes.callables,
                term,
                first,
            )
        }
        .map_err(|e| self.error(node, e))?;
        let region = BodyRegion {
            body: self.state.body,
            source: self.key(node),
            origin: BodyRegionOrigin::Inference,
        };
        self.facts
            .body_regions
            .allocate_reserved(region, first, fresh.regions as usize)
            .map_err(|e| self.error(node, e))?;
        if fresh.needs_normalization {
            self.defer_normalization(
                node,
                crate::types::lifetime_proof::ScopeId::ROOT,
                fresh.term,
            )?;
        }
        Ok(fresh.term)
    }
    pub(super) fn nominal_type(
        &mut self,
        node: NodeId,
        path: NodeId,
        owner: ItemId,
    ) -> Result<TypeId, Error> {
        if let Some(&use_) = self.facts.generics.constructor_uses.get(&self.key(path)) {
            return match use_ {
                crate::sema::generics::ConstructorUse::SelfType(ty) => Ok(ty),
                crate::sema::generics::ConstructorUse::Prefix(prefix) => self
                    .facts
                    .node(prefix)
                    .and_then(|f| f.ty)
                    .ok_or_else(|| self.error(node, "missing alias constructor type")),
            };
        }
        let template = self.facts.definitions[owner.0 as usize]
            .ok_or_else(|| self.error(node, "missing nominal template"))?;
        let count = self
            .facts
            .lifetimes
            .parameters
            .get(&DefId(owner.0))
            .map_or(0, Vec::len);
        let list = self.nominal_arguments(path, owner)?;
        if count == 0 && list.is_none() {
            return Ok(template);
        }
        let mut arguments = Vec::with_capacity(count);
        if let Some(list) = list {
            let ast = &self.report.package.unit(self.state.file).configured.ast;
            let nodes = ast.list(list);
            if nodes.len() != count {
                return Err(self.error(node, "incorrect number of lifetime arguments"));
            }
            for node in nodes {
                arguments.push(self.local_region(*node)?);
            }
        } else {
            for _ in 0..count {
                arguments.push(self.body_region(node, BodyRegionOrigin::ItemArgument)?);
            }
        }
        let args = self
            .facts
            .types
            .intern_regions(&arguments)
            .map_err(|e| self.error(node, e))?;
        self.facts
            .lifetimes
            .substitute(&mut self.facts.types, args, template)
            .map_err(|e| self.error(node, e))
    }

    pub(super) fn nominal_arguments(
        &self,
        path: NodeId,
        owner: ItemId,
    ) -> Result<Option<crate::syntax::ast::List>, Error> {
        let enum_variant = self.report.resolution.definitions[owner.0 as usize].kind
            == crate::resolve::model::DefKind::Enum;
        crate::sema::lifetimes::arguments(self.report, self.key(path), enum_variant)
    }

    pub(super) fn local_region(&mut self, node: NodeId) -> Result<RegionId, Error> {
        let Kind::Name(span, _) = self.kind(node) else {
            return Err(self.error(node, "expected lifetime argument"));
        };
        let name = self
            .report
            .package
            .unit(self.state.file)
            .spelling(span)
            .map_err(|e| self.error(node, e.message))?;
        match name {
            // A one-name type path is also a `Name`; only a quoted spelling is a lifetime.
            _ if !name.starts_with('\'') => Err(self.error(node, "expected lifetime argument")),
            "'static" => Ok(RegionId::STATIC),
            "'_" => self.body_region(node, BodyRegionOrigin::ItemArgument),
            _ => {
                let Some(Target::Definition(id)) =
                    self.facts.node(self.key(node)).and_then(|f| f.reference)
                else {
                    return Err(self.error(node, "missing lifetime argument resolution"));
                };
                self.facts
                    .lifetimes
                    .named
                    .get(&id)
                    .copied()
                    .ok_or_else(|| self.error(node, "lifetime argument is outside this body"))
            }
        }
    }

    pub(super) fn instance_field(
        &mut self,
        node: NodeId,
        receiver: TermId,
        field: FieldId,
    ) -> Result<TermId, Error> {
        let template = self.facts.members.fields[field.0 as usize].ty;
        self.state.call_types.demand_fields(receiver);
        let ty = match self.head(node, receiver)? {
            Some(Constructor::Adt(_, arguments)) => self
                .facts
                .lifetimes
                .substitute(&mut self.facts.types, arguments, template)
                .map_err(|e| self.error(node, e))?,
            Some(Constructor::AdtArgs(owner, _)) => {
                return self.inferred_field(node, receiver, owner, template);
            }
            _ => return Err(self.error(node, "missing nominal field receiver")),
        };
        self.known(node, ty)
    }
}
