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
        for (index, adjustment) in self.adjustments.iter().enumerate() {
            let key = self.key(adjustment.node);
            let target = solved.types[adjustment.expected.0 as usize];
            let actual = solved.types[adjustment.actual.0 as usize];
            let kind = adjustment.kind;
            let (proof_actual, proof_expected) = self
                .discarded_adjustments
                .get(&index)
                .map_or((actual, target), |(a, b)| {
                    (solved.types[a.0 as usize], solved.types[b.0 as usize])
                });
            self.facts.retain_coercion(Coercion {
                body: body_key,
                source: key,
                actual: proof_actual,
                expected: proof_expected,
                kind,
                reference: self
                    .reference_adjustments
                    .get(&index)
                    .map(|(conversion, _)| *conversion),
            });
            if let Some((_, borrow)) = self.reference_adjustments.get(&index) {
                // Runtime reborrows have a fresh region. Keep its relation to
                // the proof destination as well as the source-place conversion.
                // A discarded repeat uses the freshened proof type here, not
                // the annotation's otherwise unobserved destination lifetime.
                self.facts.retain_coercion(Coercion {
                    body: body_key,
                    source: key,
                    actual: solved.types[borrow.0 as usize],
                    expected: proof_expected,
                    kind: AdjustmentKind::Equality,
                    reference: None,
                });
            }
            let facts = self.facts.node_mut(key).ok_or_else(|| {
                crate::sema::model::Declarations::error(self.report, key, "invalid adjustment node")
            })?;
            if kind == AdjustmentKind::Equality && facts.ty == Some(target) && !adjustment.reset {
                continue;
            }
            facts.adjustment = (kind != AdjustmentKind::Equality || facts.ty != Some(target))
                .then_some(Adjustment { kind, target });
            if let Some((conversion, borrow)) = self.reference_adjustments.get(&index) {
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
        let first = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| self.error(node, "body region identity space exhausted"))?;
        let (term, count) = self
            .infer
            .fresh_regions(
                &mut self.facts.types,
                &self.facts.lifetimes.callables,
                term,
                first,
            )
            .map_err(|e| self.error(node, e))?;
        let region = BodyRegion {
            body: self.body,
            source: self.key(node),
            origin: BodyRegionOrigin::Inference,
        };
        self.facts
            .body_regions
            .extend(std::iter::repeat_n(region, count as usize));
        Ok(term)
    }
    pub(super) fn nominal_type(
        &mut self,
        node: NodeId,
        path: NodeId,
        owner: ItemId,
    ) -> Result<TypeId, Error> {
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
            let ast = &self.report.package.unit(self.file).configured.ast;
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
        let Kind::Name(span) = self.kind(node) else {
            return Err(self.error(node, "expected lifetime argument"));
        };
        let name = self
            .report
            .package
            .unit(self.file)
            .spelling(span)
            .map_err(|e| self.error(node, e.message))?;
        match name {
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
        let Some(Constructor::Adt(_, arguments)) = self.head(node, receiver)? else {
            return Err(self.error(node, "missing nominal field receiver"));
        };
        let template = self.facts.members.fields[field.0 as usize].ty;
        let ty = self
            .facts
            .lifetimes
            .substitute(&mut self.facts.types, arguments, template)
            .map_err(|e| self.error(node, e))?;
        self.known(node, ty)
    }
}
