//! Builtin declarations form scoped subjects without borrowing their own proof assumptions.
use super::{
    Argument, BYTE, Bound, Clause, Declarations, Kind, Predicate, Proof, RegionId, TestResult,
    TypeId, nominal, quantified, quantified_tests, tests,
};
use crate::rustc;
use crate::types::parameters::environment::Environment;
use std::sync::Arc;

fn reference(
    facts: &mut Declarations,
    ty: TypeId,
    region: RegionId,
    mutable: bool,
) -> TestResult<TypeId> {
    Ok(facts.types.intern(Kind::Reference {
        mutable,
        region,
        referent: ty,
    })?)
}

#[test]
fn builtin_declarations_defer_nonempty_quantifiers_without_assuming_closed_bounds() -> TestResult {
    for bound in [Bound::Copy, Bound::Clone, Bound::Sized] {
        let (_, mut facts) = tests::declarations("")?;
        let invalid = if bound == Bound::Sized {
            TypeId::primitive(crate::types::Primitive::Str)
        } else {
            reference(&mut facts, BYTE, RegionId::STATIC, true)?
        };
        for (ty, valid) in [(BYTE, true), (invalid, false)] {
            let predicate = Predicate { ty, bound };
            let environment = Arc::new(
                Environment::new(&facts.types, &[])?.with_predicates(&facts.types, &[predicate])?,
            );
            for count in [None, Some(0), Some(1), Some(8)] {
                let clause = if let Some(count) = count {
                    quantified(&mut facts, count, predicate)?
                } else {
                    Clause::Predicate(predicate)
                };
                let mut proof = Proof::for_declaration(Some(Arc::clone(&environment)), 0, 1);
                assert_eq!(
                    proof
                        .validate_builtin_clause(&mut facts, clause, clause)?
                        .satisfied,
                    valid || count.is_some_and(|count| count != 0),
                    "{bound:?} {count:?}"
                );
                let mut use_proof = Proof::for_declaration(None, 0, 1);
                assert_eq!(use_proof.prove_clause(&mut facts, clause)?, valid);
            }
        }
    }
    Ok(())
}

