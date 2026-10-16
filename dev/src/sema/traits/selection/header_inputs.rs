//! Match independent parameter headers without copying the input inference graph.
use super::{Context, Declarations, DefId};
use crate::types::{
    Kind, RegionId,
    infer::{Constructor, Error, TermId, predicates::Predicate},
    parameters::{ParameterId, predicates::Bound},
};

pub(super) struct Inputs {
    pub parameter: ParameterId,
    pub value: TermId,
    pub regions: Vec<RegionId>,
}

/// Exact known pointer/reference heads and one independent parameter cannot
/// bind, narrow or merge source holes. Repeated captures must agree exactly;
/// more general lifetime equations retain the ordinary isolated probe.
pub(super) fn independent(
    facts: &Declarations,
    context: &mut Context,
    goal: &Predicate,
    owner: DefId,
) -> Result<Option<Inputs>, Error> {
    let (_, arguments) = goal.trait_arguments().ok_or(Error::InvalidIdentity)?;
    if !arguments.is_empty() {
        return Ok(None);
    }
    let header = facts
        .generics
        .implementations
        .as_ref()
        .and_then(|registry| registry.header(owner))
        .ok_or(Error::InvalidIdentity)?
        .predicate;
    let Bound::Trait(id) = header.bound else {
        return Err(Error::InvalidIdentity);
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or(Error::InvalidIdentity)?;
    if facts
        .types
        .arguments(application.arguments)
        .ok_or(Error::InvalidIdentity)?
        .len()
        != 0
    {
        return Ok(None);
    }
    let parameters = facts
        .generics
        .parameter_ids(&facts.types, owner)
        .map_err(|_| Error::InvalidIdentity)?;
    context.account_work(parameters.len())?;
    let [parameter] = parameters.as_slice() else {
        return Ok(None);
    };
    let captures = facts
        .associated
        .schema(owner)
        .ok_or(Error::InvalidIdentity)?
        .captures;
    context.account_work(captures as usize)?;
    let mut regions = vec![None; captures as usize];
    let (mut pattern, mut actual) = (header.ty, goal.subject());
    loop {
        context.account_work(1)?;
        let child = match facts.types.kind(pattern).ok_or(Error::InvalidIdentity)? {
            Kind::Parameter(id) if id == *parameter => {
                return Ok(regions
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .map(|regions| Inputs {
                        parameter: id,
                        value: actual,
                        regions,
                    }));
            }
            Kind::Reference {
                region,
                mutable,
                referent,
            } => {
                let Some(Constructor::Reference {
                    region: found,
                    mutable: found_mutable,
                }) = head(&facts.types, context, actual)?
                else {
                    return Ok(None);
                };
                if mutable != found_mutable || !capture(region, found, &mut regions)? {
                    return Ok(None);
                }
                referent
            }
            Kind::Pointer { mutable, pointee } => {
                let Some(Constructor::Pointer { mutable: found }) =
                    head(&facts.types, context, actual)?
                else {
                    return Ok(None);
                };
                if mutable != found {
                    return Ok(None);
                }
                pointee
            }
            _ => return Ok(None),
        };
        pattern = child;
        actual = context.child(&facts.types, actual, 0)?;
    }
}

fn capture(
    pattern: RegionId,
    actual: RegionId,
    regions: &mut [Option<RegionId>],
) -> Result<bool, Error> {
    if pattern == RegionId::STATIC {
        return Ok(actual == pattern);
    }
    let Some((0, index)) = pattern.bound_parts() else {
        return Err(Error::InvalidIdentity);
    };
    let target = regions
        .get_mut(index as usize)
        .ok_or(Error::InvalidIdentity)?;
    if let Some(previous) = *target {
        Ok(previous == actual)
    } else {
        *target = Some(actual);
        Ok(true)
    }
}

fn head(
    store: &crate::types::Store,
    context: &mut Context,
    term: TermId,
) -> Result<Option<Constructor>, Error> {
    match context.head(store, term) {
        Err(Error::Normalization) => Ok(None),
        result => result,
    }
}
