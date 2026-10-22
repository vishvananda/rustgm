//! Reference methods forward the same canonical member through shared reborrows.
use super::{
    Body, Bound, Contract, Declarations, Kind, LocalId, Place, Rvalue, Selected, TypeId, Validated,
    ValueId, dereference, value,
};
use crate::types::{RegionId, arguments::Argument, callable::Convention};

pub(super) fn contract(
    facts: &mut Declarations,
    selected: Selected,
    source: &Validated,
) -> Result<(Contract, [TypeId; 2]), &'static str> {
    let inputs = selected
        .reference_inputs(facts)?
        .ok_or("missing reference inputs")?;
    let parameters = [
        facts.types.intern(Kind::Reference {
            referent: inputs.operands[0],
            mutable: false,
            region: RegionId::bound(0, 0)?,
        })?,
        facts.types.intern(Kind::Reference {
            referent: inputs.operands[1],
            mutable: false,
            region: RegionId::bound(0, 1)?,
        })?,
    ];
    let universals = inputs
        .assumptions
        .as_ref()
        .map_or(inputs.universals, |a| a.universals);
    let list = facts.types.intern_list(&parameters)?;
    let signature = facts.types.intern(Kind::Function {
        parameters: list,
        result: source.result,
        unsafe_: false,
        convention: Convention::RUST,
        bound_regions: universals,
    })?;
    let mut input = super::super::regions::callable(
        facts,
        source.definition,
        source.definition,
        source.source,
        signature,
    )?;
    input.parameters.clear();
    if let Some(assumptions) = inputs.assumptions {
        input.assumptions.extend(assumptions.regions);
        input.type_assumptions.extend(assumptions.types);
    }
    Ok((input, parameters))
}

pub(super) fn operation(
    body: &mut Body,
    facts: &mut Declarations,
    source: &Validated,
    parameters: [TypeId; 2],
) -> Result<ValueId, &'static str> {
    let mut arguments = Vec::new();
    let mut places = Vec::new();
    for (index, parameter) in parameters.into_iter().enumerate() {
        let (place, referent) = operand(body, facts, index, parameter)?;
        arguments.push(Argument::Type(referent));
        places.push(place);
    }
    let (callable, opened) = callable(facts, source, arguments)?;
    let Some(Kind::Function {
        parameters: call_parameters,
        result,
        ..
    }) = facts.types.kind(opened)
    else {
        return Err("reference method is not callable");
    };
    let parameters = facts
        .types
        .list(call_parameters)
        .ok_or("missing reborrow parameters")?
        .to_vec();
    if parameters.len() != 2 || result != source.result {
        return Err("reference method forwarding signature differs");
    }
    for (place, ty) in places.into_iter().zip(parameters) {
        let borrowed = value(
            body,
            ty,
            Rvalue::Borrow {
                place,
                kind: super::super::ir::BorrowKind::Immediate,
            },
        )?;
        body.arguments.push(borrowed);
    }
    let callee = value(body, callable, Rvalue::Function(source.definition))?;
    value(
        body,
        result,
        Rvalue::Call {
            signature: opened,
            callee,
            arguments: super::super::ir::ValueList { start: 0, end: 2 },
        },
    )
}

fn operand(
    body: &mut Body,
    facts: &Declarations,
    index: usize,
    parameter: TypeId,
) -> Result<(Place, TypeId), &'static str> {
    let place = dereference(
        body,
        Place::Local(LocalId(
            u32::try_from(index).map_err(|_| "reference parameter index overflow")?,
        )),
        parameter,
        &facts.types,
    )?;
    let Kind::Reference {
        referent: reference,
        ..
    } = facts
        .types
        .kind(parameter)
        .ok_or("missing reference parameter")?
    else {
        return Err("reference comparison requires borrowed reference parameters");
    };
    let place = dereference(body, place, reference, &facts.types)?;
    let Kind::Reference { referent, .. } = facts
        .types
        .kind(reference)
        .ok_or("missing reference operand")?
    else {
        return Err("reference comparison lost pointee");
    };
    Ok((place, referent))
}
fn callable(
    facts: &mut Declarations,
    source: &Validated,
    mut arguments: Vec<Argument>,
) -> Result<(TypeId, TypeId), &'static str> {
    let generic = facts
        .lifetimes
        .callables
        .generic_parameters(crate::types::ItemId(source.definition.0))
        .ok_or("missing reference method parameters")?;
    if generic.len() == 1 {
        arguments.truncate(1);
    }
    if generic.len() != arguments.len() {
        return Err("reference method has invalid generic arity");
    }
    let bindings = generic
        .iter()
        .copied()
        .zip(arguments.iter().copied())
        .collect::<Vec<_>>();
    let template = facts
        .lifetimes
        .callables
        .get(crate::types::ItemId(source.definition.0))
        .ok_or("missing reference method template")?
        .signature;
    let signature =
        Bound::parameters(&facts.types, &bindings)?.apply(&mut facts.types, template)?;
    let arguments = facts.types.intern_arguments(&arguments)?;
    let callable = facts.types.intern(Kind::FunctionItem {
        item: crate::types::ItemId(source.definition.0),
        signature,
        arguments,
    })?;
    let opened = Bound::new(vec![RegionId::bound(0, 0)?, RegionId::bound(0, 1)?]).signature(
        &mut facts.types,
        signature,
        0,
    )?;
    Ok((callable, opened))
}

pub(super) fn validate(
    facts: &mut Declarations,
    selected: Selected,
    source: &Validated,
    input: &Contract,
    parameters: &[TypeId],
) -> Result<(), &'static str> {
    let (expected, types) = contract(facts, selected, source)?;
    if parameters != types
        || input.signature != expected.signature
        || input.universals != expected.universals
        || input.assumptions != expected.assumptions
        || input.type_assumptions != expected.type_assumptions
        || input.builtins.is_some()
        || input.output.is_some()
        || input.require_result_formation
    {
        return Err(
            "reference factory differs from its exact declared prerequisite and lifetime abstraction",
        );
    }
    Ok(())
}
