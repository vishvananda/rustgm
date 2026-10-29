//! Generic source conformance retains the actual owner's immutable environment.
use super::*;

#[test]
fn blanket_type_self_uses_actual_environment_before_conformance() -> Result {
    let report = crate::test_source::source_provider(
        &source("generic-clone-prerequisite")?,
        "source_traits_v1",
    )?;
    let mut checker = Checker::new(&report)?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run()?;
    for name in ["Clone", "Copy"] {
        let owner = *owners(&checker, name)?
            .first()
            .ok_or("actual source owner")?;
        assert!(
            checker
                .facts
                .generics
                .implementations
                .as_ref()
                .ok_or("actual registry")?
                .is_validated(owner)
        );
    }
    let clone = *owners(&checker, "Clone")?
        .first()
        .ok_or("actual generic Clone owner")?;
    let mut pending = Pending::default();
    let plan = plan(&mut checker, &mut pending, clone)?;
    assert!(plan.environment.is_some());
    plan.require_no_associated(&checker.facts, clone)?;
    let before = pending.work;
    let again = super::plan(&mut checker, &mut pending, clone)?;
    again.require_current(&checker.facts, clone)?;
    assert!(pending.work > before);
    Ok(())
}

#[test]
fn blanket_type_formals_keep_real_owner_and_environment_identity() -> Result {
    let source = crate::test_source::fixture("pa31/tests/spec/source-div/size-only.rs")?;
    let source = source
        .split("pub struct PtrWords")
        .next()
        .ok_or("actual SIZE source prefix")?;
    let report = crate::test_source::source_provider(source, "size_plan")?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let owner = *owners(&checker, "SizedTypeProperties")?
        .first()
        .ok_or("actual generic owner")?;
    let mut pending = Pending::default();
    let actual = plan(&mut checker, &mut pending, owner)?;
    actual.require_constants(&checker.facts, owner)?;
    let environment = actual
        .environment
        .as_ref()
        .ok_or("actual blanket environment")?;
    let previous = checker
        .facts
        .generics
        .environments
        .insert(owner, Arc::new((**environment).clone()));
    assert!(
        actual
            .require_current(&checker.facts, owner)
            .is_err_and(|e| e.contains("environment"))
    );
    checker
        .facts
        .generics
        .environments
        .insert(owner, previous.ok_or("original environment")?);
    let parameters = checker
        .facts
        .generics
        .parameters
        .get(&owner)
        .ok_or("actual formals")?
        .clone();
    let foreign = checker
        .facts
        .generics
        .parameters
        .iter()
        .find(|(id, values)| **id != owner && !values.is_empty())
        .map(|(_, values)| values.clone())
        .ok_or("real foreign source formals")?;
    checker.facts.generics.parameters.insert(owner, foreign);
    let header = checker
        .facts
        .generics
        .implementations
        .as_ref()
        .and_then(|registry| registry.header(owner))
        .ok_or("actual header")?;
    let (node, predicate) = (header.source.ok_or("actual source")?, header.predicate);
    assert!(matches!(
        checker.early_impl_plan(owner, node, predicate, &mut pending)?,
        Eligibility::Unsupported(Unsupported::OpenSubject)
    ));
    checker.facts.generics.parameters.insert(owner, parameters);
    actual.require_current(&checker.facts, owner)?;
    assert!(pending.work > 0);
    pending.work = 1_000_000;
    assert!(
        checker
            .early_impl_plan(owner, node, predicate, &mut pending)
            .is_err_and(|e| e.message.contains("work budget"))
    );
    assert!(pending.work > 1_000_000);
    Ok(())
}

#[test]
fn blanket_constant_all_type_formals_require_real_trait_obligations() -> Result {
    let source = crate::test_source::fixture("pa31/tests/spec/source-div/type-blanket.rs")?;
    let report = crate::test_source::source_provider(&source, "type_blanket")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    assert!(facts.array_lengths.values().any(|&bound| bound == 7));
    let root = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| report.package.names.get(definition.name) == "blanket_value")
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or("actual export")?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[root],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-div-v1/candidate/type-blanket");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), &source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    let missing = source.replace(
        "pub trait PairValue<Rhs>{",
        "pub trait PairValue<Rhs>: Extra{",
    ) + "\npub trait Extra{}\n";
    let report = crate::test_source::source_provider(&missing, "missing_requirement")?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("trait requirement incorrectly became an implementation premise")?;
    assert!(
        error.message.contains("not satisfy") || error.message.contains("not satisfied"),
        "{error}"
    );
    Ok(())
}

#[test]
fn declared_blanket_bounds_supply_only_the_actual_prerequisite_environment() -> Result {
    let source = crate::test_source::fixture("pa31/tests/spec/source-div/type-blanket.rs")?;
    let source = source.split("const VALUE:u8=<").next().ok_or("actual declaration prefix")?
        .replace("pub trait PairValue<Rhs>{", "pub trait PairValue<Rhs>: Extra{")
        .replace("impl<T,U>", "impl<T:Extra,U>") + "\npub trait Extra{}\n";
    let report = crate::test_source::source_provider(&source, "declared_premise")?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let owner = *owners(&checker, "PairValue")?.first().ok_or("actual blanket owner")?;
    let mut pending = Pending::default();
    let actual = plan(&mut checker, &mut pending, owner)?;
    assert!(actual.assumptions.as_ref().is_some_and(|assumptions| !assumptions.is_empty()));
    let mut attempt = Attempt::new(actual);
    assert!(matches!(attempt.advance(&mut checker.facts, owner)?, Step::Complete));
    assert!(attempt.context.steps > 0);
    Ok(())
}
