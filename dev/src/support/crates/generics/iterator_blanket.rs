//! Version fourteen admits the provider's actual const iterator blanket.
use super::{Constness, HeaderIndex, ImplPolarity, SourceRows, Work};
use crate::support::crates::{generics::source as s, owner_catalog::DeclarationRef};

#[derive(Clone, Copy)]
pub(super) struct Contract {
    pub(super) iterator: DeclarationRef,
    pub(super) meta_sized: DeclarationRef,
    pub(super) into_trait: DeclarationRef,
    pub(super) into_method: DeclarationRef,
}

impl Contract {
    pub(super) fn premise(
        self,
        rows: &SourceRows,
        input: &s::Clause,
    ) -> Option<crate::support::crates::generics::TypeRef> {
        subject(rows, input, self.iterator, Constness::Conditional)
    }

    pub(super) fn check(
        self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        implementation: &s::ImplHeader,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        work.charge(1)?;
        let Some(application) = implementation
            .trait_application
            .and_then(|index| rows.trait_applications.get(index.0 as usize))
        else {
            return Ok(false);
        };
        if !implementation.constant
            || implementation.unsafe_
            || implementation.polarity != ImplPolarity::Positive
            || application.declaration.artifact != rows.policy.provider
            || application.declaration.declaration != self.into_trait
            || application.arguments.start != application.arguments.end
            || application.subject != implementation.self_type
            || implementation.region_captures != 0
            || implementation.region_bounds.start != implementation.region_bounds.end
        {
            return Ok(false);
        }
        let Some(s::Type::Parameter(slot)) = rows.types.get(application.subject.0 as usize) else {
            return Ok(false);
        };
        if headers
            .scopes()
            .complete_slots(implementation.scope, work)?
            != 1
            || !rows.slots.get(slot.0 as usize).is_some_and(|slot| {
                slot.owner == implementation.self_definition
                    && slot.ordinal == 0
                    && slot.index == 0
                    && matches!(slot.kind, s::ParameterKind::Type { .. })
            })
        {
            return Ok(false);
        }
        let inputs = implementation.inputs.slice(&rows.clauses)?;
        work.charge(inputs.len())?;
        let Some([premise, sizing]) = inputs.get(..implementation.declared_inputs as usize) else {
            return Err("source iterator blanket lost its exact declared premises");
        };
        if premise.owner != implementation.self_definition
            || sizing.owner != implementation.self_definition
            || self.premise(rows, premise) != Some(application.subject)
            || subject(rows, sizing, self.meta_sized, Constness::Ordinary)
                != Some(application.subject)
        {
            return Err("source iterator blanket changed its actual Iterator or MetaSized premise");
        }
        self.members(rows, headers, implementation, work)
    }

    fn members(
        self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        implementation: &s::ImplHeader,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        let [method, first, second] = implementation.members.slice(&rows.member_links)? else {
            return Ok(false);
        };
        work.charge(3)?;
        let Some(method_id) = method.implementation else {
            return Ok(false);
        };
        if method.trait_member
            != Some(s::DeclarationAddress {
                artifact: rows.policy.provider,
                declaration: self.into_method,
            })
        {
            return Ok(false);
        }
        let Some(s::HeaderRef::Function(index)) = headers.header(method_id, work)? else {
            return Ok(false);
        };
        let method = &rows.functions[index as usize];
        if method.parent != Some(implementation.self_definition)
            || method.constness != Constness::Const
            || !matches!(method.body, s::FunctionBody::Original(_))
            || method.original.parameters.slice(&rows.roots)? != [implementation.self_type]
            || method.original.result != implementation.self_type
            || first.trait_member == second.trait_member
        {
            return Ok(false);
        }
        for link in [first, second] {
            let (Some(member), Some(result)) = (link.trait_member, link.implementation) else {
                return Ok(false);
            };
            if member.artifact != rows.policy.provider
                || !matches!(headers.header(member.declaration, work)?, Some(s::HeaderRef::AssociatedType(index))
                    if rows.associated_types[index as usize].parent == self.into_trait)
                || !matches!(
                    headers.header(result, work)?,
                    Some(s::HeaderRef::AssociatedDefinition(_))
                )
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn subject(
    rows: &SourceRows,
    input: &s::Clause,
    declaration: DeclarationRef,
    effect: Constness,
) -> Option<crate::support::crates::generics::TypeRef> {
    let s::Requirement::Trait {
        application,
        effect: actual,
    } = input.requirement
    else {
        return None;
    };
    if actual != effect || input.binder.is_some() {
        return None;
    }
    let bound = rows.trait_applications.get(application.0 as usize)?;
    (bound.declaration.artifact == rows.policy.provider
        && bound.declaration.declaration == declaration
        && bound.arguments.start == bound.arguments.end)
        .then_some(bound.subject)
}
