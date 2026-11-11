//! A nominal source goal proves unchanged arguments and the actual final field.
use super::plan::{Failure, Outcome};
use crate::{
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{
        ItemId, TypeId,
        arguments::Argument as Canonical,
        infer::{Context, Error, TermId, arguments::Argument, predicates::Predicate},
    },
};

pub(super) fn requirements(
    facts: &mut Declarations,
    context: &mut Context,
    pair: (TermId, TermId),
    owners: (ItemId, ItemId),
    goals: &mut Vec<Predicate>,
) -> Result<Outcome, Failure> {
    if owners.0 != owners.1 {
        return Ok(Outcome::Absent);
    }
    let selected = parameters(facts, context, owners.0)?;
    if selected.is_empty() {
        return Ok(Outcome::Absent);
    }
    let source = context.item_arguments(&facts.types, pair.0)?;
    let target = context.item_arguments(&facts.types, pair.1)?;
    unchanged_arguments(facts, context, &source, &target, &selected)?;
    let field = facts
        .aggregates
        .get(owners.0)
        .and_then(|definition| facts.aggregates.variants(definition).first())
        .and_then(|variant| facts.types.list(variant.fields))
        .and_then(|fields| fields.last())
        .copied()
        .ok_or("source Unsize lost its completed final field")?;
    let source = tail(facts, context, owners.0, source, field)?;
    let target = tail(facts, context, owners.0, target, field)?;
    let role = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::Unsize))
        .ok_or("nominal source Unsize lost its actual trait")?;
    context.account_work(1)?;
    goals.push(Predicate::trait_application(
        source,
        ItemId(role.definition.0),
        vec![Argument::Type(target)],
    ));
    Ok(Outcome::Ready)
}

fn parameters(
    facts: &mut Declarations,
    context: &mut Context,
    owner: ItemId,
) -> Result<Vec<u32>, Failure> {
    let rules = facts.unsizing_rules.get_or_insert_with(Default::default);
    let before = rules.steps;
    let selected = (|| {
        if let Some(slot) = rules.parameter(&facts.types, &facts.aggregates, owner)? {
            Ok(vec![slot])
        } else {
            rules
                .projected_parameters(&facts.types, &facts.aggregates, owner)
                .map(<[u32]>::to_vec)
        }
    })();
    // Failed declaration walks consume their work too; cache hits stay bounded.
    context.account_work(rules.steps.saturating_sub(before))?;
    let selected = selected?;
    context.account_work(selected.len())?;
    facts.charge_caller_inputs(0, selected.capacity() * size_of::<u32>())?;
    Ok(selected)
}

fn unchanged_arguments(
    facts: &Declarations,
    context: &mut Context,
    source: &[Argument],
    target: &[Argument],
    selected: &[u32],
) -> Result<(), Failure> {
    if source.len() != target.len() {
        return Err(Error::Mismatch.into());
    }
    context.account_work(source.len())?;
    let mut slots = selected.iter().copied().peekable();
    let mut unchanged = Vec::with_capacity(source.len());
    for (index, pair) in source
        .iter()
        .copied()
        .zip(target.iter().copied())
        .enumerate()
    {
        if slots.peek().is_some_and(|&slot| slot as usize == index) {
            slots.next();
            if !matches!(
                pair,
                (Argument::Type(_), Argument::Type(_)) | (Argument::Const(_), Argument::Const(_))
            ) {
                return Err(Error::InvalidIdentity.into());
            }
        } else {
            unchanged.push(pair);
        }
    }
    if slots.next().is_some() {
        return Err(Error::InvalidIdentity.into());
    }
    facts.charge_caller_inputs(0, unchanged.capacity() * size_of::<(Argument, Argument)>())?;
    context.equate_input_pairs(&facts.types, &unchanged)?;
    Ok(())
}

fn tail(
    facts: &mut Declarations,
    context: &mut Context,
    owner: ItemId,
    arguments: Vec<Argument>,
    field: TypeId,
) -> Result<TermId, Failure> {
    let mut fields =
        crate::sema::annotations::Fields::for_arguments(facts, context, owner, arguments)?;
    let pay =
        crate::sema::header_attributes::caller_payments(facts.must_use_contracts.as_deref(), true)
            .map_err(|_| "nominal source Unsize lacks its completed payer")?;
    let result = fields.regions.argument_inside_charged(
        &mut facts.types,
        Canonical::Type(field),
        0,
        &|work, bytes| pay.charge(work, bytes),
    );
    context.account_work(fields.regions.steps)?;
    let Canonical::Type(field) = result? else {
        return Err("nominal source Unsize field changed kind".into());
    };
    Ok(context.instantiate_with(&mut facts.types, field, &mut fields.parameters)?)
}
