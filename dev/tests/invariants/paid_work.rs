//! Exact per-selection debt checks must not revisit the growing dispatcher prefix.
use super::*;
use crate::{
    body::regions::Contract,
    resolve::model::DefId,
    types::{
        Kind, Primitive, RegionId, TypeId,
        arguments::{Argument, Constant},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<(Declarations, Contract, DefId)> {
    let report = crate::test_source::resolve(
        "trait V<'a>{const X:u8;}impl<'a,T:'a>V<'a>for T{const X:u8=7;}struct S<'a,T,const N:u8>(&'a u8,T);impl<'a,T:V<'a>,const N:u8>S<'a,T,N>{const C:u8={let n=T::X;n};}fn caller<'a:'b,'b>(_:&'a u8,_:&'b u8){}fn main(){}",
        true,
    )?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let find = |name| {
        report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == name)
            .and_then(|n| u32::try_from(n).ok())
            .map(DefId)
            .ok_or("definition")
    };
    let owner = find("C")?;
    let caller = find("caller")?;
    let contract = crate::body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|c| c.key == crate::body::ir::BodyKey::Declaration(caller))
        .ok_or("caller contract")?;
    Ok((facts, contract, owner))
}
#[test]
fn paid_selection_work_is_linear_for_distinct_and_repeated_roots() -> Result {
    for distinct in [false, true] {
        let mut previous = None;
        for count in [8usize, 32, 128] {
            let (mut facts, contract, owner) = fixture()?;
            let crate::body::ir::BodyKey::Declaration(declaration) = contract.key else {
                return Err("caller".into());
            };
            let key = catalog::CallerKey {
                source: crate::test_sites::node(contract.source)?,
                declaration: Some(declaration),
                universals: contract.universals,
                enclosing: catalog::Enclosing::Original,
            };
            let publication = catalog::Publication::new([0..0, 0..0])?;
            let inputs = Inputs {
                source: crate::test_sites::node(contract.source)?,
                universals: contract.universals,
                signature: contract.signature,
                parameters: &contract.parameters,
                assumptions: &contract.assumptions,
                type_assumptions: &contract.type_assumptions,
                require_result: false,
            };
            inputs.prepare(&mut facts)?;
            let mut dispatch = crate::sema::dispatch::Dispatcher::new(contract.universals, 100)?;
            let opening = dispatch.source_opening(contract.universals)?;
            let reference = facts.types.intern(Kind::Reference {
                region: RegionId::bound(0, 0)?,
                mutable: false,
                referent: TypeId::primitive(Primitive::U8),
            })?;
            let before = facts.body_work.canonical_steps;
            let mut fresh = 0;
            for n in 0..count {
                let constant = facts.types.intern_constant(Constant {
                    ty: TypeId::primitive(Primitive::U8),
                    bits: if distinct { n as u128 } else { 0 },
                })?;
                let input = Instance {
                    definition: owner,
                    arguments: facts.types.intern_arguments(&[
                        Argument::Lifetime(RegionId::bound(0, 1)?),
                        Argument::Type(reference),
                        Argument::Const(constant),
                    ])?,
                };
                let input = dispatch.source_instance(&mut facts, input)?;
                dispatch.source_constant(&mut facts, input, &mut |facts, proof| {
                    fresh += usize::from(proof.input == input && proof.proof.is_some());
                    let (decision, paid) =
                        observe(facts, key, publication, inputs, &opening, proof)?;
                    if paid.is_some() {
                        assert!(matches!(decision, ConstantDecision::Checked(_)));
                    }
                    Ok(decision)
                })?;
            }
            assert_eq!(fresh, count, "a warm identity alone never skips payment");
            let work = facts.body_work.canonical_steps - before;
            assert!(
                work > count && work < 2000 * count,
                "work {work} at {count}"
            );
            if let Some(old) = previous {
                assert!(
                    work <= old * 4 + 128,
                    "growing prefix revalidation: {old} -> {work}"
                );
            }
            previous = Some(work);
            let debts = dispatch.finish();
            assert!(!debts.types.is_empty() && debts.regions.end > 100);
            eprintln!(
                "paid distinct={distinct} count={count} work={work} retained={}",
                debts.retained_allocation().bytes
            );
        }
    }
    Ok(())
}

#[test]
fn declared_outlives_is_not_an_alias_equality_assumption() -> Result {
    let (facts, contract, _) = fixture()?;
    let inputs = Inputs {
        source: crate::test_sites::node(contract.source)?,
        universals: contract.universals,
        signature: contract.signature,
        parameters: &contract.parameters,
        assumptions: &contract.assumptions,
        type_assumptions: &contract.type_assumptions,
        require_result: false,
    };
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut work = 0;
    assert!(
        inputs.aliases(&facts, [(a, b)], &mut work).is_err(),
        "the opposite direction is still a goal"
    );
    assert!(
        work >= 1_000_000,
        "failed bounded solving must retain its conservative charge"
    );
    let mut work = 0;
    inputs.aliases(&facts, [(a, a), (b, b)], &mut work)?;
    assert!(work > 0);
    Ok(())
}
