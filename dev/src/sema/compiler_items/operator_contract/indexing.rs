//! Index shape checks borrow the actual Idx, associated owner and parent contract.
use super::{
    Convention, DefId, DefKind, Failure, Method, Mode, Operator, Report, SourceOperator, Work,
    headers, imported_item,
};
use crate::{
    sema::model::Declarations,
    types::{
        ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::{ParameterKind, clauses::Clause},
    },
};

pub(super) fn parameter(
    facts: &Declarations,
    header: &headers::Trait<'_>,
) -> Result<TypeId, Failure> {
    let [parameter] = header.parameters else {
        return Err("source indexing requires its actual Idx formal".into());
    };
    let Argument::Type(ty) = parameter.argument else {
        return Err("source indexing Idx must be an actual type formal".into());
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err("source indexing lost its actual Idx identity".into());
    };
    if parameter.anonymous
        || parameter.default.is_some()
        || facts.types.parameter(id)
            != Some(crate::types::parameters::Parameter {
                owner: ItemId(header.definition.0),
                index: 0,
                kind: ParameterKind::Type { sized: false },
            })
    {
        return Err("source indexing requires actual ?Sized Idx with no default".into());
    }
    Ok(ty)
}
pub(super) fn parent(
    report: &Report,
    facts: &Declarations,
    header: &headers::Trait<'_>,
    operator: Operator,
    idx: Option<TypeId>,
    work: &mut Work,
) -> Result<Option<(DefId, DefId)>, Failure> {
    let inputs = header.canonical_inputs(facts)?;
    work.charge(1)?;
    if !matches!(operator, Operator::IndexMut) {
        return Ok(None);
    }
    let base = match header.origin {
        crate::resolve::body_origin::DeclarationOrigin::Source(_) => {
            super::checked(report, facts, Operator::Index, work)?
        }
        crate::resolve::body_origin::DeclarationOrigin::Imported(_) => {
            let catalog = facts
                .source_bindings
                .as_deref()
                .ok_or("imported indexing parent lacks its actual source catalog")?;
            let role = catalog
                .role(crate::support::bindings::LangItem::Index)?
                .ok_or("imported IndexMut requires its actual Index language declaration")?;
            super::imported_operator(report, facts, role, Operator::Index, work)?
        }
    };
    work.charge(inputs.len())?;
    let exact = inputs.iter().any(|input| {
        let Clause::Predicate(predicate) = input.value else {
            return false;
        };
        predicate.ty == header.self_type
            && predicate.bound.trait_parts().is_some_and(|(id, effect)| {
                effect == crate::support::constness::Constness::Conditional
                    && facts
                        .types
                        .trait_application(id)
                        .is_some_and(|application| {
                            application.item.0 == base.definition.0
                                && facts.types.arguments(application.arguments).is_some_and(
                                    |mut args| {
                                        args.next() == idx.map(Argument::Type)
                                            && args.next().is_none()
                                    },
                                )
                        })
            })
    });
    if !exact {
        return Err("source IndexMut requires its actual [const] Index<Idx> parent".into());
    }
    Ok(Some((
        base.definition,
        base.output
            .ok_or("source indexing parent lost its actual Output")?,
    )))
}
pub(super) fn signature(
    facts: &Declarations,
    header: &headers::Trait<'_>,
    contract: &Method,
    parameters: crate::types::ListId,
    result: TypeId,
    bound_regions: u32,
    work: &mut Work,
) -> Result<(), Failure> {
    let Mode::Index { mutable } = contract.mode else {
        return Err("source indexing lost its checked signature mode".into());
    };
    let Some([receiver, argument]) = facts.types.list(parameters) else {
        return Err("source indexing requires its actual receiver and Idx".into());
    };
    let Some(Kind::Reference {
        mutable: input_mutable,
        region,
        referent,
    }) = facts.types.kind(*receiver)
    else {
        return Err("source indexing requires its actual borrowed receiver".into());
    };
    let Some(Kind::Reference {
        mutable: output_mutable,
        region: output_region,
        referent: target,
    }) = facts.types.kind(result)
    else {
        return Err("source indexing requires its actual reference result".into());
    };
    if bound_regions != 1
        || input_mutable != mutable
        || output_mutable != mutable
        || region != output_region
        || region.bound_parts() != Some((0, 0))
        || referent != header.self_type
        || Some(*argument) != contract.rhs
    {
        return Err("source indexing changed its exact receiver, Idx or result lifetime".into());
    }
    let output = contract
        .output
        .ok_or("source indexing lacks its actual Output declaration")?;
    let declared = facts
        .associated
        .types
        .get(&output)
        .ok_or("source indexing Output lacks its completed contract")?;
    work.charge(declared.bounds.len().saturating_add(1))?;
    if facts.associated.trait_owner(output)!=Some(contract.projection_owner) || facts.types.projection_schema(ItemId(output.0)).is_none_or(|schema|schema.trait_item.0!=contract.projection_owner.0 || schema.sized) || declared.bounds.iter().any(|clause|matches!(*clause,Clause::Predicate(predicate) if predicate.ty==declared.projection && predicate.bound==crate::types::parameters::predicates::Bound::Sized)){return Err("source indexing requires its actual ?Sized Output owner and schema".into());}
    let Some(Kind::Projection(projection)) = facts.types.kind(target) else {
        return Err("source indexing result lost its actual Output projection".into());
    };
    if projection.item.0 != output.0
        || projection.subject != header.self_type
        || projection.arguments != crate::types::GenericArgs::EMPTY
        || facts
            .types
            .trait_application(projection.trait_application)
            .is_none_or(|application| {
                application.item.0 != contract.projection_owner.0
                    || facts
                        .types
                        .arguments(application.arguments)
                        .is_none_or(|mut args| {
                            args.next() != contract.rhs.map(Argument::Type) || args.next().is_some()
                        })
            })
    {
        return Err("source indexing result changed its actual Output application".into());
    }
    Ok(())
}

