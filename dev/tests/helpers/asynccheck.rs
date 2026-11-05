//! Public deferred-body consumers retain the ordinary Copy, lifetime and loan stages.
use rustgm::{body, borrow, resolve::expansion::Report, sema::model::Declarations, types};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn prepare(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(body::templates::Templates, Vec<body::regions::Contract>)> {
    let mut templates = body::templates::build(report, facts)?;
    body::promote::analyze_templates(report, facts, &mut templates)?;
    let mut inputs = body::regions::collect(facts)?.bodies;
    for template in &templates.bodies {
        if !matches!(
            template.body.owner,
            body::ir::Owner::AsyncBody { .. } | body::ir::Owner::Function { .. }
        ) {
            return Err("source template has another actual owner".into());
        }
        let contract = inputs
            .iter_mut()
            .find(|input| input.key == template.body.key())
            .ok_or("deferred original lost its real input contract")?;
        body::prepare_source_copy(facts, &template.body, contract)?;
        contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    Ok((templates, inputs))
}
pub fn verify<'a>(
    facts: &Declarations,
    templates: &'a body::templates::Templates,
    inputs: Vec<body::regions::Contract>,
) -> Result<borrow::templates::Checked<'a>> {
    let contracts = body::regions::Contracts {
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
        bodies: inputs,
    };
    let mut copies = types::copy::Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut copies,
    };
    Ok(borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        templates,
        &[],
        |ty, member| body::drops::nominal_position(facts, ty, member),
    )?)
}

pub fn future(
    report: &Report,
    facts: &mut Declarations,
    name: &str,
    arguments: &[types::arguments::Argument],
) -> Result<types::TypeId> {
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
        })
        .ok_or("actual async declaration")?;
    let Some(types::Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("actual async constructor signature".into());
    };
    let Some(types::Kind::Opaque(identity, _)) = facts.types.kind(result) else {
        return Err("actual async opaque owner".into());
    };
    let arguments = facts.types.intern_arguments(arguments)?;
    Ok(facts.types.intern(types::Kind::Future {
        identity,
        arguments,
    })?)
}
