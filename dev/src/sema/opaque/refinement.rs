//! Refined trait outputs share unresolved captures while the original query retains proof.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        Kind, RegionId,
        infer::{Context, Instantiation, TermId, arguments::Argument, predicates::Predicate},
        opaque::OpaqueId,
        outlives::ResultId,
        parameters::ParameterId,
        substitute::{Alias, Bound},
    },
};

pub(in crate::sema) fn application(
    facts: &mut Declarations,
    context: &mut Context,
    root: TermId,
) -> Result<(OpaqueId, Predicate, Vec<Argument>), String> {
    let Some(ResultId::Opaque(id)) = context
        .result_identity(&facts.types, root)
        .map_err(inference)?
    else {
        return Err("missing inferred trait result identity".into());
    };
    let actual = context
        .item_arguments(&facts.types, root)
        .map_err(inference)?;
    let declaration = facts
        .opaques
        .declarations
        .get(&id)
        .ok_or("missing inferred result declaration")?;
    let Some(Kind::Opaque(_, formal)) = facts.types.kind(declaration.application) else {
        return Err("invalid inferred result template".into());
    };
    let mut captures = declaration.inferred_arguments(&facts.types, &actual)?;
    let predicate = super::overrides::requirement(facts, Alias::TraitResult(id, formal))?;
    let predicate = captures.regions.predicate(&mut facts.types, predicate)?;
    context
        .account_work(captures.regions.steps)
        .map_err(inference)?;
    let mut substitution =
        Instantiation::new(context, &facts.types, &captures.parameters).map_err(inference)?;
    let predicate = context
        .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
        .map_err(inference)?;
    Ok((id, predicate, actual))
}

pub(in crate::sema) struct Inputs {
    pub owner: DefId,
    pub id: OpaqueId,
    pub arguments: Vec<Argument>,
    pub bindings: Vec<(ParameterId, Argument)>,
    pub regions: Vec<RegionId>,
}
pub(in crate::sema) fn result(
    facts: &mut Declarations,
    context: &mut Context,
    mut inputs: Inputs,
) -> Result<Option<TermId>, String> {
    let Some(template) = facts
        .opaques
        .implementations
        .get(&(inputs.owner, inputs.id))
    else {
        return Ok(None);
    };
    let declaration = facts
        .opaques
        .declarations
        .get(&inputs.id)
        .ok_or("missing inferred result declaration")?;
    template.method_inputs(
        &facts.types,
        declaration,
        inputs.arguments.into_iter(),
        &mut inputs.bindings,
        &mut inputs.regions,
        |argument| match argument {
            Argument::Lifetime(region) => Some(region),
            Argument::Type(_) | Argument::Const(_) => None,
        },
    )?;
    let mut fold = Bound::new(inputs.regions);
    let ty = fold.apply(&mut facts.types, template.ty)?;
    context.account_work(fold.steps).map_err(inference)?;
    context
        .instantiate(&mut facts.types, ty, &inputs.bindings)
        .map(Some)
        .map_err(inference)
}
fn inference(error: crate::types::infer::Error) -> String {
    error.to_string()
}
