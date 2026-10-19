//! Canonical builtin clauses pass actual call, lifetime, instance and `LowIR` consumers.
use super::{BYTE, Bound, Clause, Kind, Predicate, Proof, RegionId, TestResult, tests};
use crate::{rustc, sema, types::parameters::clauses::Quantified};
use std::sync::Arc;

fn retain_native_case(name: &str, program: &crate::lowir::program::Program) -> TestResult {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/language-qualification/{}/{name}.lowir",
        std::process::id()
    ));
    crate::lowir::writer::write(program, &mut std::fs::File::create(&path)?)?;
    eprintln!("builtin native input: {}", path.display());
    Ok(())
}

fn install(
    facts: &mut sema::model::Declarations,
    report: &crate::resolve::expansion::Report,
    ty: crate::types::TypeId,
    bound: Bound,
    count: u32,
) -> TestResult {
    let owner = tests::definition(report, "f")?;
    let function = *facts
        .functions
        .iter()
        .find(|f| f.definition == owner)
        .ok_or("missing function")?;
    let source = function.body.ok_or("missing body")?;
    let Some(Kind::Function { bound_regions, .. }) = facts.types.kind(function.signature) else {
        return Err("missing callable signature".into());
    };
    let id = facts.types.intern_quantified(Quantified {
        bound_regions: count,
        predicate: Predicate { ty, bound },
    })?;
    let clause = Clause::ForAll(id);
    let environment = Arc::new(facts.generics.parameter_environment(&facts.types, owner)?);
    let mut proof = Proof::for_declaration(Some(environment), bound_regions, 1);
    let formation = proof.validate_builtin_clause(facts, clause, clause)?;
    assert!(formation.satisfied);
    facts
        .generics
        .record_builtin_formation(owner, id, formation.subject)?;
    facts.generics.predicates.insert(
        owner,
        vec![sema::generics::Predicate {
            source,
            value: clause,
        }],
    );
    let environment = facts.generics.parameter_environment(&facts.types, owner)?;
    facts
        .generics
        .environments
        .insert(owner, Arc::new(environment));
    Ok(())
}

fn type_parameter(
    facts: &sema::model::Declarations,
    report: &crate::resolve::expansion::Report,
) -> TestResult<crate::types::TypeId> {
    let owner = tests::definition(report, "f")?;
    facts
        .generics
        .parameters_in_scope(owner)
        .find_map(|parameter| match parameter.argument {
            crate::types::arguments::Argument::Type(ty) => Some(ty),
            _ => None,
        })
        .ok_or_else(|| "missing function type parameter".into())
}

#[test]
fn builtin_assumptions_authorize_generic_copies_before_instance_lowering() -> TestResult {
    for count in [0, 1, 8] {
        for (bound, name) in [
            (Bound::Copy, "Copy"),
            (Bound::Clone, "Clone"),
            (Bound::Sized, "Sized"),
        ] {
            let binder = (0..count)
                .map(|i| format!("'a{i}"))
                .collect::<Vec<_>>()
                .join(",");
            let source =
                "fn f<T>(x:T)->(T,T){(x,x)}fn main(){let p=f(7u8);if p.0!=7||p.1!=7{loop{}}}";
            rustc::qualify(
                &format!("builtin-template-{count}-{name}"),
                &source.replace("{(x,x)}", &format!("where for<{binder}>T:{name}{{(x,x)}}")),
                bound == Bound::Copy,
            )?;
            let (mut report, mut facts) = tests::declarations(source)?;
            let ty = type_parameter(&facts, &report)?;
            install(&mut facts, &report, ty, bound, count)?;
            let mut facts = sema::bodies::analyze(&report, facts)?;
            let entry = sema::entry::select(&mut report, &facts)?;
            let result = crate::lower::instances::build(&report, &mut facts, entry);
            if bound == Bound::Copy {
                retain_native_case(&format!("builtin-template-{count}-{name}"), &result?)?;
            } else {
                let error = result
                    .err()
                    .ok_or("non-Copy assumption authorized duplication")?;
                assert!(
                    error.message.contains("moved") || error.message.contains("consumed"),
                    "{error}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn builtin_composite_permissions_reach_ownership_without_reverse_element_bounds() -> TestResult {
    for whole in [false, true] {
        let source = if whole {
            "fn f<'x,T>(x:(&'x u8,T))->((&'x u8,T),(&'x u8,T)){(x,x)}fn main(){let n=3u8;let p=f((&n,7u8));if *p.0.0!=3||p.1.1!=7{loop{}}}"
        } else {
            "fn f<T>(x:T)->(T,T){(x,x)}fn main(){let p=f(7u8);if p.0!=p.1{loop{}}}"
        };
        rustc::qualify(
            &format!("builtin-composite-{whole}"),
            &source.replace("{(x,x)}", "where for<'a>(&'a u8,T):Copy{(x,x)}"),
            whole,
        )?;
        let (mut report, mut facts) = tests::declarations(source)?;
        let parameter = type_parameter(&facts, &report)?;
        let reference = facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(0, 0)?,
            referent: BYTE,
        })?;
        let fields = facts.types.intern_list(&[reference, parameter])?;
        let ty = facts.types.intern(Kind::Tuple(fields))?;
        install(&mut facts, &report, ty, Bound::Copy, 1)?;
        let mut facts = sema::bodies::analyze(&report, facts)?;
        let entry = sema::entry::select(&mut report, &facts)?;
        let result = crate::lower::instances::build(&report, &mut facts, entry);
        if whole {
            retain_native_case("builtin-composite-true", &result?)?;
        } else {
            let error = result
                .err()
                .ok_or("composite assumption authorized an element copy")?;
            assert!(
                error.message.contains("moved") || error.message.contains("consumed"),
                "{error}"
            );
        }
    }
    Ok(())
}

fn lower_correlated_copy(source: &str) -> TestResult<crate::lowir::program::Program> {
    let (mut report, mut facts) = tests::declarations(source)?;
    let parameter = type_parameter(&facts, &report)?;
    let reference = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: BYTE,
    })?;
    let fields = facts
        .types
        .intern_list(&[reference, reference, parameter])?;
    let subject = facts.types.intern(Kind::Tuple(fields))?;
    install(&mut facts, &report, subject, Bound::Copy, 1)?;
    let mut facts = sema::bodies::analyze(&report, facts)?;
    if source.contains("duplicate(x)") {
        assert!(
            !facts.generics.builtin_requirements.is_empty(),
            "inferred builtin use must retain its pending proof"
        );
    }
    let entry = sema::entry::select(&mut report, &facts)?;
    Ok(crate::lower::instances::build(&report, &mut facts, entry)?)
}

