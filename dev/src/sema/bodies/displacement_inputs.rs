//! Offset inference selects one real member without value reads or autodereferencing.
use super::{Checker, Constructor, DefKind, Error, NodeId, TermId};
use crate::{
    sema::{displacement_plans, members::FieldId},
    types::infer::arguments::Argument,
};

#[derive(Clone, Copy)]
pub(super) struct OpenPlan {
    pub node: NodeId,
    pub container: TermId,
    pub field: FieldId,
    pub field_ty: TermId,
    pub call: super::try_control::OpenCall,
}
#[derive(Default)]
pub(super) struct Pending {
    pub rows: Vec<OpenPlan>,
}
impl Checker<'_> {
    pub(super) fn offset_readiness(
        &mut self,
        node: NodeId,
    ) -> Result<Option<super::Requirement>, Error> {
        let crate::syntax::ast::Kind::OffsetOf { container, .. } = self.kind(node) else {
            return Ok(None);
        };
        crate::resolve::require_offset(self.report, self.key(node))?;
        let receiver = self.offset_container(node, container)?;
        if let Some(Constructor::Adt(owner, _) | Constructor::AdtArgs(owner, _)) =
            self.head(node, receiver)?
            && self.facts.aggregates.get(owner).is_none()
        {
            return Ok(Some(super::Requirement::Nominal(owner)));
        }
        let target = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(crate::support::bindings::LangItem::OffsetOf))
            .ok_or_else(|| {
                self.error(
                    node,
                    "offset requires its actual source intrinsic declaration",
                )
            })?
            .definition;
        if !self.facts.function_indices.contains_key(&target) {
            return Ok(Some(super::Requirement::Definition(target)));
        }
        Ok(None)
    }
    pub(super) fn offset_of(
        &mut self,
        node: NodeId,
        container: NodeId,
        fields: NodeId,
    ) -> Result<TermId, Error> {
        crate::resolve::require_offset(self.report, self.key(node))?;
        let receiver = self.offset_container(node, container)?;
        let Some(Constructor::Adt(owner, _) | Constructor::AdtArgs(owner, _)) =
            self.head(node, receiver)?
        else {
            return Err(self.error(node, "offset container must be nominal without autoderef"));
        };
        let parts =
            displacement_plans::components(self.report, self.facts, self.key(node), fields)?;
        let variant = if self.report.resolution.definitions[owner.0 as usize].kind == DefKind::Enum
        {
            if parts[0].0 == parts[1].0 {
                return Err(self.error(node, "enum offset requires a variant and field"));
            }
            displacement_plans::variant(
                self.report,
                self.facts,
                self.key(node),
                owner,
                parts[1],
                self.state.module,
            )?
        } else {
            if parts[0].0 != parts[1].0 {
                return Err(self.error(node, "nested offset fields are not implemented"));
            }
            let [variant] = self.facts.members.variants(owner) else {
                return Err(self.error(node, "offset container has no unique storage variant"));
            };
            *variant
        };
        let name = self.offset_member(node, parts[0].1)?;
        let field = self.select_field(node, variant, name, parts[0].1, "unknown offset field")?;
        let field_ty = self.instance_field(node, receiver, field)?;
        let target = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(crate::support::bindings::LangItem::OffsetOf))
            .ok_or_else(|| {
                self.error(
                    node,
                    "offset requires its actual source intrinsic declaration",
                )
            })?
            .definition;
        let callable = self.implicit_source_function(node, target)?;
        let arguments = self
            .state
            .infer
            .item_arguments(&self.facts.types, callable)
            .map_err(|e| self.error(node, e))?;
        let [Argument::Type(subject)] = arguments.as_slice() else {
            return Err(self.error(node, "offset intrinsic changed its actual type formal"));
        };
        self.shape_equation(node, *subject, receiver)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        let result = self.child(node, signature, 2)?;
        self.retain_offset(OpenPlan {
            node,
            container: receiver,
            field,
            field_ty,
            call: super::try_control::OpenCall {
                callable,
                signature,
            },
        })?;
        self.state.calls.push((node, signature));
        Ok(result)
    }
    fn offset_container(&mut self, node: NodeId, container: NodeId) -> Result<TermId, Error> {
        if self
            .facts
            .node(self.key(container))
            .and_then(|fact| fact.ty)
            .is_none()
        {
            return Err(self.error(
                node,
                "offset container requires its completed source annotation",
            ));
        }
        self.annotation(container)
    }
    fn offset_member(
        &mut self,
        node: NodeId,
        span: crate::syntax::token_tree::Span,
    ) -> Result<crate::sema::members::Name, Error> {
        let spelling = self
            .report
            .package
            .unit(self.state.file)
            .spelling(span)
            .map_err(|e| self.error(node, e.message))?;
        crate::sema::header_attributes::charge_source_contract_query(
            self.facts,
            spelling.len().saturating_add(1),
        )?;
        if spelling.as_bytes().first().is_some_and(u8::is_ascii_digit) {
            return self.member_name(node, span);
        }
        let mut names = std::mem::take(&mut self.facts.members.names);
        let selected = names.intern_paid(spelling, &mut |growth| {
            crate::sema::header_attributes::charge_source_retention(
                self.facts,
                growth.work,
                growth.bytes,
            )
            .map_err(|_| "offset member name budget exhausted")
        });
        self.facts.members.names = names;
        selected
            .map(crate::sema::members::Name::Named)
            .map_err(|e| self.error(node, e))
    }
    fn retain_offset(&mut self, plan: OpenPlan) -> Result<(), Error> {
        let rows = self.state.call_types.offsets.as_deref();
        let (length, capacity) =
            rows.map_or((0, 0), |rows| (rows.rows.len(), rows.rows.capacity()));
        let growth = crate::support::retained::Growth::vector::<OpenPlan>(length, capacity, 1);
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            growth.work.saturating_add(1),
            growth
                .bytes
                .saturating_add(usize::from(rows.is_none()) * size_of::<Vec<OpenPlan>>()),
        )?;
        let rows = self
            .state
            .call_types
            .offsets
            .get_or_insert_with(Default::default);
        let needed = if length == capacity {
            length.saturating_mul(2).max(4) - length
        } else {
            0
        };
        rows.rows.try_reserve_exact(needed).map_err(|_| {
            crate::sema::model::Declarations::error(
                self.report,
                self.state.body,
                "offset staging allocation failed",
            )
        })?;
        if rows
            .rows
            .capacity()
            .saturating_sub(capacity)
            .saturating_mul(size_of::<OpenPlan>())
            > growth.bytes
        {
            return Err(crate::sema::model::Declarations::error(
                self.report,
                self.state.body,
                "offset staging allocation exceeds prepaid storage",
            ));
        }
        rows.rows.push(plan);
        Ok(())
    }
}
