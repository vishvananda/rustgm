//! Borrow actual comparison headers without granting bootstrap library factories.
use super::{DefId, Error, LangItem, Report, headers};
use crate::{
    resolve::{model::DefKind, unit::NodeKey},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    support::{
        comparisons::{Comparison, Method},
        constness::Constness,
    },
    types::{
        Kind, Primitive, TypeId,
        arguments::Argument,
        callable::Convention,
        parameters::{ParameterKind, clauses::Clause},
    },
};
enum Failure {
    Header(Error),
    Contract(&'static str),
}
impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Self::Header(error)
    }
}
impl From<&'static str> for Failure {
    fn from(message: &'static str) -> Self {
        Self::Contract(message)
    }
}
pub(in crate::sema) struct SourceComparison {
    pub definition: DefId,
    pub method: DefId,
    pub signature: TypeId,
}
fn role(kind: Comparison) -> Option<LangItem> {
    match kind {
        Comparison::PartialEq => Some(LangItem::PartialEq),
        Comparison::PartialOrd => Some(LangItem::PartialOrd),
        _ => None,
    }
}
/// Classification of an already selected method is not implementation permission.
pub(crate) fn comparison_member(
    report: &Report,
    facts: &Declarations,
    member: DefId,
) -> Option<Method> {
    if let Some(method) = facts.associated.comparison_method(member) {
        return Some(method);
    }
    let catalog = facts.source_bindings.as_deref()?;
    let owner = facts.associated.trait_owner(member)?;
    let definition = report.resolution.definitions.get(member.0 as usize)?;
    if definition.kind != DefKind::Function || definition.node.is_none() {
        return None;
    }
    [Comparison::PartialEq, Comparison::PartialOrd]
        .into_iter()
        .find_map(|kind| {
            let declaration = catalog.language(role(kind)?)?;
            (declaration.definition == owner).then_some(())?;
            kind.methods()
                .iter()
                .copied()
                .find(|method| report.package.names.get(definition.name) == method.name())
        })
}
pub(in crate::sema) fn source_comparison(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    method: Method,
) -> Result<SourceComparison, Error> {
    // A bounded borrowed walk is paid by the existing body-work owner on failure too.
    let charge = facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.language(role(method.owner())?))
        .and_then(|d| facts.associated.owner(d.definition))
        .and_then(|id| facts.associated.owner_members(id))
        .map_or(1, |m| m.len() + 1);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(charge * 16);
    if facts.body_work.canonical_steps > crate::support::crates::MAX_COMPONENTS {
        return Err(Declarations::error(
            report,
            site,
            "source comparison query exceeds its work budget",
        ));
    }
    checked(report, facts, method).map_err(|e| match e {
        Failure::Header(error) => error,
        Failure::Contract(message) => Declarations::error(report, site, message),
    })
}
fn current(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    origin: NodeKey,
) -> Result<(), Failure> {
    let header = facts.header_attribute_contracts(report, definition)?;
    if header
        .origin
        .source()
        .is_none_or(|s| s.declaration != origin)
        || matches!(header.must_use, ContractKnowledge::Unknown)
        || matches!(header.inline, ContractKnowledge::Unknown)
    {
        return Err("source comparison requires its positive current source header".into());
    }
    Ok(())
}
fn checked(
    report: &Report,
    facts: &Declarations,
    selected: Method,
) -> Result<SourceComparison, Failure> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source comparison lacks its catalog")?;
    catalog.require_provider_report(report)?;
    let kind = selected.owner();
    let declaration = catalog
        .language(role(kind).ok_or("unsupported source comparison owner")?)
        .ok_or("source comparison requires its actual language declaration")?;
    current(report, facts, declaration.definition, declaration.origin)?;
    let header = headers::Trait::source(report, facts, declaration)?;
    header.profile(
        &facts.types,
        headers::Flags {
            unsafe_: false,
            auto_: false,
            constant: true,
        },
        1,
    )?;
    let [parameter] = header.parameters else {
        return Err("source comparison requires its Rhs formal".into());
    };
    let Argument::Type(rhs) = parameter.argument else {
        return Err("source comparison Rhs is not a type".into());
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(rhs) else {
        return Err("source comparison Rhs lost its formal identity".into());
    };
    if parameter.anonymous
        || parameter.default != Some(Argument::Type(header.self_type))
        || facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != header.definition.0
                || p.index != 0
                || p.kind != ParameterKind::Type { sized: false }
        })
        || header.written.parameters != 1
        || header.written.supertraits != if kind == Comparison::PartialEq { 1 } else { 2 }
    {
        return Err(
            "source comparison requires its actual PointeeSized Rhs and Self default".into(),
        );
    }
    premises(facts, catalog, &header, rhs, kind)?;
    members(report, facts, &header, rhs, selected)
}
fn members(
    report: &Report,
    facts: &Declarations,
    header: &headers::Trait<'_>,
    rhs: TypeId,
    selected: Method,
) -> Result<SourceComparison, Failure> {
    let kind = selected.owner();
    let expected: &[&str] = if kind == Comparison::PartialEq {
        &["eq", "ne"]
    } else {
        &[
            "partial_cmp",
            "lt",
            "le",
            "gt",
            "ge",
            "__chaining_lt",
            "__chaining_le",
            "__chaining_gt",
            "__chaining_ge",
        ]
    };
    if header.members.len() != expected.len() {
        return Err("source comparison changed its complete member family".into());
    }
    let mut seen = 0u16;
    let mut result = None;
    for &member in header.members {
        let definition = report
            .resolution
            .definitions
            .get(member.0 as usize)
            .ok_or("source comparison member is missing")?;
        let name = report.package.names.get(definition.name);
        let index = expected
            .iter()
            .position(|&n| n == name)
            .ok_or("source comparison has an unsupported member")?;
        if definition.kind != DefKind::Function
            || seen & (1 << index) != 0
            || facts.associated.trait_owner(member) != Some(header.definition)
        {
            return Err("source comparison member belongs to another current owner".into());
        }
        seen |= 1 << index;
        let origin = definition
            .node
            .ok_or("source comparison member has no current source")?;
        current(report, facts, member, origin)?;
        let function = facts
            .declaration_header(member)?
            .ok_or("source comparison member header is pending")?;
        let lifetimes = facts
            .lifetimes
            .functions
            .get(&member)
            .ok_or("source comparison method has no actual lifetime scheme")?;
        if function.declaration.source() != Some(origin)
            || function.universals != 2
            || lifetimes.total != 2
            || lifetimes.captures != 0
            || !lifetimes.early.is_empty()
            || lifetimes.late != [0, 1]
            || !lifetimes.bounds.is_empty()
            || function.body.is_some() != (index != 0)
            || facts
                .generics
                .parameters
                .get(&member)
                .is_some_and(|p| !p.is_empty())
        {
            return Err("source comparison changed its required/provided member contract".into());
        }
        signature(
            facts,
            function.signature,
            header.self_type,
            rhs,
            kind == Comparison::PartialEq || (1..=4).contains(&index),
        )?;
        if name == "partial_cmp" {
            partial_result(facts, function.signature)?;
        }
        if name == selected.name() {
            result = Some((member, function.signature));
        }
    }
    let (method, signature) = result.ok_or("source comparison has no selected actual member")?;
    Ok(SourceComparison {
        definition: header.definition,
        method,
        signature,
    })
}
fn premises(
    facts: &Declarations,
    catalog: &super::Catalog,
    header: &headers::Trait<'_>,
    rhs: TypeId,
    kind: Comparison,
) -> Result<(), &'static str> {
    let pointee = catalog
        .language(LangItem::PointeeSized)
        .ok_or("source comparison lacks actual PointeeSized")?
        .definition;
    let equality = catalog.language(LangItem::PartialEq).map(|d| d.definition);
    let mut found = [false; 3];
    // The canonical producer precedes supertrait elaboration. Its actual inputs
    // describe this declaration, rather than consequences copied from PartialEq.
    for input in header.canonical_inputs(facts)?.iter() {
        let Clause::Predicate(predicate) = input.value else {
            return Err("source comparison has unsupported quantified premises");
        };
        // The defining trait's canonical Self application is an assumption,
        // separate from its authored Rhs and supertrait obligations.
        if facts.generics.trait_self_bounds.get(&header.definition) == Some(&predicate) {
            continue;
        }
        let Some((id, effect)) = predicate.bound.trait_parts() else {
            return Err("source comparison changed its actual trait premises");
        };
        let application = facts
            .types
            .trait_application(id)
            .ok_or("invalid comparison premise")?;
        let mut args = facts
            .types
            .arguments(application.arguments)
            .ok_or("invalid comparison premise arguments")?;
        let index = if application.item.0 == pointee.0
            && effect == Constness::Ordinary
            && args.len() == 0
        {
            if predicate.ty == header.self_type {
                0
            } else if predicate.ty == rhs {
                1
            } else {
                return Err("comparison PointeeSized belongs to another subject");
            }
        } else if kind == Comparison::PartialOrd
            && equality.is_some_and(|d| d.0 == application.item.0)
            && effect == Constness::Conditional
            && predicate.ty == header.self_type
            && args.next() == Some(Argument::Type(rhs))
            && args.next().is_none()
        {
            2
        } else {
            return Err("source comparison changed its genuine supertrait application");
        };
        found[index] = true;
    }
    if !found[0] || !found[1] || (kind == Comparison::PartialOrd && !found[2]) {
        return Err("source comparison is missing its actual PointeeSized or PartialEq premise");
    }
    Ok(())
}
fn signature(
    facts: &Declarations,
    ty: TypeId,
    subject: TypeId,
    rhs: TypeId,
    boolean: bool,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 2,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(ty)
    else {
        return Err("source comparison requires two independent safe shared borrows");
    };
    let Some([left, right]) = facts.types.list(parameters) else {
        return Err("source comparison requires exactly two operands");
    };
    for (input, referent, slot) in [(*left, subject, 0), (*right, rhs, 1)] {
        if !matches!(facts.types.kind(input), Some(Kind::Reference {mutable:false, referent:actual, region})
            if actual == referent && region.bound_parts() == Some((0,slot)))
        {
            return Err("source comparison changed its independent shared operand contract");
        }
    }
    if boolean && facts.types.kind(result) != Some(Kind::Primitive(Primitive::Bool)) {
        return Err("source comparison requires its actual bool result");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/comparison_authority.rs"]
mod tests;

fn partial_result(facts: &Declarations, signature: TypeId) -> Result<(), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("comparison catalog disappeared")?;
    let option = catalog
        .language(LangItem::Option)
        .ok_or("partial_cmp requires actual Option")?;
    let ordering = catalog
        .language(LangItem::Ordering)
        .ok_or("partial_cmp requires actual Ordering")?;
    let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
        return Err("partial_cmp signature is invalid");
    };
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(result) else {
        return Err("partial_cmp result requires actual Option<Ordering>");
    };
    let mut args = facts
        .types
        .arguments(arguments)
        .ok_or("invalid partial_cmp result arguments")?;
    let (Some(Argument::Type(argument)), None) = (args.next(), args.next()) else {
        return Err("partial_cmp result requires one Ordering argument");
    };
    if item.0 != option.definition.0
        || !matches!(facts.types.kind(argument), Some(Kind::Adt(item,args))
        if item.0 == ordering.definition.0 && facts.types.arguments(args).is_some_and(|a|a.len()==0))
    {
        return Err("partial_cmp result changed actual Option/Ordering identity");
    }
    Ok(())
}
