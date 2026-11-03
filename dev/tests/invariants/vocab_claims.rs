//! Pinned text tables check declarations independently from builtin implementations.
use crate::{
    pipeline,
    support::bindings::{GenericRequirement, LangItem},
    test_source,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct Case {
    name: String,
    path: String,
    accepted: bool,
    role: Option<String>,
}
fn cases() -> TestResult<Vec<Case>> {
    test_source::fixture("pa31/tests/spec/compiler-language-claims/cases-v5.tsv")?
        .lines()
        .map(|line| {
            let [name, path, status, role]: [_; 4] = line
                .split('\t')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| "four case columns")?;
            if !matches!(status, "0" | "1") {
                return Err("invalid predicted outcome".into());
            }
            Ok(Case {
                name: name.to_owned(),
                path: path.to_owned(),
                accepted: status == "0",
                role: (!role.is_empty()).then(|| role.to_owned()),
            })
        })
        .collect()
}
fn input(case: &Case) -> TestResult<String> {
    Ok(test_source::fixture(&case.path)?)
}
fn source(name: &str) -> TestResult<String> {
    input(
        &cases()?
            .into_iter()
            .find(|case| case.name == name)
            .ok_or("case")?,
    )
}
#[test]
fn pinned_declaration_targets_formals_and_weak_only_policy_match_all_source_outcomes() -> TestResult
{
    let cases = cases()?;
    assert_eq!(cases.len(), 135);
    for case in cases {
        let outcome = test_source::source_provider(&input(&case)?, "language_claims");
        if case.accepted {
            let report = outcome.map_err(|error| format!("{}: {error}", case.name))?;
            if let Some(name) = case.role.as_deref() {
                let role = LangItem::from_name(name).ok_or("known role")?;
                assert!(
                    report
                        .resolution
                        .source_bindings
                        .as_deref()
                        .ok_or("reservations")?
                        .language(role)
                        .is_some(),
                    "{name}"
                );
            }
        } else {
            let error = outcome
                .err()
                .ok_or_else(|| format!("invalid {} accepted", case.name))?;
            assert!(
                error.contains("required declaration target")
                    || error.contains("required generic arity")
                    || error.contains("duplicate source language item")
                    || error.contains("unsupported source language item")
                    || error.contains("lang requires its explicit source feature"),
                "{}: {error}",
                case.name
            );
        }
    }
    Ok(())
}
#[test]
fn append_only_vocabulary_preserves_ordinals_spellings_and_checked_profiles() -> TestResult {
    assert_eq!(LangItem::ALL.len(), 187);
    assert_eq!(LangItem::DECLARATIONS.len(), 100);
    for (index, role) in LangItem::ALL.into_iter().enumerate() {
        assert_eq!(role.index(), index);
        assert_eq!(role.declaration_profile().is_some(), index >= 87);
    }
    for line in
        test_source::fixture("pa31/tests/spec/compiler-language-claims/roles-v5.tsv")?.lines()
    {
        let [name, ordinal, requirement, count]: [_; 4] = line
            .split('\t')
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| "four role columns")?;
        let role = LangItem::from_name(name).ok_or("role")?;
        assert_eq!(role.index(), ordinal.parse::<usize>()?);
        let expected = match requirement {
            "None" => GenericRequirement::Any,
            "Exact" => GenericRequirement::Exact(count.parse()?),
            _ => return Err("unsupported pinned generic requirement".into()),
        };
        assert_eq!(
            role.declaration_profile().ok_or("profile")?.generics,
            expected
        );
    }
    assert_eq!(LangItem::TypeId.index(), 86);
    assert_eq!(LangItem::CompilerMove.index(), 144);
    assert_eq!(LangItem::CompilerCopy.index(), 145);
    assert!(LangItem::from_name("send").is_none());
    assert_eq!(size_of::<LangItem>(), 1);
    Ok(())
}
#[test]
fn unimplemented_builtin_claims_stop_before_granting_layout_drop_or_auto_trait_semantics()
-> TestResult {
    for case in cases()? {
        if !case.accepted {
            continue;
        }
        let Some(name) = case.role.as_deref() else {
            continue;
        };
        let role = LangItem::from_name(name).ok_or("known role")?;
        if matches!(role, LangItem::CompilerMove | LangItem::CompilerCopy) {
            continue;
        }
        let report = test_source::source_provider(&input(&case)?, "language_claims")?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("unimplemented builtin accepted")?
            .to_string();
        assert!(
            error.contains("source compiler language item semantic contract is not implemented"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn profiling_hooks_execute_actual_ordinary_bodies_and_fence_unencoded_transport() -> TestResult {
    let source = source("ordinary-marker-bodies")?;
    let report = test_source::source_provider(&source, "language_claims")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    for role in [LangItem::CompilerMove, LangItem::CompilerCopy] {
        assert!(catalog.language(role).is_some());
        assert!(catalog.validated_language(role).is_err());
    }
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["observed_move", "observed_copy"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some())
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/compiler-language-claims-v1/native");
    std::fs::create_dir_all(&folder)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("ordinary-marker-bodies.lowir"))?,
    )?;
    let provider = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let error = crate::sema::crates::authored_export::Stage::begin_module_conversions(
        &report, &mut facts, &provider,
    )
    .err()
    .ok_or("unencoded profiling contract exported")?;
    assert_eq!(
        error,
        "source compiler language contract transport is not implemented"
    );
    let invalid = source.replace("()->u8{9}", "()->u8{false}");
    assert_ne!(invalid, source);
    let invalid = test_source::source_provider(&invalid, "language_claims")?;
    assert!(pipeline::analyze_source(&invalid).is_err());
    Ok(())
}
#[test]
fn measure_appended_cold_storage_without_widening_ordinary_records() -> TestResult {
    use super::super::{Catalog, binding_views, roles};
    let catalog = size_of::<Catalog>();
    let reservations = size_of::<crate::resolve::bindings::Reservations>();
    let provider = size_of::<binding_views::Provider>();
    let imported = size_of::<binding_views::Imported>();
    let role_set = size_of::<roles::RoleSet>();
    let body = size_of::<crate::body::ir::Body>();
    let node = size_of::<crate::syntax::ast::Node>();
    let node_facts = size_of::<crate::sema::model::NodeFacts>();
    let sizes = format!(
        r#"{{"roles":187,"catalog":{catalog},"reservations":{reservations},"provider":{provider},"imported":{imported},"role_set":{role_set},"body":{body},"node":{node},"node_facts":{node_facts}}}"#
    );
    println!("{sizes}");
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/compiler-language-claims-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("measured-sizes.json"), sizes)?;
    assert_eq!(body, 456);
    assert_eq!(node, 36);
    assert_eq!(node_facts, 80);
    Ok(())
}
