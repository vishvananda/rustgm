//! Compare family inputs in one impl lifetime scope, using the assumption side's rigid IDs.
use crate::{
    resolve::model::DefId,
    sema::{
        generics::{Parameter, parameter_id},
        model::Declarations,
    },
    types::{RegionId, arguments::Argument, parameters::predicates::Predicate, substitute::Bound},
};

pub(super) struct Mapping {
    pub expected: Bound,
    pub actual: Bound,
    pub parameters: Vec<Parameter>,
    pub total: u32,
    pub captures: u32,
}

pub(super) fn build(
    facts: &mut Declarations,
    owner: DefId,
    predicate: Predicate,
    expected: DefId,
    actual: DefId,
    declared: bool,
) -> Result<Mapping, &'static str> {
    let a = facts
        .generics
        .parameters
        .get(&expected)
        .map_or(&[][..], Vec::as_slice);
    let b = facts
        .generics
        .parameters
        .get(&actual)
        .map_or(&[][..], Vec::as_slice);
    if a.len() != b.len() {
        return Err("associated family generic parameter arity mismatch");
    }
    let captures = facts
        .associated
        .schema(owner)
        .ok_or("missing impl schema")?
        .captures;
    let mut regions = (0..captures)
        .map(universal)
        .collect::<Result<Vec<_>, _>>()?;
    let mut own = Vec::with_capacity(a.len());
    let mut bindings = Vec::new();
    let mut parameters = Vec::with_capacity(a.len());
    for (&a, &b) in a.iter().zip(b) {
        if !matches!(
            (a.argument, b.argument),
            (Argument::Lifetime(_), Argument::Lifetime(_))
                | (Argument::Type(_), Argument::Type(_))
                | (Argument::Const(_), Argument::Const(_))
        ) {
            return Err("associated family generic parameter kind mismatch");
        }
        if let (Argument::Const(a), Argument::Const(b)) = (a.argument, b.argument)
            && facts.types.const_type(a) != facts.types.const_type(b)
        {
            return Err("associated family const parameter type mismatch");
        }
        let mut selected = if declared { a } else { b };
        if let (Argument::Lifetime(_), Argument::Lifetime(_)) = (a.argument, b.argument) {
            let index =
                u32::try_from(regions.len()).map_err(|_| "family lifetime capacity exceeded")?;
            let region = universal(index)?;
            regions.push(region);
            selected.argument = Argument::Lifetime(region);
        } else {
            // Bound validates type/const kinds and const parameter types. The
            // chosen rigid IDs carry only this side's implicit Sized promises.
            bindings.push((parameter_id(&facts.types, b.argument)?, selected.argument));
        }
        own.push(selected.argument);
        parameters.push(selected);
    }
    let total = u32::try_from(regions.len()).map_err(|_| "family lifetime capacity exceeded")?;
    let actual = Bound::parameters_captured(&facts.types, &bindings, regions)?;
    let own = facts.types.intern_arguments(&own)?;
    let expected = facts.trait_substitution(predicate, Some((expected, own)))?;
    Ok(Mapping {
        expected,
        actual,
        parameters,
        total,
        captures,
    })
}

fn universal(index: u32) -> Result<RegionId, &'static str> {
    RegionId::bound(
        0,
        u16::try_from(index).map_err(|_| "family lifetime capacity exceeded")?,
    )
}
