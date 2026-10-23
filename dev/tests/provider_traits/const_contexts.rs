//! Closed initializer demand never rewrites a generic const body's declaration.
use super::{TestResult, cases, report};
use rustgm::{
    pipeline,
    types::parameters::{clauses::Clause, predicates::Bound},
};

pub(super) fn source(paid: bool, constant: bool) -> TestResult<String> {
    let (prefix, _) = cases::UNUSED_CONDITIONAL_CONST_PAID
        .split_once("const fn unused")
        .ok_or("prefix")?;
    let mut source = prefix.to_owned();
    if !paid {
        source = source.replace("const impl Clone for u8", "impl Clone for u8");
    }
    source.push_str("const fn unused<T:[const]Clone>()->u8{7}const fn outer<T:[const]Clone>()->u8{unused::<T>()}");
    source.push_str(if constant {
        "const N:u8=outer::<u8>();#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{N}"
    } else {
        "#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{outer::<u8>()}"
    });
    source.push('\n');
    Ok(source)
}

#[test]
fn generic_defining_clauses_stay_conditional_in_runtime_and_closed_const_uses() -> TestResult {
    for (paid, constant, accepted) in [
        (false, false, true),
        (true, true, true),
        (false, true, false),
    ] {
        let report = report(&source(paid, constant)?)?;
        let analysis = pipeline::analyze_source(&report);
        if !accepted {
            assert!(analysis.is_err());
            continue;
        }
        let analysis = analysis?;
        let owner = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == "outer")
            .ok_or("outer")?;
        let owner = rustgm::resolve::model::DefId(u32::try_from(owner)?);
        assert!(analysis.facts.generics.predicates_in_scope(owner).any(|p| matches!(p.value, Clause::Predicate(value) if matches!(value.bound, Bound::MaybeConstTrait(_)))));
    }
    Ok(())
}

#[test]
fn actual_const_impl_overlap_and_unused_body_legality_keep_exact_owners() -> TestResult {
    for (name, accepted) in [
        ("overlapping-effects", false),
        ("unused-invalid-const-override", false),
        ("unused-valid-const-override", true),
    ] {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../pa31/tests/spec/source-const-payment")
                .join(format!("{name}.rs")),
        )?;
        let result = super::lower(&source);
        if accepted {
            result?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("{name} accepted"))?
                .to_string();
            if name == "unused-invalid-const-override" {
                assert!(
                    error.contains("non-const calls are not allowed in const functions"),
                    "{error}"
                );
            } else {
                assert!(
                    error.contains("conflict") || error.contains("overlap"),
                    "{error}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn conditional_intrinsic_keeps_its_kernel_boundary_and_ordinary_names_keep_bodies() -> TestResult {
    for (name, accepted) in [
        ("conditional-intrinsic-fallback", false),
        ("conditional-ordinary-lookalike", true),
    ] {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../pa31/tests/spec/source-const-payment")
                .join(format!("{name}.rs")),
        )?;
        let report = report(&source)?;
        let analysis = pipeline::analyze_source(&report);
        if accepted {
            analysis?;
        } else {
            let error = analysis
                .err()
                .ok_or("conditional intrinsic fallback executed")?
                .to_string();
            assert!(
                error.contains("payment for intrinsic targets is not implemented"),
                "{error}"
            );
        }
    }
    Ok(())
}