#[test]
fn builtin_correlations_accept_proved_lifetime_equalities() -> TestResult {
    for (parameters, equal) in [("'x,'y,T", false), ("'x:'y,'y:'x,T", true)] {
        let pair = "(&'x u8,&'y u8,T)";
        let source = format!(
            "fn f<{parameters}>(x:{pair})->({pair},{pair}){{(x,x)}}fn main(){{let a=3u8;let b=5u8;let p=f((&a,&b,7u8));if *p.0.0!=3||*p.1.1!=5||p.1.2!=7{{loop{{}}}}}}"
        );
        rustc::qualify(
            &format!("builtin-equal-captures-{equal}"),
            &source.replace("{(x,x)}", "where for<'a>(&'a u8,&'a u8,T):Copy{(x,x)}"),
            equal,
        )?;
        let result = lower_correlated_copy(&source);
        if equal {
            result?;
        } else {
            let error = result
                .err()
                .ok_or("unrelated lifetimes authorized a correlated copy")?;
            assert!(
                error.to_string().contains("moved") || error.to_string().contains("consumed"),
                "{error}"
            );
        }
    }
    Ok(())
}

fn correlated_bound_cases(called: bool) -> TestResult {
    for (name, parameters, implied, equal) in [
        ("unrelated", "'x,'y,T", false, false),
        ("forward", "'x:'y,'y,T", false, false),
        ("reverse", "'x,'y:'x,T", false, false),
        ("mutual", "'x:'y,'y:'x,T", false, true),
        ("cycle", "'x:'z,'y:'x,'z:'y,T", false, true),
        ("static", "'x:'static,'y:'static,T", false, true),
        ("static-one", "'x:'static,'y,T", false, false),
        ("implied", "'x:'y,'y,T", true, true),
        ("implied-one", "'x,'y,T", true, false),
    ] {
        let pair = "(&'x u8,&'y u8,T)";
        let extra = if implied { ",_proof:&'x &'y u8" } else { "" };
        let argument = if implied { ",&proof" } else { "" };
        let expression = if called { "duplicate(x)" } else { "(x,x)" };
        let source = format!(
            "fn duplicate<V:Copy>(v:V)->(V,V){{(v,v)}}fn f<{parameters}>(x:{pair}{extra})->({pair},{pair}){{{expression}}}static A:u8=3;static B:u8=5;fn main(){{let proof=&B;let p=f((&A,&B,7u8){argument});if *p.0.0!=3||*p.1.1!=5||p.1.2!=7{{loop{{}}}}}}"
        );
        let name = format!("builtin-equality-called-{called}-{name}");
        rustc::qualify(
            &name,
            &source.replace(
                &format!("{{{expression}}}"),
                &format!("where for<'a>(&'a u8,&'a u8,T):Copy{{{expression}}}"),
            ),
            equal,
        )?;
        let result = lower_correlated_copy(&source);
        if equal {
            retain_native_case(&name, &result.map_err(|e| format!("{name}: {e}"))?)?;
        } else {
            let error = result
                .err()
                .ok_or("unproved lifetime equality authorized a Copy call")?;
            let message = error.to_string();
            assert!(
                message.contains("Copy")
                    || message.contains("moved")
                    || message.contains("consumed"),
                "{name}: {error}"
            );
        }
    }
    Ok(())
}

