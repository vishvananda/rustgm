//! Actual root self declarations share the existing extern prelude, without factories.
use super::*;
use crate::{resolve::index::State, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn actual_self_aliases_preserve_ordinary_and_source_lookup_shadowing_and_privacy() -> Result {
    for (name, accepted) in [
        ("nested", true),
        ("macro", true),
        ("inner-shadow", true),
        ("root-shadow", false),
        ("nonroot", false),
        ("no-prelude", false),
        ("no-prelude-explicit", true),
        ("private", false),
    ] {
        let input = test_source::fixture(&format!("pa6/tests/spec/self-aliases/{name}.rs"))?;
        let ordinary = test_source::resolve(&input, true);
        assert_eq!(
            ordinary.is_ok(),
            accepted,
            "ordinary {name}: {:?}",
            ordinary.err()
        );
        let source = test_source::source_provider(
            &format!("#![feature(no_core)]#![no_core]{input}"),
            "self_alias",
        );
        assert_eq!(
            source.is_ok(),
            accepted,
            "source {name}: {:?}",
            source.err()
        );
    }
    Ok(())
}

#[test]
fn self_alias_prelude_rejects_changed_actual_declaration_owner_and_members() -> Result {
    let mut report = test_source::source_provider(
        "#![feature(no_core)]#![no_core]extern crate self as local;mod unrelated{}",
        "alias_owner",
    )?;
    let name = report.package.names.intern("local")?;
    let external = report
        .resolution
        .extern_scope
        .ok_or("missing actual extern scope")?;
    let binding = *report
        .resolution
        .bindings
        .get(external, name, Namespace::Type)
        .ok_or("missing actual self alias")?;
    let mut state = State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let resolver = state.bind(&mut report.package);
    assert!(resolver.self_alias(name, &binding));
    let definition = &mut resolver.facts.definitions[binding.target.0 as usize];
    let original_members = definition.members;
    definition.members = None;
    assert!(!resolver.self_alias(name, &binding));
    resolver.facts.definitions[binding.target.0 as usize].members = original_members;
    resolver.facts.definitions[binding.target.0 as usize].kind = DefKind::Module;
    assert!(!resolver.self_alias(name, &binding));
    resolver.facts.definitions[binding.target.0 as usize].kind = DefKind::ExternCrate;
    resolver.facts.definitions[binding.target.0 as usize].scope = external;
    assert!(!resolver.self_alias(name, &binding));
    resolver.facts.definitions[binding.target.0 as usize].scope = ScopeId(0);
    let mut ambiguous = binding;
    ambiguous.ambiguous = true;
    assert!(!resolver.self_alias(name, &ambiguous));
    let other = resolver.package.names.intern("other")?;
    assert!(!resolver.self_alias(other, &binding));
    assert!(resolver.self_alias(name, &binding));
    Ok(())
}

#[test]
fn actual_self_alias_borrows_lexical_stability_without_overwriting_parent_policy() -> Result {
    for (name, accepted) in [("unstable", true), ("stable", false)] {
        let input = test_source::fixture(&format!(
            "pa31/tests/spec/source-format-macros/arguments/stability-alias/{name}.rs"
        ))?;
        let result = (|| -> Result {
            let report = test_source::resolve(&input, true)?;
            crate::pipeline::analyze_source(&report)?;
            Ok(())
        })();
        if accepted {
            result?;
        } else {
            let error = result
                .err()
                .ok_or("stable parent lost its actual child policy requirement")?;
            assert!(
                error.to_string().contains("missing library stability"),
                "{error}"
            );
        }
    }
    Ok(())
}
