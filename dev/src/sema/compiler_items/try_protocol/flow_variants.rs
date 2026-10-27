//! Branching consumes two real single-field variants of one canonical enum.
use super::{DefId, Error, ItemId, LangItem, Query, TypeId, VariantId};
use crate::{
    sema::header_attributes,
    syntax::item::Shape,
    types::{Kind, arguments::Argument},
};

impl Query<'_> {
    pub(super) fn variants(
        &self,
        result: TypeId,
        residual: DefId,
        output: DefId,
        subject: TypeId,
    ) -> Result<(ItemId, VariantId, VariantId), Error> {
        header_attributes::charge_source_contract_query(self.facts, 8)?;
        let Some(Kind::Adt(owner, arguments)) = self.facts.types.kind(result) else {
            return Err(self.error("source Try branch must return its actual control-flow enum"));
        };
        let mut arguments = self
            .facts
            .types
            .arguments(arguments)
            .ok_or_else(|| self.error("source Try branch lost its actual enum arguments"))?;
        if arguments.next() != Some(Argument::Type(self.projection(residual)?))
            || arguments.next() != Some(Argument::Type(self.projection(output)?))
            || arguments.next().is_some()
        {
            return Err(
                self.error("source Try branch changed its actual residual/output arguments")
            );
        }
        let parameters = self
            .facts
            .generics
            .parameters
            .get(&DefId(owner.0))
            .ok_or_else(|| self.error("source control-flow enum lost its actual formals"))?;
        let [break_type, continue_type] = parameters.as_slice() else {
            return Err(self.error("source control-flow enum requires its actual two type formals"));
        };
        let (Argument::Type(break_type), Argument::Type(continue_type)) =
            (break_type.argument, continue_type.argument)
        else {
            return Err(self.error("source control-flow enum has non-type formals"));
        };
        let continue_ = self.variant(LangItem::ControlFlowContinue, owner, continue_type)?;
        let break_ = self.variant(LangItem::ControlFlowBreak, owner, break_type)?;
        if continue_ == break_
            || self.facts.members.variants(owner).len() != 2
            || self.projection_subject(output)? != subject
            || self.projection_subject(residual)? != subject
        {
            return Err(self.error("source control-flow enum or Try projection identities changed"));
        }
        Ok((owner, continue_, break_))
    }
    fn projection_subject(&self, definition: DefId) -> Result<TypeId, Error> {
        let Some(Kind::Projection(projection)) =
            self.facts.types.kind(self.projection(definition)?)
        else {
            return Err(self.error("source Try associated type has no genuine projection"));
        };
        Ok(projection.subject)
    }
    fn variant(
        &self,
        role: LangItem,
        owner: ItemId,
        field_type: TypeId,
    ) -> Result<VariantId, Error> {
        let declaration = self.claimed(role)?;
        header_attributes::require_control_flow_variant(self.facts, self.report, declaration)?;
        let id = self
            .facts
            .members
            .constructor(declaration.definition)
            .ok_or_else(|| self.error("source control-flow role lost its actual constructor"))?;
        let variant = self
            .facts
            .members
            .variants
            .get(id.0 as usize)
            .ok_or_else(|| self.error("source control-flow variant identity is stale"))?;
        let Some([field]) = self.facts.members.fields(id) else {
            return Err(self.error("source control-flow variant requires its actual single field"));
        };
        if variant.owner != owner
            || variant.shape != Shape::Tuple
            || variant.ordinal >= 2
            || variant.origin.source() != Some(declaration.origin)
            || field.ty != field_type
            || self
                .facts
                .members
                .variants(owner)
                .get(variant.ordinal as usize)
                != Some(&id)
        {
            return Err(self
                .error("source control-flow variant changed its actual owner, ordinal or field"));
        }
        Ok(id)
    }
}
