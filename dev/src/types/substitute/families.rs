//! Complete parent/member substitutions shared by semantic and lifetime preparation.
use crate::types::{
    GenericArgs, ItemId, RegionId, Store,
    arguments::Argument,
    parameters::{ParameterId, argument_id},
    substitute::Bound,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Application {
    pub owner: ItemId,
    pub parent: GenericArgs,
    pub member: ItemId,
    pub own: GenericArgs,
}

pub(crate) struct Mappings {
    pub parameters: Vec<(ParameterId, Argument)>,
    pub regions: Vec<RegionId>,
}
impl Mappings {
    pub fn finish(self, store: &Store) -> Result<Bound, &'static str> {
        Bound::parameters_captured(store, &self.parameters, self.regions)
    }
}

pub(crate) fn mappings(
    store: &Store,
    application: Application,
    parameters: (
        impl Iterator<Item = Argument> + Clone,
        impl ExactSizeIterator<Item = Argument>,
    ),
    captures: u32,
) -> Result<Mappings, &'static str> {
    let (parent, own) = parameters;
    let mut parent_arguments = store
        .arguments(application.parent)
        .ok_or("invalid associated parent arguments")?;
    let own_arguments = store
        .arguments(application.own)
        .ok_or("invalid associated family arguments")?;
    let captures = captures as usize;
    let parent_parameters = parent.filter(|p| !matches!(p, Argument::Lifetime(_)));
    if parent_arguments.len() != captures + parent_parameters.clone().count()
        || own_arguments.len() != own.len()
        || parent_arguments.len().saturating_add(own_arguments.len()) > 1_000_000
    {
        return Err("associated family argument arity or storage limit exceeded");
    }
    let mut regions = Vec::with_capacity(captures);
    for _ in 0..captures {
        let Some(Argument::Lifetime(region)) = parent_arguments.next() else {
            return Err("associated parent requires its complete lifetime captures");
        };
        regions.push(region);
    }
    let mut mappings = Vec::new();
    for (formal, actual) in parent_parameters.zip(parent_arguments) {
        let parameter = argument_id(store, formal)?;
        if store
            .parameter(parameter)
            .is_none_or(|p| p.owner.0 != application.owner.0)
        {
            return Err("associated parent parameter belongs to another owner");
        }
        mappings.push((parameter, actual));
    }
    member_mappings(
        store,
        application.member,
        own,
        own_arguments,
        &mut mappings,
        &mut regions,
        |arg| match arg {
            Argument::Lifetime(region) => Some(region),
            Argument::Type(_) | Argument::Const(_) => None,
        },
    )?;
    Ok(Mappings {
        parameters: mappings,
        regions,
    })
}

// Canonical normalization and inferred result shapes use the same declaration
// identities, complete capture prefix, arity and lifetime-slot validation.
pub(crate) fn member_mappings<A: Copy>(
    store: &Store,
    member: ItemId,
    parameters: impl ExactSizeIterator<Item = Argument>,
    arguments: impl ExactSizeIterator<Item = A>,
    mappings: &mut Vec<(ParameterId, A)>,
    regions: &mut Vec<RegionId>,
    lifetime: impl Fn(A) -> Option<RegionId>,
) -> Result<(), &'static str> {
    if parameters.len() != arguments.len() || parameters.len() > 1_000_000 {
        return Err("associated family argument arity or storage limit exceeded");
    }
    for (formal, actual) in parameters.zip(arguments) {
        if let Argument::Lifetime(region) = formal {
            let slot =
                u16::try_from(regions.len()).map_err(|_| "family lifetime capacity exceeded")?;
            if region != RegionId::bound(0, slot)? {
                return Err("associated family lifetime is outside its captured binder");
            }
            regions
                .push(lifetime(actual).ok_or("associated family lifetime argument kind mismatch")?);
        } else {
            let parameter = argument_id(store, formal)?;
            if store
                .parameter(parameter)
                .is_none_or(|p| p.owner.0 != member.0)
            {
                return Err("associated family parameter belongs to another member");
            }
            mappings.push((parameter, actual));
        }
    }
    Ok(())
}
