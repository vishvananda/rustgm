//! Selected trait outputs substitute implementation inputs and captured method inputs together.
use super::super::model::Declarations;
use crate::{
    resolve::model::DefId,
    types::{
        GenericArgs, ItemId, TypeId,
        arguments::Argument,
        opaque::OpaqueId,
        parameters::predicates::{Bound as PredicateBound, Predicate},
        substitute::{Alias, Bound},
        traits::Application,
    },
};

pub(in crate::sema) fn requirement(
    facts: &mut Declarations,
    alias: Alias,
) -> Result<Predicate, &'static str> {
    let Alias::TraitResult(id, arguments) = alias else {
        let application = alias.associated().ok_or("missing associated application")?;
        return Ok(Predicate {
            ty: application.subject,
            bound: PredicateBound::Trait(application.trait_application),
        });
    };
    let declaration = facts
        .opaques
        .declarations
        .get(&id)
        .ok_or("missing trait result declaration")?;
    let owner = facts
        .associated
        .trait_owner(declaration.owner)
        .ok_or("missing result trait owner")?;
    let subject = facts
        .associated
        .schema(owner)
        .ok_or("missing result trait schema")?
        .self_type;
    let mut fold = declaration.substitution(&facts.types, arguments)?;
    let arguments = facts
        .generics
        .parameters
        .get(&owner)
        .into_iter()
        .flatten()
        .map(|p| p.argument)
        .collect::<Vec<_>>();
    let arguments = facts.types.intern_arguments(&arguments)?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(owner.0),
        arguments,
    })?;
    fold.predicate(
        &mut facts.types,
        Predicate {
            ty: subject,
            bound: PredicateBound::Trait(application),
        },
    )
}

/// The caller must retain the selected implementation proof. This maps structure only.
pub(in crate::sema) fn result(
    facts: &mut Declarations,
    owner: DefId,
    selected: GenericArgs,
    id: OpaqueId,
    arguments: GenericArgs,
) -> Result<Option<TypeId>, &'static str> {
    let declaration = facts
        .opaques
        .declarations
        .get(&id)
        .ok_or("missing trait result declaration")?;
    let Some(template) = facts.opaques.implementations.get(&(owner, id)) else {
        return Ok(None);
    };
    let ty = template.ty;
    let parameters = facts
        .generics
        .parameters
        .get(&owner)
        .map_or(&[][..], Vec::as_slice);
    let parent_captures = facts
        .associated
        .schema(owner)
        .ok_or("missing selected result parent schema")?
        .captures;
    let crate::types::substitute::families::Mappings {
        parameters: mut bindings,
        mut regions,
    } = crate::types::substitute::families::mappings(
        &facts.types,
        crate::types::substitute::families::Application {
            owner: ItemId(owner.0),
            parent: selected,
            member: ItemId(owner.0),
            own: GenericArgs::EMPTY,
        },
        (parameters.iter().map(|p| p.argument), std::iter::empty()),
        parent_captures,
    )?;
    let actual = facts
        .types
        .arguments(arguments)
        .ok_or("invalid result capture arguments")?;
    template.method_inputs(
        &facts.types,
        declaration,
        actual,
        &mut bindings,
        &mut regions,
        |argument| match argument {
            Argument::Lifetime(region) => Some(region),
            Argument::Type(_) | Argument::Const(_) => None,
        },
    )?;
    Bound::parameters_captured(&facts.types, &bindings, regions)?
        .apply(&mut facts.types, ty)
        .map(Some)
}
