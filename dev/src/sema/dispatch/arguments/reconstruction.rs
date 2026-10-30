//! A call exporter pays structural reconstruction without acquiring a second proof.
use super::{
    Argument, Arguments, Declarations, DefId, GenericArgs, ItemId, Kind, RegionId, TypeId,
};
use crate::{
    support::retained::{Growth, Payment},
    types::{parameters::ParameterId, substitute::Bound},
};

impl Arguments {
    pub(in crate::sema::dispatch) fn requested_paid(
        &self,
        facts: &mut Declarations,
        member: DefId,
        goal: crate::types::parameters::predicates::Predicate,
        pay: Payment<'_>,
    ) -> Result<TypeId, &'static str> {
        pay(8, 0)?;
        let owner = facts
            .associated
            .trait_owner(member)
            .ok_or("requested method has no actual trait owner")?;
        let (application, _) = goal
            .bound
            .trait_parts()
            .ok_or("requested method has no complete trait application")?;
        let application = facts
            .types
            .trait_application(application)
            .ok_or("requested method lost its actual trait application")?;
        let schema = facts
            .associated
            .schema(owner)
            .ok_or("requested trait has no Self schema")?;
        if application.item != ItemId(owner.0) || schema.captures != 0 {
            return Err(
                "requested method changed its trait or requires inherited lifetime mapping",
            );
        }
        let self_type = schema.self_type;
        let template = facts
            .lifetimes
            .callables
            .get(ItemId(member.0))
            .ok_or("requested method has no complete callable template")?;
        let parameters = facts
            .lifetimes
            .callables
            .generic_parameters(ItemId(member.0))
            .ok_or("requested method lost its complete formal order")?;
        if template.parameters as usize != self.regions.len() {
            return Err("requested method changed its complete early lifetime arity");
        }
        let count = self.regions.len().saturating_add(parameters.len());
        pay(
            count.saturating_add(self.bindings.len()),
            Growth::vector::<Argument>(0, 0, count).bytes,
        )?;
        let own = self
            .bindings
            .iter()
            .filter(|(id, _)| {
                facts
                    .types
                    .parameter(*id)
                    .is_some_and(|p| p.owner == ItemId(self.method.0))
            })
            .count();
        let offset = self.bindings.len().saturating_sub(own);
        let mut arguments = Vec::with_capacity(count);
        arguments.extend(self.regions.iter().copied().map(Argument::Lifetime));
        for &id in parameters {
            pay(8, 0)?;
            let parameter = facts
                .types
                .parameter(id)
                .ok_or("requested method has a foreign formal")?;
            arguments.push(
                if facts.types.kind(self_type) == Some(Kind::Parameter(id)) {
                    Argument::Type(goal.ty)
                } else if parameter.owner == ItemId(owner.0) {
                    trait_argument(facts, owner, id, parameter.index, application.arguments)?
                } else if parameter.owner == ItemId(member.0) {
                    let (actual, value) = *self
                        .bindings
                        .get(offset.saturating_add(parameter.index as usize))
                        .ok_or("selected method lost its own formal suffix")?;
                    let actual = facts
                        .types
                        .parameter(actual)
                        .ok_or("selected method lost its own formal")?;
                    if actual.owner != ItemId(self.method.0)
                        || actual.index != parameter.index
                        || actual.kind != parameter.kind
                    {
                        return Err("selected and requested own method formals do not correspond");
                    }
                    value
                } else {
                    return Err("requested method has a foreign inherited formal owner");
                },
            );
        }
        instantiate_paid(facts, member, &arguments, pay)
    }
    pub(in crate::sema::dispatch) fn new_paid(
        facts: &Declarations,
        callable: TypeId,
        pay: Payment<'_>,
    ) -> Result<Self, &'static str> {
        pay(1, 0)?;
        let Some(Kind::FunctionItem {
            item, arguments, ..
        }) = facts.types.kind(callable)
        else {
            return Err("selected callable is not a function item");
        };
        let count = facts
            .types
            .arguments(arguments)
            .ok_or("selected callable inputs are absent")?
            .len();
        let Some(template) = facts.lifetimes.callables.get(item) else {
            if count != 0
                || facts
                    .lifetimes
                    .functions
                    .get(&DefId(item.0))
                    .is_some_and(|scheme| scheme.captures != 0 || !scheme.early.is_empty())
            {
                return Err("selected callable lacks its complete captured template");
            }
            return Ok(Self {
                method: DefId(item.0),
                regions: Vec::new(),
                bindings: Vec::new(),
            });
        };
        let early = template.parameters as usize;
        let formals = facts
            .lifetimes
            .callables
            .generic_parameters(item)
            .ok_or("selected callable formals are absent")?
            .len();
        let mut growth = Growth::vector::<Argument>(0, 0, count);
        growth.include(Growth::vector::<RegionId>(0, 0, early));
        growth.include(Growth::vector::<(ParameterId, Argument)>(0, 0, formals));
        pay(
            count.saturating_add(early).saturating_add(formals),
            growth.bytes,
        )?;
        Self::new(facts, callable)
    }
    pub(in crate::sema::dispatch) fn substitution_paid(
        &self,
        facts: &Declarations,
        pay: Payment<'_>,
    ) -> Result<Bound, &'static str> {
        let growth = Growth::vector::<RegionId>(0, 0, self.regions.len());
        pay(self.regions.len(), growth.bytes)?;
        Bound::parameters_captured_charged(
            &facts.types,
            &self.bindings,
            self.regions.clone(),
            &|s, b| pay(s, b),
        )
    }
    pub(in crate::sema::dispatch) fn target_paid(
        &self,
        facts: &mut Declarations,
        implementation: DefId,
        selected: GenericArgs,
        method: DefId,
        pay: Payment<'_>,
    ) -> Result<TypeId, &'static str> {
        let count = facts
            .types
            .arguments(selected)
            .ok_or("selected implementation inputs are absent")?
            .len();
        // target_arguments copies the implementation prefix and appends each
        // actual own method argument; prepay both selected vectors first.
        let bound = count
            .saturating_add(self.regions.len())
            .saturating_add(self.bindings.len());
        let mut growth = Growth::vector::<Argument>(0, 0, count);
        growth.include(Growth::vector::<Argument>(0, 0, bound));
        // Include each source visit plus geometric relocation/copy work while
        // the selected prefix and actual method suffix are assembled.
        pay(bound.saturating_mul(4).saturating_add(count), growth.bytes)?;
        let arguments = self.target_arguments(facts, implementation, selected)?;
        instantiate_paid(facts, method, &arguments, pay)
    }
}
fn trait_argument(
    facts: &Declarations,
    owner: DefId,
    id: ParameterId,
    index: u32,
    actuals: GenericArgs,
) -> Result<Argument, &'static str> {
    let formal = facts
        .generics
        .parameters
        .get(&owner)
        .and_then(|parameters| parameters.get(index as usize))
        .ok_or("requested trait lost its actual written formal")?;
    if !matches!(formal.argument, Argument::Type(ty) if facts.types.kind(ty) == Some(Kind::Parameter(id)))
    {
        return Err("requested trait changed its actual formal identity or kind");
    }
    facts
        .types
        .arguments(actuals)
        .and_then(|mut args| args.nth(index as usize))
        .ok_or("requested trait lost a complete actual argument")
}
fn instantiate_paid(
    facts: &mut Declarations,
    method: DefId,
    arguments: &[Argument],
    pay: Payment<'_>,
) -> Result<TypeId, &'static str> {
    pay(arguments.len().saturating_add(1), 0)?;
    let item = ItemId(method.0);
    let signature = if let Some(template) = facts.lifetimes.callables.get(item) {
        let early = template.parameters as usize;
        let parameters = facts
            .lifetimes
            .callables
            .generic_parameters(item)
            .ok_or("selected method formals are absent")?;
        if arguments.len() != early.saturating_add(parameters.len()) {
            return Err("selected method argument arity changed");
        }
        let mut growth = Growth::vector::<RegionId>(0, 0, early);
        growth.include(Growth::vector::<(ParameterId, Argument)>(
            0,
            0,
            parameters.len(),
        ));
        pay(arguments.len(), growth.bytes)?;
        let regions = arguments[..early]
            .iter()
            .map(|argument| match argument {
                Argument::Lifetime(region) => Ok(*region),
                _ => Err("selected method early argument is not a lifetime"),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bindings: Vec<_> = parameters
            .iter()
            .copied()
            .zip(arguments[early..].iter().copied())
            .collect();
        let signature = template.signature;
        let mut fold =
            Bound::parameters_captured_charged(&facts.types, &bindings, regions, &|s, b| {
                pay(s, b)
            })?;
        let Argument::Type(signature) =
            fold.apply_argument_charged(&mut facts.types, Argument::Type(signature), &|s, b| {
                pay(s, b)
            })?
        else {
            return Err("selected signature changed its actual type argument kind");
        };
        signature
    } else {
        if !arguments.is_empty() {
            return Err("selected method lacks its complete callable template");
        }
        ordinary_paid(facts, method, pay)?
    };
    let arguments = facts
        .types
        .intern_arguments_observed::<true>(arguments, &|s, b| pay(s, b))?;
    facts.types.intern_observed::<true>(
        Kind::FunctionItem {
            item,
            arguments,
            signature,
        },
        &|s, b| pay(s, b),
    )
}
fn ordinary_paid(
    facts: &mut Declarations,
    method: DefId,
    pay: Payment<'_>,
) -> Result<TypeId, &'static str> {
    let ty = facts
        .definitions
        .get(method.0 as usize)
        .copied()
        .flatten()
        .ok_or("selected method definition is absent")?;
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(ty)
    else {
        return Err("selected method definition is not callable");
    };
    if item != ItemId(method.0) {
        return Err("selected method declaration identity changed");
    }
    let Some(scheme) = facts.lifetimes.functions.get(&method) else {
        return Ok(signature);
    };
    if scheme.captures != 0 || !scheme.early.is_empty() {
        return Err("selected method requires its captured template");
    }
    let growth = Growth::vector::<RegionId>(0, 0, scheme.total as usize);
    pay(
        (scheme.total as usize).saturating_add(scheme.late.len()),
        growth.bytes,
    )?;
    let mut regions = vec![RegionId::STATIC; scheme.total as usize];
    for (index, &slot) in scheme.late.iter().enumerate() {
        let region = regions
            .get_mut(usize::from(slot))
            .ok_or("selected late lifetime slot changed")?;
        *region = RegionId::bound(
            0,
            u16::try_from(index).map_err(|_| "selected method has too many late lifetimes")?,
        )?;
    }
    let late = u32::try_from(scheme.late.len())
        .map_err(|_| "selected method has too many late lifetimes")?;
    Bound::new(regions).signature_charged(&mut facts.types, signature, late, &|s, b| pay(s, b))
}
