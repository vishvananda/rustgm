//! Map both method contracts to trait-owned rigid parameters and one lifetime proof scope.
use super::{Clause, Contract, Declarations, DefId, Function, Predicate, RegionId};
use crate::{
    sema::{
        generics::{Parameter, parameter_id},
        lifetimes::FunctionRegions,
    },
    types::{
        Kind, Store,
        arguments::Argument,
        parameters::{
            ParameterId, ParameterKind,
            environment::{Abilities, Environment},
            predicates::Bound,
        },
        substitute,
    },
};

struct Folders {
    expected: substitute::Bound,
    actual: substitute::Bound,
    universals: u32,
    output_regions: Vec<RegionId>,
    variables: Vec<RegionId>,
}
fn scheme(facts: &Declarations, function: Function) -> FunctionRegions {
    facts
        .lifetimes
        .functions
        .get(&function.definition)
        .cloned()
        .unwrap_or(FunctionRegions {
            total: 0,
            captures: 0,
            early: Vec::new(),
            late: Vec::new(),
            has_declared_late: false,
            bounds: Vec::new(),
        })
}
fn own_parameters(facts: &Declarations, owner: DefId) -> Vec<Parameter> {
    facts
        .generics
        .parameters
        .get(&owner)
        .into_iter()
        .flatten()
        .filter(|p| !matches!(p.argument, Argument::Lifetime(_)))
        .copied()
        .collect()
}
fn paired_parameters(
    facts: &Declarations,
    expected: DefId,
    actual: DefId,
) -> Result<Vec<(Parameter, Parameter)>, String> {
    let expected = own_parameters(facts, expected);
    let actual = own_parameters(facts, actual);
    if expected.len() != actual.len() {
        return Err("implementation method generic parameter arity mismatch".into());
    }
    let parameters: Vec<_> = expected.into_iter().zip(actual).collect();
    if parameters.iter().any(|(a, b)| a.anonymous != b.anonymous) {
        return Err("implementation method anonymous parameters disagree with trait".into());
    }
    Ok(parameters)
}
fn universal(index: u32) -> Result<RegionId, &'static str> {
    RegionId::bound(
        0,
        u16::try_from(index).map_err(|_| "method conformance lifetime budget exceeded")?,
    )
}

fn rigid_unused(regions: &mut [RegionId], count: &mut u32) -> Result<(), &'static str> {
    // Captures, early parameters and used late parameters already have nonstatic
    // mappings. An unused executable slot can still occur in a written family
    // input: its formation must hold for every lifetime, not just 'static.
    for region in regions {
        if *region == RegionId::STATIC {
            *region = universal(*count)?;
            *count = count
                .checked_add(1)
                .ok_or("too many method lifetime parameters")?;
        }
    }
    Ok(())
}

