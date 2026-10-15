//! Inferred aliases preserve erased Sized premises in the body's shared inference graph.
use super::{Fields, Host, error};
use crate::{
    resolve::{model::DefId, package::Error, unit::NodeKey},
    sema::{
        alias_requirements::{Scope, Use},
        generics::AnnotationArgument,
    },
    types::{
        parameters::predicates::{Bound, Predicate},
        substitute,
    },
};

pub(super) fn retain(
    host: &mut impl Host,
    key: NodeKey,
    owner: DefId,
    supplied: &[AnnotationArgument],
    fields: &mut Fields,
) -> Result<(), Error> {
    let parameters = &host.facts().generics.parameters[&owner];
    let owners: Vec<_> = std::iter::once(owner)
        .chain(
            parameters
                .iter()
                .zip(supplied)
                .filter_map(|(parameter, source)| {
                    matches!(source, AnnotationArgument::Default(_)).then_some(parameter.binding)
                }),
        )
        .filter(|&owner| host.facts().alias_requirements.contains(owner))
        .collect();
    if owners.is_empty() {
        return Ok(());
    }
    // Expand canonical parameter identities first, then substitute the same inference
    // variables as the result. A normalized result never determines an erased input.
    let arguments = parameters.iter().map(|p| p.argument).collect::<Vec<_>>();
    let arguments = host
        .inference()
        .0
        .types
        .intern_arguments(&arguments)
        .map_err(|e| error(host, key, e))?;
    let mut scope = Scope::default();
    for owner in owners {
        let (facts, _) = host.inference();
        let expansion = facts
            .alias_requirements
            .instantiate(&mut facts.types, Use { owner, arguments }, 1, &mut scope)
            .map_err(|e| error(host, key, e))?;
        if !expansion.proof.is_empty() || !expansion.normalizations.is_empty() {
            return Err(error(
                host,
                key,
                "inferred alias proofs require delayed projection normalization",
            ));
        }
        for (scope, ty) in expansion.sized {
            let depth = expansion
                .proof
                .scope_depth(scope)
                .map_err(|e| error(host, key, e))?;
            // Local quantified lifetimes cannot be moved into the body's scope.
            // `lower` checks this while restoring captured caller lifetimes.
            let ty = substitute::Bound::lower(depth)
                .apply(&mut host.inference().0.types, ty)
                .map_err(|e| error(host, key, e))?;
            let ty = fields
                .regions
                .apply(&mut host.inference().0.types, ty)
                .map_err(|e| error(host, key, e))?;
            let (facts, infer) = host.inference();
            let predicate = infer
                .instantiate_predicate(
                    &mut facts.types,
                    Predicate {
                        ty,
                        bound: Bound::Sized,
                    },
                    &mut fields.parameters,
                )
                .map_err(|e| error(host, key, e))?;
            host.requirement(key, predicate);
        }
    }
    Ok(())
}
