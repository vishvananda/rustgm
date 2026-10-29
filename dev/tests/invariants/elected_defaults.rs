//! A unit choice neither constructs payloads nor invents their Default premises.
use super::*;
const ENUM_FIXTURES: &str = "pa31/tests/spec/enum-default/";
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{ENUM_FIXTURES}{name}.rs"))?)
}

#[test]
fn original_abi_and_const_generics_preserve_actual_choice_and_constraints() -> TestResult {
    let source = input("abi-expansion")?;
    let report = expanded_provider(&source, "actual_abi_default")?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual Abi transaction")?
        .entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].contexts.len(), 2);
    report.expanded_derive(entries[0].owner, entries[0].effect)?;
    let output: String = report.package.units[0].source.text()[source.len()..]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(output.contains("Self::ExternRust"));
    let source = input("generic-source")?;
    let report = expanded_provider(&source, "actual_generic_default")?;
    let output: String = report.package.units[0].source.text()[source.len()..]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(output.contains("T,'a") || output.contains("T:'a"));
    assert!(output.contains("constN:usize"));
    assert!(!output.contains("T:crate::default::Default"));
    let source = input("abi-expansion")?.replace(
        "allow_internal_unstable)]",
        "allow_internal_unstable,derive_const)]",
    );
    let end = source
        .find("#[derive(Debug, Default)]")
        .ok_or("original Abi")?;
    let source = source[..end].to_owned()
        + "#[derive_const(Default)]enum Const<T>{#[default]First,Payload(T)}";
    let report = expanded_provider(&source, "const_enum_default")?;
    let output: String = report.package.units[0].source.text()[source.len()..]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(output.contains("constimpl<T,"));
    assert!(!output.contains("T:crate::default::Default"));
    Ok(())
}

#[test]
fn nondefault_payload_and_configured_unit_choice_reach_actual_lowir() -> TestResult {
    produce("generic-source", "enum_default_native", "provider")
}

fn produce(name: &str, tag: &str, artifact: &str) -> TestResult {
    let source = input(name)?;
    let report = test_source::source_provider(&source, tag)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
    );
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/enum-default-v1/candidate");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join(format!("{artifact}.rs")), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join(format!("{artifact}.lowir")))?,
    )?;
    Ok(())
}

#[test]
fn ordinary_self_unit_paths_keep_actual_owner_arguments_and_associated_boundaries() -> TestResult {
    produce("self-source", "ordinary_self_values", "self-provider")?;
    let source = input("self-associated")?;
    let report = test_source::source_provider(&source, "self_associated_items")?;
    crate::pipeline::analyze_source(&report)?;
    for case in [
        "self-missing",
        "self-wrong-owner",
        "self-nonunit",
        "self-wrong-argument",
        "self-suffix-argument",
        "self-prefix-argument",
        "self-abstract",
        "self-nonenum",
    ] {
        let source = input(case)?;
        let result =
            test_source::source_provider(&source, "self_value_opponent").and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        let error = result.err().ok_or_else(|| format!("accepted {case}"))?;
        assert!(!error.contains("budget"), "{case}: {error}");
    }
    Ok(())
}

#[test]
fn marker_ownership_malformed_selection_and_mutated_replay_cannot_grant_a_choice() -> TestResult {
    let source = input("generic-source")?;
    let prefix = source
        .split("struct NoDefault")
        .next()
        .ok_or("source prefix")?;
    for case in [
        "missing",
        "multiple",
        "duplicate",
        "arguments",
        "equals",
        "nonunit-tuple",
        "nonunit-record",
        "empty-tuple",
        "empty-record",
        "non-exhaustive-variant",
        "misplaced-item",
        "misplaced-field",
        "cfg-missing",
        "wrong-helper",
        "const-gate",
    ] {
        let source = prefix.to_owned() + &input(&format!("{case}-rust"))?;
        assert!(
            expanded_provider(&source, "bad_enum_default").is_err(),
            "{case}"
        );
    }
    let wrong = source.replace(
        "rustc_builtin_macro(Default, attributes(default))",
        "rustc_builtin_macro(Default)",
    );
    assert!(expanded_provider(&wrong, "missing_helper_claim").is_err());
    replay_mutations(&source)
}