fn folders(
    facts: &Declarations,
    predicate: Predicate,
    expected: &FunctionRegions,
    actual: &FunctionRegions,
    method_parameters: &[(Parameter, Parameter)],
) -> Result<Folders, &'static str> {
    let Some((id, _)) = predicate.bound.trait_parts() else {
        return Err("missing impl trait application");
    };
    let app = facts
        .types
        .trait_application(id)
        .ok_or("missing impl trait application")?;
    let trait_owner = DefId(app.item.0);
    let self_type = facts
        .associated
        .schema(trait_owner)
        .ok_or("missing trait schema")?
        .self_type;
    let mut bindings = vec![(
        parameter_id(&facts.types, Argument::Type(self_type))?,
        Argument::Type(predicate.ty),
    )];
    let parameters = facts
        .generics
        .parameters
        .get(&trait_owner)
        .map_or(&[][..], Vec::as_slice);
    let arguments = facts
        .types
        .arguments(app.arguments)
        .ok_or("missing impl trait arguments")?;
    if arguments.len() != parameters.len() || expected.early.len() != actual.early.len() {
        return Err("implementation method early lifetime parameters disagree with trait");
    }
    let mut a = vec![RegionId::STATIC; expected.total as usize];
    let mut b = vec![RegionId::STATIC; actual.total as usize];
    for (formal, value) in parameters.iter().zip(arguments) {
        if let Argument::Lifetime(region) = formal.argument {
            let Some((0, slot)) = region.bound_parts() else {
                return Err("invalid trait lifetime parameter");
            };
            let Argument::Lifetime(value) = value else {
                return Err("trait lifetime argument kind mismatch");
            };
            *a.get_mut(usize::from(slot))
                .ok_or("missing captured trait lifetime")? = value;
        } else {
            bindings.push((parameter_id(&facts.types, formal.argument)?, value));
        }
    }
    for index in 0..actual.captures {
        b[index as usize] = universal(index)?;
    }
    let mut count = actual.captures;
    for ((_, x), (_, y)) in expected.early.iter().zip(&actual.early) {
        a[usize::from(*x)] = universal(count)?;
        b[usize::from(*y)] = universal(count)?;
        count = count
            .checked_add(1)
            .ok_or("too many method lifetime parameters")?;
    }
    for &slot in &expected.late {
        a[usize::from(slot)] = universal(count)?;
        count = count
            .checked_add(1)
            .ok_or("too many method lifetime parameters")?;
    }
    let mut variables = Vec::new();
    for &slot in &actual.late {
        let index =
            u32::try_from(variables.len()).map_err(|_| "too many method late parameters")?;
        let region = RegionId(index + 1);
        b[usize::from(slot)] = region;
        variables.push(region);
    }
    rigid_unused(&mut b, &mut count)?;
    let output_regions = a.clone();
    let expected = substitute::Bound::parameters_captured(&facts.types, &bindings, a)?;
    let replacements = method_parameters
        .iter()
        .map(|(a, b)| Ok((parameter_id(&facts.types, b.argument)?, a.argument)))
        .collect::<Result<Vec<_>, &'static str>>()?;
    let actual = substitute::Bound::parameters_captured(&facts.types, &replacements, b)?;
    Ok(Folders {
        expected,
        actual,
        universals: count,
        output_regions,
        variables,
    })
}

// Normalized shapes can erase inputs whose formation remains an obligation.
fn written_signature(facts: &Declarations, function: Function) -> crate::types::TypeId {
    if facts.associated.families.is_empty() {
        return function.signature;
    }
    match facts
        .unreduced
        .node(function.node)
        .and_then(|ty| facts.types.kind(ty))
    {
        Some(Kind::FunctionItem { signature, .. }) => signature,
        _ => function.signature,
    }
}

