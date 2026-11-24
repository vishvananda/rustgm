//! Reprove the provided methods admitted for the actual Iterator role.
use super::{
    Catalog, Declaration, Declarations, DefId, DefKind, ItemId, Kind, LangItem, Report,
    reductions::Reduction,
};
use crate::{
    sema::{
        compiler_items::headers::Trait,
        declaration_inputs::{self, Inputs},
    },
    support::constness::Constness,
    types::{
        GenericArgs, Primitive, TypeId,
        arguments::Argument,
        callable::Convention,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

pub(in crate::sema) fn source_default(
    report: &Report,
    facts: &Declarations,
    member: DefId,
    owner: DefId,
) -> Result<Option<Declaration>, &'static str> {
    let Some(method) = Method::of(report, member) else {
        return Ok(None);
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source Iterator default lost its actual provider catalog")?;
    let Some(iterator) = catalog
        .language(LangItem::Iterator)
        .filter(|role| role.definition == owner)
    else {
        return Ok(None);
    };
    let option = catalog
        .language(LangItem::Option)
        .ok_or("source Iterator defaults require their actual Option declaration")?;
    catalog.require_provider_report(report)?;
    let mut steps = method.header_work();
    let checked = super::loop_headers::trait_header(report, facts, iterator.into(), &mut steps)
        .and_then(|header| {
            method.require(facts, catalog, &header, ItemId(option.definition.0), member)
        });
    crate::sema::header_attributes::charge_source_contract_query(facts, steps)
        .map_err(|_| "source Iterator default exceeds its declaration work budget")?;
    checked?;
    Ok(Some(iterator))
}

pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    iterator: &Trait<'_>,
    option: ItemId,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(iterator.members.len());
    for &member in iterator.members {
        if let Some(method) = Method::of(report, member) {
            *steps = steps.saturating_add(method.header_work());
            method.require(facts, catalog, iterator, option, member)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Method {
    SizeHint,
    ByRef,
    ByValue(Reduction),
}
impl Method {
    const fn header_work(self) -> usize {
        match self {
            Self::SizeHint | Self::ByRef => 16,
            Self::ByValue(Reduction::Fold) => 64,
            Self::ByValue(Reduction::Last | Reduction::Count) => 36,
        }
    }

    fn of(report: &Report, member: DefId) -> Option<Self> {
        let definition = report.resolution.definitions.get(member.0 as usize)?;
        if definition.kind != DefKind::Function {
            return None;
        }
        let [size_hint, by_ref, fold, last, count] = LangItem::PROVIDED_ITERATOR_METHODS;
        Some(match report.package.names.get(definition.name) {
            name if name == size_hint => Self::SizeHint,
            name if name == by_ref => Self::ByRef,
            name if name == fold => Self::ByValue(Reduction::Fold),
            name if name == last => Self::ByValue(Reduction::Last),
            name if name == count => Self::ByValue(Reduction::Count),
            _ => return None,
        })
    }

    fn require(
        self,
        facts: &Declarations,
        catalog: &Catalog,
        iterator: &Trait<'_>,
        option: ItemId,
        member: DefId,
    ) -> Result<(), &'static str> {
        let function = facts
            .declaration_header(member)?
            .ok_or("source Iterator default has no actual completed header")?;
        let regions = u32::from(!matches!(self, Self::ByValue(_)));
        if function.body.is_none()
            || facts.associated.trait_owner(member) != Some(iterator.definition)
            || function.universals != regions
            || regions != 0
                && facts
                    .generics
                    .parameters
                    .get(&member)
                    .is_some_and(|own| !own.is_empty())
            || function.declaration.source().is_some() && facts.nonconst_trait_member(member)?
        {
            return Err(
                "source Iterator default changed its actual provided conditional declaration",
            );
        }
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            convention: Convention::RUST,
            bound_regions,
        }) = facts.types.kind(function.signature)
        else {
            return Err("source Iterator default requires its actual safe Rust signature");
        };
        if bound_regions != regions {
            return Err("source Iterator default changed its actual lifetime binder");
        }
        let inputs = facts
            .types
            .list(parameters)
            .ok_or("source Iterator default lacks its actual input list")?;
        if let Self::ByValue(reduction) = self {
            return reduction.require(facts, catalog, iterator, option, member, (inputs, result));
        }
        let [receiver] = inputs else {
            return Err("source Iterator default requires one actual input");
        };
        let expected_mutable = matches!(self, Self::ByRef);
        if !matches!(facts.types.kind(*receiver), Some(Kind::Reference { referent, region, mutable })
            if referent == iterator.self_type && mutable == expected_mutable && region.bound_parts() == Some((0, 0)))
        {
            return Err(match self {
                Self::SizeHint => {
                    "source size_hint requires its actual late-bound shared Self input"
                }
                Self::ByRef => "source by_ref requires its actual late-bound mutable Self input",
                Self::ByValue(_) => "source Iterator reduction has an unexpected reference input",
            });
        }
        let inputs =
            declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), member)?;
        match self {
            Self::SizeHint => {
                if inputs.len() != 0 {
                    return Err("source size_hint requires its actual empty written premises");
                }
                size_hint_result(facts, option, result)
            }
            Self::ByRef => {
                if *receiver != result {
                    return Err(
                        "source by_ref requires its actual receiver lifetime in the result",
                    );
                }
                by_ref_sized(facts, catalog, iterator.self_type, inputs)
            }
            Self::ByValue(_) => {
                Err("source Iterator reduction did not validate its by-value header")
            }
        }
    }
}

fn by_ref_sized(
    facts: &Declarations,
    catalog: &Catalog,
    self_type: TypeId,
    inputs: Inputs<'_>,
) -> Result<(), &'static str> {
    const SIZED: &str = "source by_ref requires its actual ordinary Self: Sized premise";
    let sized = catalog.validated_role(LangItem::Sized)?.ok_or(SIZED)?;
    let Some(declaration_inputs::Input {
        value: Clause::Predicate(predicate),
        ..
    }) = inputs.get(0).filter(|_| inputs.len() == 1)
    else {
        return Err(SIZED);
    };
    if predicate.ty != self_type {
        return Err(SIZED);
    }
    if predicate.bound == Bound::Sized {
        return Ok(());
    }
    let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
        return Err(SIZED);
    };
    if !facts
        .types
        .trait_application(application)
        .is_some_and(|application| {
            application.item == ItemId(sized.definition.0)
                && application.arguments == GenericArgs::EMPTY
        })
    {
        return Err(SIZED);
    }
    Ok(())
}

fn size_hint_result(
    facts: &Declarations,
    option: ItemId,
    result: TypeId,
) -> Result<(), &'static str> {
    let usize_ty = TypeId::primitive(Primitive::Usize);
    let Some([lower, upper]) = facts.types.kind(result).and_then(|kind| match kind {
        Kind::Tuple(fields) => facts.types.list(fields),
        _ => None,
    }) else {
        return Err("source size_hint requires its actual two-field result");
    };
    if *lower != usize_ty
        || !matches!(facts.types.kind(*upper), Some(Kind::Adt(owner, args))
            if owner == option && facts.types.arguments(args).is_some_and(|arguments| arguments.eq([Argument::Type(usize_ty)])))
    {
        return Err("source size_hint requires its actual (usize, Option<usize>) result");
    }
    Ok(())
}
