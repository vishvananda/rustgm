//! Anonymous roots retain their own typed view beside the parent's const expression.
use super::{Builder, NodeId, Owner, TypeId};
use crate::{
    sema::{inline_constants::RootFacts, model::ReferenceAdjustment},
    types::unsizing::Conversion,
};
impl Builder<'_> {
    pub(super) fn inline_root(&self, node: NodeId) -> Option<&RootFacts> {
        if !matches!(self.body.owner, Owner::AnonymousConstant) || node != self.body.source.node {
            return None;
        }
        self.facts
            .inline_constants
            .as_deref()?
            .get(self.body.source)?
            .root
            .as_deref()
    }
    pub(super) fn reference_adjustment(&self, node: NodeId) -> Option<ReferenceAdjustment> {
        self.inline_root(node).map_or_else(
            || {
                self.facts
                    .reference_adjustments
                    .get(&super::NodeKey {
                        node,
                        ..self.body.source
                    })
                    .copied()
            },
            |root| root.reference,
        )
    }
    pub(super) fn pointer_adjustment(&self, node: NodeId) -> Option<TypeId> {
        self.inline_root(node).map_or_else(
            || {
                self.facts
                    .pointer_adjustments
                    .get(&super::NodeKey {
                        node,
                        ..self.body.source
                    })
                    .copied()
            },
            |root| root.pointer,
        )
    }
    pub(super) fn unsizing_adjustment(&self, node: NodeId) -> Option<Conversion<TypeId>> {
        self.inline_root(node).map_or_else(
            || {
                self.facts
                    .unsizing_adjustments
                    .get(&super::NodeKey {
                        node,
                        ..self.body.source
                    })
                    .copied()
            },
            |root| root.unsizing,
        )
    }
}

pub(super) fn concrete(
    report: &super::Report,
    facts: &super::Declarations,
    scratch: &mut super::Scratch,
    program: &mut super::Program,
    prepared: &mut std::collections::HashMap<crate::body::ir::BodyKey, super::Body>,
    mut query: Option<&mut crate::types::destruction::NeedsDrop>,
) -> Result<(), super::Error> {
    for (source, ty) in facts.constant_bodies() {
        if facts
            .inline_constants
            .as_deref()
            .and_then(|r| r.get(source))
            .is_some_and(|b| {
                b.evaluation == crate::sema::inline_constants::Evaluation::DeferredTemplate
            })
        {
            continue;
        }
        if let Some(body) = prepared.remove(&crate::body::ir::BodyKey::Anonymous(source)) {
            if body.source != source
                || body.result != ty
                || !matches!(body.owner, Owner::AnonymousConstant)
            {
                return Err(super::Declarations::error(
                    report,
                    source,
                    "retained inline CFG identity or result changed",
                ));
            }
            program.bodies.push(body);
            continue;
        }
        let scope_plan = super::scopes::constant_with_scratch(
            report,
            facts,
            source,
            std::mem::take(&mut scratch.scopes),
        )?;
        let body = super::Body::new(Owner::AnonymousConstant, source, ty);
        program.bodies.push(
            Builder::new(
                report,
                facts,
                scratch,
                body,
                scope_plan,
                query.as_deref_mut(),
            )
            .run(None)
            .map_err(|e| report.diagnostic(e))?,
        );
    }
    Ok(())
}