pub(super) fn contract(
    facts: &mut Declarations,
    owner: DefId,
    predicate: Predicate,
    expected: Function,
    actual: Function,
) -> Result<Contract, String> {
    let method_parameters = paired_parameters(facts, expected.definition, actual.definition)?;
    let expected_regions = scheme(facts, expected);
    let actual_regions = scheme(facts, actual);
    let mut folders = folders(
        facts,
        predicate,
        &expected_regions,
        &actual_regions,
        &method_parameters,
    )?;
    let expected_signature = folders
        .expected
        .signature(&mut facts.types, expected.signature, 0)?;
    // The written impl signature may already have normalized to a scalar.
    // Its original applications still carry requirements; they cannot acquire
    // the declared signature's implied lifetime permissions by disappearing.
    let actual_template = written_signature(facts, actual);
    let actual_signature = folders
        .actual
        .signature(&mut facts.types, actual_template, 0)?;
    let mut assumptions: Vec<_> = facts
        .generics
        .predicates_in_scope(owner)
        .map(|p| p.value)
        .collect();
    // The impl header already proves the trait's parent requirements. Retain
    // the method's own caller bounds here; synthetic inherited Self assumptions
    // would hide the impl's concrete associated definitions during normalization.
    let method_bounds = facts.generics.predicates.get(&expected.definition);
    for predicate in method_bounds.into_iter().flatten() {
        assumptions.push(folders.expected.clause(&mut facts.types, predicate.value)?);
    }
    let obligations = method_requirements(
        facts,
        actual.definition,
        &mut folders.actual,
        &method_parameters,
    )?;
    let scope: Vec<_> = facts
        .generics
        .parameters_in_scope(owner)
        .copied()
        .chain(method_parameters.iter().map(|(expected, _)| *expected))
        .collect();
    let (environment, parameters) = environment(&facts.types, &scope, &assumptions)?;
    let mut assumed_regions = pairs(
        &mut facts.types,
        &mut folders.expected,
        &expected_regions.bounds,
    )?;
    let required_regions = pairs(
        &mut facts.types,
        &mut folders.actual,
        &actual_regions.bounds,
    )?;
    assumed_regions.extend(
        facts
            .associated
            .schema(owner)
            .ok_or("missing impl schema")?
            .bounds
            .iter()
            .copied(),
    );
    let Some((id, _)) = predicate.bound.trait_parts() else {
        return Err("missing impl trait application".into());
    };
    let app = facts
        .types
        .trait_application(id)
        .ok_or("missing impl application")?;
    let mut inputs = vec![predicate.ty];
    inputs.extend(
        facts
            .types
            .arguments(app.arguments)
            .ok_or("missing impl arguments")?
            .filter_map(|a| {
                if let Argument::Type(ty) = a {
                    Some(ty)
                } else {
                    None
                }
            }),
    );
    Ok(Contract {
        outputs: super::outputs::contracts(facts, expected.definition, &mut folders.expected)?,
        expected: expected_signature,
        declared: expected_signature,
        output_regions: folders.output_regions,
        actual: actual_signature,
        environment: environment.into(),
        parameters,
        assumptions,
        obligations,
        assumed_regions,
        required_regions,
        universals: folders.universals,
        variables: folders.variables,
        inputs,
    })
}
fn method_requirements(
    facts: &mut Declarations,
    method: DefId,
    folder: &mut substitute::Bound,
    method_parameters: &[(Parameter, Parameter)],
) -> Result<Vec<Clause>, &'static str> {
    let mut obligations = Vec::new();
    for predicate in facts.generics.predicates.get(&method).into_iter().flatten() {
        obligations.push(folder.clause(&mut facts.types, predicate.value)?);
    }
    for (_, implementation) in method_parameters {
        if let Argument::Type(ty) = implementation.argument {
            let id = parameter_id(&facts.types, implementation.argument)?;
            if matches!(
                facts.types.parameter(id).map(|p| p.kind),
                Some(ParameterKind::Type { sized: true })
            ) {
                obligations.push(Clause::Predicate(Predicate {
                    ty: folder.apply(&mut facts.types, ty)?,
                    bound: Bound::Sized,
                }));
            }
        }
    }
    Ok(obligations)
}

fn environment(
    store: &Store,
    parameters: &[Parameter],
    predicates: &[Clause],
) -> Result<(Environment, Vec<ParameterId>), &'static str> {
    let mut types = Vec::new();
    let mut constants = Vec::new();
    for p in parameters {
        match p.argument {
            Argument::Type(_) => {
                types.push((parameter_id(store, p.argument)?, Abilities::default()));
            }
            Argument::Const(_) => constants.push(parameter_id(store, p.argument)?),
            Argument::Lifetime(_) => {}
        }
    }
    let environment = Environment::new(store, &types)?
        .with_const_parameters(store, &constants)?
        .with_clauses(store, predicates)?;
    Ok((environment, types.into_iter().map(|(id, _)| id).collect()))
}
fn pairs(
    store: &mut Store,
    folder: &mut substitute::Bound,
    pairs: &[(RegionId, RegionId)],
) -> Result<Vec<(RegionId, RegionId)>, &'static str> {
    pairs
        .iter()
        .map(|&(a, b)| {
            let (Argument::Lifetime(a), Argument::Lifetime(b)) = (
                folder.apply_argument(store, Argument::Lifetime(a))?,
                folder.apply_argument(store, Argument::Lifetime(b))?,
            ) else {
                return Err("invalid method lifetime bound");
            };
            Ok((a, b))
        })
        .collect()
}