pub(super) fn target_policy(
    report: &Report,
    facts: &Declarations,
    method: DefId,
    unsafe_: bool,
) -> Result<(), Failure> {
    use crate::sema::target_features::{self, Policy};
    match target_features::query(report, facts, method)? {
        Policy::Local(None) => Ok(()),
        Policy::Local(Some(policy)) if policy.declared_unsafe() && unsafe_ => Ok(()),
        _ => Err("source indexing target-feature permission profile is not implemented".into()),
    }
}

fn imported_members(
    report: &Report,
    facts: &Declarations,
    header: &headers::Trait<'_>,
    operator: Operator,
    work: &mut Work,
) -> Result<(Option<DefId>, DefId), Failure> {
    let own_output = operator.own_output();
    if header.written.parameters != 1
        || header.written.supertraits != usize::from(matches!(operator, Operator::IndexMut))
        || header.members.len() != if own_output { 2 } else { 1 }
    {
        return Err("imported indexing operator changed its actual member profile".into());
    }
    let (mut output, mut method) = (None, None);
    for &member in header.members {
        work.charge(3)?;
        let definition = report
            .resolution
            .definitions
            .get(member.0 as usize)
            .ok_or("imported operator member has no actual definition")?;
        if facts.associated.trait_owner(member) != Some(header.definition) {
            return Err("imported operator member belongs to another actual owner".into());
        }
        match (definition.kind, report.package.names.get(definition.name)) {
            (DefKind::TypeAlias, "Output") if own_output && output.is_none() => {
                output = Some(member);
            }
            (DefKind::Function, name) if name == operator.method() && method.is_none() => {
                method = Some(member);
            }
            _ => return Err("imported indexing operator changed its actual member profile".into()),
        }
    }
    let method = method.ok_or("imported indexing operator omitted its operation")?;
    Ok((output, method))
}

pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    header: &headers::Trait<'_>,
    operator: Operator,
    work: &mut Work,
) -> Result<SourceOperator, Failure> {
    work.charge(8)?;
    header.profile(
        &facts.types,
        headers::Flags {
            unsafe_: false,
            auto_: false,
            constant: header.flags.constant,
        },
        1,
    )?;
    let rhs = parameter(facts, header)?;
    let own_output = operator.own_output();
    let (output, method) = imported_members(report, facts, header, operator, work)?;
    let inherited = parent(report, facts, header, operator, Some(rhs), work)?;
    let output = output
        .or(inherited.map(|(_, output)| output))
        .ok_or("imported indexing operator omitted its Output")?;
    let function = facts
        .declaration_header(method)?
        .ok_or("imported operator has no completed method header")?;
    if function.body.is_some()
        || function.universals != 1
        || (own_output && facts.associated.families.contains_key(&output))
        || facts
            .generics
            .parameters
            .get(&method)
            .is_some_and(|p| !p.is_empty())
    {
        return Err("imported indexing operator requires its required nongeneric profile".into());
    }
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("imported indexing operator requires a safe Rust signature".into());
    };
    let contract = Method {
        definition: method,
        output: Some(output),
        projection_owner: inherited.map_or(header.definition, |(owner, _)| owner),
        rhs: Some(rhs),
        mode: operator.mode(),
    };
    signature(
        facts,
        header,
        &contract,
        parameters,
        result,
        bound_regions,
        work,
    )?;
    imported_item(facts, method, function.signature)?;
    Ok(SourceOperator {
        definition: header.definition,
        method,
        signature: function.signature,
        rhs: Some(rhs),
        output: Some(output),
    })
}
