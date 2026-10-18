use super::*;

#[test]
fn correlated_and_independent_targets_keep_different_meanings() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let subject = f.projection(11, t, &[Argument::Lifetime(a)])?;
    for (target, expected_static) in [(a, false), (b, true)] {
        let clause = f.clause(2, subject, target)?;
        for shorter in [a, RegionId::STATIC] {
            let (mut proof, mut queries) = f.proof(1)?;
            proof.assume_outlives_clause(&mut queries, clause)?;
            proof.require_type(&mut queries, subject, shorter, ProofScope::ROOT, 42)?;
            assert_eq!(proof.solve().is_ok(), shorter == a || expected_static);
        }
        let (mut proof, mut queries) = f.proof(1)?;
        proof.assume_outlives_clause(&mut queries, clause)?;
        proof.require_type(&mut queries, t, RegionId::STATIC, ProofScope::ROOT, 42)?;
        assert!(
            proof.solve().is_err(),
            "opaque output must not prove its Self input"
        );
    }
    Ok(())
}

#[test]
fn repeated_slots_require_identical_bindings_even_with_mutual_outlives() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let pattern = f.projection(11, t, &[Argument::Lifetime(a), Argument::Lifetime(a)])?;
    let clause = f.clause(1, pattern, RegionId::STATIC)?;
    let distinct = f.projection(11, t, &[Argument::Lifetime(a), Argument::Lifetime(b)])?;
    for (actual, expected) in [(pattern, true), (distinct, false)] {
        let (mut proof, mut queries) = f.proof(2)?;
        proof.assume(a, b)?;
        proof.assume(b, a)?;
        proof.assume_outlives_clause(&mut queries, clause)?;
        proof.require_type(&mut queries, actual, RegionId::STATIC, ProofScope::ROOT, 0)?;
        assert_eq!(proof.solve().is_ok(), expected);
    }
    Ok(())
}

#[test]
fn captured_inputs_are_conditions_and_failed_alternatives_do_not_equate_them() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let pattern = f.projection(11, t, &[Argument::Lifetime(RegionId::bound(1, 0)?)])?;
    let clause = f.clause(1, pattern, RegionId::STATIC)?;
    let actual = f.projection(11, t, &[Argument::Lifetime(b)])?;
    for equal in [false, true] {
        let (mut proof, mut queries) = f.proof(2)?;
        proof.assume_outlives_clause(&mut queries, clause)?;
        if equal {
            proof.assume(a, b)?;
            proof.assume(b, a)?;
        }
        proof.require_type(&mut queries, actual, RegionId::STATIC, ProofScope::ROOT, 0)?;
        assert_eq!(proof.solve().is_ok(), equal);
    }
    let own = f.projection(11, t, &[Argument::Lifetime(a)])?;
    let (mut proof, mut queries) = f.proof(2)?;
    proof.assume_outlives_clause(&mut queries, clause)?;
    proof.require_type(&mut queries, own, RegionId::STATIC, ProofScope::ROOT, 0)?;
    proof.outlives(a, b, 1)?;
    assert!(proof.solve().is_err());
    Ok(())
}

#[test]
fn member_and_complete_rigid_inputs_are_part_of_identity() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let u = f.parameter()?;
    let a = RegionId::bound(0, 0)?;
    let pattern = f.projection(11, t, &[Argument::Lifetime(a), Argument::Type(t)])?;
    let clause = f.clause(1, pattern, RegionId::STATIC)?;
    let different_input = f.projection(11, t, &[Argument::Lifetime(a), Argument::Type(u)])?;
    let different_self = f.projection(11, u, &[Argument::Lifetime(a), Argument::Type(t)])?;
    let different_member = f.projection(12, t, &[Argument::Lifetime(a), Argument::Type(t)])?;
    let Kind::Projection(mut application) = f.types.kind(pattern).ok_or("missing projection")?
    else {
        return Err("not a projection".into());
    };
    application.arguments = f.types.intern_arguments(&[Argument::Type(u)])?;
    let different_family = f.types.intern(Kind::Projection(application))?;
    for actual in [
        pattern,
        different_input,
        different_self,
        different_member,
        different_family,
    ] {
        let (mut proof, mut queries) = f.proof(1)?;
        proof.assume_outlives_clause(&mut queries, clause)?;
        proof.require_type(&mut queries, actual, RegionId::STATIC, ProofScope::ROOT, 0)?;
        assert_eq!(proof.solve().is_ok(), actual == pattern);
    }
    Ok(())
}