fn replay_mutations(source: &str) -> TestResult {
    let mut report = expanded_provider(source, "changed_enum_helper")?;
    let (owner, effect, choice) = {
        let entry = report
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or("actual choice")?
            .entries
            .iter()
            .find(|entry| entry.enum_default.is_some())
            .ok_or("enum choice")?;
        (
            entry.owner,
            entry.effect,
            entry.enum_default.ok_or("actual choice")?,
        )
    };
    report.expanded_derive(owner, effect)?;
    let previous = report.package.units[0].configured.effects[choice.effect]
        .attribute
        .attribute
        .input;
    report.package.units[0].configured.effects[choice.effect]
        .attribute
        .attribute
        .input = crate::syntax::attribute::Input::Equals(crate::syntax::token_tree::TreeRange {
        start: 0,
        end: 0,
    });
    assert!(report.expanded_derive(owner, effect).is_err());
    report.package.units[0].configured.effects[choice.effect]
        .attribute
        .attribute
        .input = previous;
    report.expanded_derive(owner, effect)?;
    let transactions = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.derives.as_deref_mut())
        .ok_or("actual transactions")?;
    let position = *transactions
        .index
        .get(&(owner.file, effect))
        .ok_or("actual enum transaction")?;
    transactions.entries[position].enum_default = None;
    assert!(report.expanded_derive(owner, effect).is_err());
    report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.derives.as_deref_mut())
        .ok_or("actual transactions")?
        .entries[position]
        .enum_default = Some(choice);
    report.expanded_derive(owner, effect)?;
    report.package.units[0].configured.active[choice.variant.get() as usize] = false;
    assert!(report.expanded_derive(owner, effect).is_err());
    Ok(())
}

#[test]
fn choice_work_and_replay_scale_and_failed_payment_publishes_no_derive() -> TestResult {
    use std::fmt::Write as _;
    let source = input("generic-source")?;
    let prefix = source
        .split("struct NoDefault")
        .next()
        .ok_or("source prefix")?;
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut source = prefix.to_owned() + "#[derive(Default)]enum Many<T>{#[default]Selected,";
        for index in 0..count {
            write!(&mut source, "V{index}(T),")?;
        }
        source.push('}');
        let report = expanded_provider(&source, "many_enum_choices")?;
        let payer = &report.package.source_core().ok_or("source payer")?.macros;
        let cold = payer.usage();
        let entries = &report
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or("actual choice")?
            .entries;
        for entry in entries {
            report.expanded_derive(entry.owner, entry.effect)?;
        }
        let warm = payer.usage();
        assert!(warm.0 > cold.0);
        if let Some((work, bytes)) = previous {
            assert!(warm.0 <= work * 6);
            assert!(warm.1 <= bytes * 6);
        }
        previous = Some((warm.0, warm.1));
        eprintln!("enum variants={count} cold={cold:?} warm={warm:?}");
    }
    super::super::super::super::role_declarations::with_engine(&source, true, |engine| {
        let before = engine.loader.package.units[0].source.text().len();
        assert!(
            engine
                .loader
                .package
                .source_core()
                .ok_or("source payer")?
                .macros
                .charge(usize::MAX, 0)
                .is_err()
        );
        assert!(engine.run().is_err());
        assert_eq!(engine.loader.package.units[0].source.text().len(), before);
        Ok(())
    })?;
    eprintln!(
        "enum Site={} Transaction={} Choice={} Body={} Node={}",
        size_of::<crate::expand::builtin::derive_output::Site<'_>>(),
        size_of::<Transaction>(),
        size_of::<super::super::super::variant_defaults::Choice>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::syntax::ast::Node>()
    );
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    Ok(())
}