#[test]
fn builtin_correlations_reach_call_site_proof_and_implied_bounds() -> TestResult {
    correlated_bound_cases(true)
}

#[test]
fn builtin_correlations_reach_direct_ownership_and_implied_bounds() -> TestResult {
    correlated_bound_cases(false)
}

#[test]
fn ordinary_composite_copy_bounds_follow_proved_lifetime_equalities() -> TestResult {
    for called in [false, true] {
        for (name, parameters, accepted) in [
            ("unrelated", "'x,'y,T", false),
            ("forward", "'x:'y,'y,T", false),
            ("mutual", "'x:'y,'y:'x,T", true),
        ] {
            let pair = "(&'x u8,&'y u8,T)";
            let expression = if called { "duplicate(x)" } else { "(x,x)" };
            let source = format!(
                "fn duplicate<V:Copy>(v:V)->(V,V){{(v,v)}}fn f<{parameters}>(x:{pair})->({pair},{pair})where (&'x u8,&'x u8,T):Copy{{{expression}}}fn main(){{let a=3u8;let b=5u8;let p=f((&a,&b,7u8));if *p.0.0!=3||*p.1.1!=5||p.1.2!=7{{loop{{}}}}}}"
            );
            let name = format!("ordinary-builtin-equality-called-{called}-{name}");
            rustc::qualify(&name, &source, accepted)?;
            let (mut report, facts) = tests::declarations(&source)?;
            let mut facts = sema::bodies::analyze(&report, facts)?;
            let entry = sema::entry::select(&mut report, &facts)?;
            let result = crate::lower::instances::build(&report, &mut facts, entry);
            if accepted {
                retain_native_case(&name, &result?)?;
            } else {
                let error = result
                    .err()
                    .ok_or("one-way lifetime bound granted a correlated copy")?;
                assert!(
                    error.message.contains("Copy")
                        || error.message.contains("moved")
                        || error.message.contains("consumed"),
                    "{error}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn builtin_clauses_reach_instance_lowering_after_call_site_proof() -> TestResult {
    for count in [0, 1, 8] {
        for accepted in [false, true] {
            let binder = (0..count)
                .map(|i| format!("'a{i}"))
                .collect::<Vec<_>>()
                .join(",");
            let subject = if accepted { "u8" } else { "&'static mut u8" };
            let source = "fn f()->u8{7}fn main(){if f()!=7{loop{}}}";
            rustc::qualify(
                &format!("builtin-instance-{count}-{accepted}"),
                &format!(
                    "fn f()->u8 where for<{binder}>{subject}:Copy{{7}}fn main(){{if f()!=7{{loop{{}}}}}}"
                ),
                accepted,
            )?;
            // Source admission is still gated. Install the complete declaration
            // clause, then use the ordinary body and instance pipeline unchanged.
            let (mut report, mut facts) = tests::declarations(source)?;
            let owner = tests::definition(&report, "f")?;
            let source = facts
                .functions
                .iter()
                .find(|f| f.definition == owner)
                .and_then(|f| f.body)
                .ok_or("missing builtin instance body")?;
            let ty = if accepted {
                BYTE
            } else {
                facts.types.intern(Kind::Reference {
                    mutable: true,
                    region: RegionId::STATIC,
                    referent: BYTE,
                })?
            };
            let id = facts.types.intern_quantified(Quantified {
                bound_regions: count,
                predicate: Predicate {
                    ty,
                    bound: Bound::Copy,
                },
            })?;
            let clause = Clause::ForAll(id);
            let mut proof = Proof::for_declaration(None, 0, 1);
            let formation = proof.validate_builtin_clause(&mut facts, clause, clause)?;
            if !formation.satisfied {
                assert!(!accepted && count == 0);
                continue;
            }
            facts
                .generics
                .record_builtin_formation(owner, id, formation.subject)?;
            facts.generics.predicates.insert(
                owner,
                vec![sema::generics::Predicate {
                    source,
                    value: clause,
                }],
            );
            let environment = facts
                .generics
                .parameter_environment(&facts.types, owner)?
                .with_clauses(&facts.types, &[clause])?;
            facts
                .generics
                .environments
                .insert(owner, Arc::new(environment));
            let result = sema::bodies::analyze(&report, facts);
            if !accepted {
                let error = result.err().ok_or("invalid builtin call was accepted")?;
                assert_eq!(
                    error.message,
                    "trait obligation requires an unambiguous matching assumption or implementation"
                );
                continue;
            }
            let mut facts = result?;
            let entry = sema::entry::select(&mut report, &facts)?;
            crate::lower::instances::build(&report, &mut facts, entry)?;
        }
    }
    Ok(())
}
