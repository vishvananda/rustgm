use super::*;

#[test]
fn universal_requirements_use_fresh_scopes_and_never_grant_assumptions() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let a = RegionId::bound(0, 0)?;
    let output = f.projection(11, t, &[Argument::Lifetime(a)])?;
    let universal = f.clause(1, output, a)?;
    for universal_assumption in [false, true] {
        let (mut proof, mut queries) = f.proof(1)?;
        if universal_assumption {
            proof.assume_outlives_clause(&mut queries, universal)?;
        } else {
            proof.assume_type_outlives(&mut queries, output, a)?;
        }
        proof.require_outlives_clause(&mut queries, universal, 0)?;
        proof.require_outlives_clause(&mut queries, universal, 1)?;
        assert_eq!(proof.solve().is_ok(), universal_assumption);
    }
    let required = f.clause(1, t, a)?;
    let (mut proof, mut queries) = f.proof(1)?;
    proof.require_outlives_clause(&mut queries, required, 0)?;
    assert!(proof.solve().is_err());
    Ok(())
}

#[test]
fn cached_owner_match_does_not_supply_a_fresh_universal_input() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let a = RegionId::bound(0, 0)?;
    let actual = f.projection(11, t, &[Argument::Lifetime(a)])?;
    let pattern = f.projection(11, t, &[Argument::Lifetime(RegionId::bound(1, 0)?)])?;
    let premise = f.clause(1, pattern, RegionId::STATIC)?;
    let universal = f.clause(1, actual, RegionId::STATIC)?;
    let (mut proof, mut queries) = f.proof(1)?;
    proof.assume_outlives_clause(&mut queries, premise)?;
    proof.require_type(&mut queries, actual, RegionId::STATIC, ProofScope::ROOT, 0)?;
    proof.require_outlives_clause(&mut queries, universal, 1)?;
    assert!(proof.solve().is_err());
    Ok(())
}

#[test]
fn empty_required_binders_preserve_captured_targets() -> Result {
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let a = RegionId::bound(0, 0)?;
    let required = f.clause(0, t, RegionId::bound(1, 0)?)?;
    let (mut proof, mut queries) = f.proof(1)?;
    proof.assume_type_outlives(&mut queries, t, a)?;
    proof.require_outlives_clause(&mut queries, required, 0)?;
    proof.solve()?;
    Ok(())
}

fn callable(f: &mut Fixture, region: RegionId) -> Result<TypeId> {
    let reference = f.reference(region, TypeId::UNIT)?;
    let parameters = f.types.intern_list(&[reference])?;
    Ok(f.types.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
    })?)
}

#[test]
fn outer_premise_slots_cannot_capture_nested_callable_locals() -> Result {
    // This is a typed scope invariant. Generic rustc late-outlives matching has
    // known permissive cases; those alone are not evidence for concrete calls.
    let mut f = Fixture::default();
    let t = f.parameter()?;
    let captured = callable(&mut f, RegionId::bound(1, 0)?)?;
    let local = callable(&mut f, RegionId::bound(0, 0)?)?;
    let pattern = f.projection(11, t, &[Argument::Type(captured)])?;
    let actual = f.projection(11, t, &[Argument::Type(local)])?;
    let clause = f.clause(1, pattern, RegionId::STATIC)?;
    let (mut proof, mut queries) = f.proof(1)?;
    proof.assume_outlives_clause(&mut queries, clause)?;
    proof.require_type(&mut queries, actual, RegionId::STATIC, ProofScope::ROOT, 0)?;
    assert!(proof.solve().is_err());
    let local_clause = f.clause(1, actual, RegionId::STATIC)?;
    let (mut proof, mut queries) = f.proof(0)?;
    proof.assume_outlives_clause(&mut queries, local_clause)?;
    proof.require_type(&mut queries, actual, RegionId::STATIC, ProofScope::ROOT, 0)?;
    proof.solve()?;
    Ok(())
}