#[test]
fn builtin_formation_defers_a_local_subject_without_authorizing_its_nominal_type() -> TestResult {
    for supplied in [false, true] {
        let implementation = if supplied { "impl A for u8{}" } else { "" };
        let (report, mut facts) =
            tests::declarations(&format!("trait A{{}}struct N<T:A>(T);{implementation}"))?;
        let nominal = nominal(
            &mut facts,
            tests::definition(&report, "N")?,
            &[Argument::Type(BYTE)],
        )?;
        let local = reference(&mut facts, nominal, RegionId::bound(0, 0)?, false)?;
        for ty in [nominal, local] {
            for bound in [Bound::Copy, Bound::Clone, Bound::Sized] {
                let clause = quantified(&mut facts, 1, Predicate { ty, bound })?;
                let mut proof = Proof::for_declaration(None, 0, 1);
                let result = proof.validate_builtin_clause(&mut facts, clause, clause);
                assert_eq!(result.is_ok(), supplied || ty == local);
                if supplied || ty == local {
                    assert!(result?.satisfied);
                    assert_eq!(proof.check(&mut facts, nominal).is_ok(), supplied);
                } else {
                    assert_eq!(
                        result.err(),
                        Some("nominal type argument must implement its declared trait")
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn builtin_source_controls_qualify_formation_and_definition_rules() -> TestResult {
    for (bound, ty) in [
        ("Copy", "&'static mut u8"),
        ("Clone", "&'static mut u8"),
        ("Sized", "str"),
    ] {
        for (binder, used, accepted) in [
            ("", false, false),
            ("for<>", false, false),
            ("for<'a>", false, true),
            ("for<'a>", true, false),
        ] {
            rustc::qualify(
                &format!("builtin-{bound}-{}-{used}", binder.len()),
                &format!(
                    "fn f()where {binder}{ty}:{bound}{{}}fn main(){{{}}}",
                    if used { "f();" } else { "" }
                ),
                accepted,
            )?;
        }
    }
    for (name, subject, extra, accepted) in [
        ("local", "&'a N<u8>", "", true),
        ("local-called", "&'a N<u8>", "fn g(){f();}", true),
        ("independent", "N<u8>", "", false),
        ("actual-type", "&'a N<u8>", "fn g(_:N<u8>){}", false),
        ("malformed-local", "(str,&'a u8)", "", true),
        ("callable-local", "fn(&'a N<u8>)", "", true),
        ("callable-independent", "for<'b>fn(&'b N<u8>)", "", false),
    ] {
        rustc::qualify(
            &format!("builtin-formation-{name}"),
            &format!(
                "trait A{{}}struct N<T:A>(T);fn f()where for<'a>{subject}:Copy{{}}{extra}fn main(){{}}"
            ),
            accepted,
        )?;
    }
    for (name, subject, extra, accepted) in [
        ("local", "&'a &'x T", "", true),
        ("captured", "&'x T", "", false),
        ("captured-given", "&'x T", ",T:'x", true),
    ] {
        rustc::qualify(
            &format!("builtin-body-{name}"),
            &format!("fn f<'x,T>()where for<'a>{subject}:Copy{extra}{{}}fn main(){{}}"),
            accepted,
        )?;
    }
    Ok(())
}

#[test]
fn builtin_projection_formation_retains_its_universal_lifetime_proof() -> TestResult {
    for (implementation, valid) in [
        ("impl<'a>A<'a>for u8{type Out=&'a u8;}", true),
        ("impl A<'static>for u8{type Out=&'static u8;}", false),
    ] {
        for count in [0, 1, 8] {
            let (report, mut facts) =
                tests::declarations(&format!("trait A<'a>{{type Out;}}{implementation}"))?;
            let region = RegionId::bound(u16::from(count == 0), 0)?;
            let ty = super::terms::projected(&mut facts, &report, region)?;
            let clause = quantified(
                &mut facts,
                count,
                Predicate {
                    ty,
                    bound: Bound::Copy,
                },
            )?;
            let mut proof = Proof::for_declaration(None, 1, 19);
            assert!(
                proof
                    .validate_builtin_clause(&mut facts, clause, clause)?
                    .satisfied
            );
            let evidence = proof
                .take_trait_proof()
                .ok_or("missing scoped builtin formation")?;
            assert_eq!(quantified_tests::valid(&facts, &evidence, 1)?, valid);
        }
    }
    Ok(())
}

#[test]
fn builtin_formation_rejects_missing_scope_and_keeps_local_shape_deferral_separate() -> TestResult {
    let (_, mut facts) = tests::declarations("")?;
    for count in [0, 1] {
        let clause = quantified(
            &mut facts,
            count,
            Predicate {
                ty: BYTE,
                bound: Bound::Copy,
            },
        )?;
        assert!(
            Proof::new(None)
                .validate_builtin_clause(&mut facts, clause, clause)
                .is_err()
        );
    }
    let string = TypeId::primitive(crate::types::Primitive::Str);
    let local = reference(&mut facts, BYTE, RegionId::bound(0, 0)?, false)?;
    let list = facts.types.intern_list(&[string, local])?;
    let ty = facts.types.intern(Kind::Tuple(list))?;
    let clause = quantified(
        &mut facts,
        1,
        Predicate {
            ty,
            bound: Bound::Copy,
        },
    )?;
    let mut proof = Proof::for_declaration(None, 0, 1);
    assert!(
        proof
            .validate_builtin_clause(&mut facts, clause, clause)?
            .satisfied
    );
    assert!(!proof.check(&mut facts, ty)?.well_formed);
    Ok(())
}

#[test]
fn builtin_formations_share_repeated_growing_subjects() -> TestResult {
    for (width, captured) in [
        (16, false),
        (128, false),
        (1024, false),
        (16, true),
        (128, true),
        (1024, true),
    ] {
        let (_, mut facts) = tests::declarations("")?;
        let mut ty = BYTE;
        let mut clauses = Vec::new();
        let mut proof = Proof::for_declaration(None, u32::from(captured), 1);
        for _ in 0..width {
            ty = reference(
                &mut facts,
                ty,
                RegionId::bound(u16::from(captured), 0)?,
                false,
            )?;
            let clause = quantified(
                &mut facts,
                1,
                Predicate {
                    ty,
                    bound: Bound::Copy,
                },
            )?;
            assert!(
                proof
                    .validate_builtin_clause(&mut facts, clause, clause)?
                    .satisfied
            );
            clauses.push(clause);
        }
        let first = proof.formation_steps();
        for clause in clauses {
            assert!(
                proof
                    .validate_builtin_clause(&mut facts, clause, clause)?
                    .satisfied
            );
        }
        let repeated = proof.formation_steps() - first;
        eprintln!(
            "builtin formation width={width} captured={captured} first={first} repeated={repeated}"
        );
        assert!(first < 32 * width + 32);
        assert!(repeated <= 2 * width);
    }
    Ok(())
}
