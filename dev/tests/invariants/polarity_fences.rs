//! Consumer fences that admission keeps unreachable, reached by direct calls
//! on honest graphs. The `source-branches` provider exports under versions 12
//! and 13 and declares only ordinary traits, so each imported auto-trait and
//! auto-impl check rejects its traits and impls with its own text.
use crate::{
    lower::instances::SourcePublication, resolve::model::DefId, sema::associated::OwnerKind,
    support::crates, types::parameters::predicates::Bound,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Answer<T> = std::result::Result<T, &'static str>;
/// Each imported trait's and each imported trait impl's automatic answer.
type Answers = (Vec<Answer<()>>, Vec<Answer<bool>>);
const PROVIDER: &str = "pa29/tests/spec/source-branches/provider.rs";
const CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n";
/// The `source-branches` client's answers under one publication profile.
fn answers(profile: SourcePublication) -> Result<Answers> {
    let provider = crate::test_source::fixture(PROVIDER)?;
    let interface = crate::module_packets::exported_profile(&provider, profile, "source_branches")?;
    let decoded = crates::decode(&crates::encode(&interface)?)?;
    let (_, analysis, _) = crate::source_packets::client_analysis(decoded, CLIENT)?;
    let facts = &analysis.facts;
    let registry = facts
        .generics
        .implementations
        .as_deref()
        .ok_or("imported impl registry")?;
    let (mut traits, mut impls) = (Vec::new(), Vec::new());
    for header in registry.headers().filter(|header| header.source.is_none()) {
        let Bound::Trait(application) = header.predicate.bound else {
            return Err("imported trait impl predicate".into());
        };
        let definition = facts
            .types
            .trait_application(application)
            .map(|application| DefId(application.item.0))
            .ok_or("imported trait application")?;
        let owner = facts
            .associated
            .owner(header.owner)
            .and_then(|id| facts.associated.get(id))
            .copied()
            .filter(|owner| matches!(owner.kind, OwnerKind::ImportedImpl(_)))
            .ok_or("imported impl owner")?;
        traits.push(facts.associated.imported_automatic_trait(definition));
        impls.push(facts.associated.imported_automatic_implementation(
            &facts.types,
            owner,
            header.predicate,
        ));
    }
    assert!(!impls.is_empty(), "source-branches declares trait impls");
    Ok((traits, impls))
}
#[test]
fn imported_auto_checks_reject_ordinary_traits_and_older_profiles_by_name() -> Result {
    for (profile, trait_text, impl_text) in [
        (
            SourcePublication::ModuleBodies,
            "imported automatic trait requires its actual auto trait owner",
            "imported automatic implementation requires its exact auto trait",
        ),
        (
            SourcePublication::ModuleBranches,
            "imported automatic trait requires its version-thirteen profile",
            "imported automatic implementation requires its version-thirteen profile",
        ),
    ] {
        let (traits, impls) = answers(profile)?;
        assert!(
            traits.iter().all(|answer| *answer == Err(trait_text)),
            "{traits:?}"
        );
        assert!(
            impls.iter().all(|answer| *answer == Err(impl_text)),
            "{impls:?}"
        );
    }
    Ok(())
}
