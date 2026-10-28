//! Authenticated source requirements become clauses without publishing assumptions.
#[cfg(test)]
#[path = "../../../../tests/invariants/annotated_features.rs"]
mod tests;
use crate::{
    resolve::{
        expansion::Report,
        lint_policy::DeclarationAnnotation,
        model::{DefId, DefKind},
        package::Error,
        stability::{Level, PolicyKind},
        unit::NodeKey,
    },
    sema::{associated::OwnerKind, generics::Predicate, model::Declarations},
    syntax::{ast::Kind, item::ItemKind},
    types::parameters::clauses::Clause,
};

pub(in crate::sema::declarations) fn declaration_clauses(
    report: &Report,
    facts: &mut Declarations,
    owner: DefId,
    key: NodeKey,
) -> Result<Option<Vec<Predicate>>, Error> {
    let producer = report.lint_producer().ok_or_else(|| {
        Error::at(
            key.file,
            0,
            "feature clause query requires its authenticated attribute producer",
        )
    })?;
    producer
        .diagnostic_payload(key, 1, 0)
        .map_err(Error::from)?;
    if report
        .resolution
        .definitions
        .get(owner.0 as usize)
        .is_none_or(|definition| definition.node != Some(key))
    {
        return Err(Error::at(
            key.file,
            0,
            "feature requirement changed its actual source definition",
        ));
    }
    let Some(annotation) = producer
        .declaration_annotation(&report.package, key)
        .map_err(Error::from)?
        .filter(|annotation| annotation.bound.is_some())
    else {
        return Ok(None);
    };
    let error = |message| annotation_error(key, annotation, message);
    let pay = |steps, bytes| {
        producer
            .diagnostic_payload(key, steps, bytes)
            .map_err(|error| error.message)
    };
    pay(4, 0).map_err(error)?;
    require_owner(report, facts, owner, key).map_err(error)?;
    let inventory = report
        .package
        .library_policy()
        .and_then(crate::resolve::stability::Library::staged_inventory)
        .ok_or_else(|| error("unstable_feature_bound requires completed staged_api policies"))?;
    let policy = inventory
        .local_declaration(owner)
        .map_err(error)?
        .ok_or_else(|| error("unstable_feature_bound requires its actual unstable owner"))?;
    if policy.kind != PolicyKind::Regular || !matches!(policy.level, Level::Unstable { .. }) {
        return Err(error(
            "unstable_feature_bound requires an actual regular unstable owner",
        ));
    }
    pay(
        annotation.bounds.len(),
        annotation
            .bounds
            .len()
            .saturating_mul(size_of::<Predicate>()),
    )
    .map_err(error)?;
    let mut clauses = Vec::with_capacity(annotation.bounds.len());
    for feature in annotation.bounds {
        let id = facts
            .types
            .intern_feature_charged(feature, &pay)
            .map_err(error)?;
        clauses.push(Predicate {
            source: annotation.owner,
            value: Clause::UnstableFeature(id),
        });
    }
    Ok(Some(clauses))
}
fn annotation_error(key: NodeKey, annotation: DeclarationAnnotation<'_>, message: &str) -> Error {
    Error::at(
        key.file,
        annotation.bound.map_or(0, |attribute| attribute.span.start),
        message,
    )
}
fn require_owner(
    report: &Report,
    facts: &Declarations,
    owner: DefId,
    key: NodeKey,
) -> Result<(), &'static str> {
    let definition = report
        .resolution
        .definitions
        .get(owner.0 as usize)
        .filter(|definition| definition.node == Some(key))
        .ok_or("feature requirement changed its actual source definition")?;
    let ast = &report.package.unit(key.file).configured.ast;
    let Kind::Item(index) = ast.node(key.node).kind else {
        return Err("feature requirement lost its actual declaration item");
    };
    let associated = facts
        .associated
        .owner(owner)
        .and_then(|id| facts.associated.get(id));
    let valid = match ast.items[index as usize].kind {
        ItemKind::Function { .. } => {
            definition.kind == DefKind::Function
                && associated.is_none()
                && report.resolution.scopes.get(definition.scope.0 as usize)
                    .is_some_and(|scope| !scope.associated)
                && !report.resolution.foreign_owners.contains_key(&owner)
        }
        ItemKind::Trait { .. } => {
            definition.kind == DefKind::Trait
                && associated.is_some_and(|associated| {
                    associated.source == Some(key)
                        && matches!(associated.kind, OwnerKind::Trait { definition, .. } if definition == owner)
                })
        }
        ItemKind::Impl { trait_path: Some(_), .. } => {
            definition.kind == DefKind::SelfType
                && associated.is_some_and(|associated| {
                    associated.source == Some(key)
                        && associated.self_definition == owner
                        && matches!(associated.kind, OwnerKind::Impl { trait_type: Some(_), .. })
                })
        }
        _ => false,
    };
    if !valid {
        return Err(
            "unstable_feature_bound requires its actual free function, trait or trait impl owner",
        );
    }
    Ok(())
}
