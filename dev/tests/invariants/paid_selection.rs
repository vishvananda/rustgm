//! Raw implementation debts remain goals until actual caller inputs pay them.
use super::*;
use crate::{
    sema::values::Instance,
    types::{Kind, Primitive, TypeId, outlives::Queries, relations::Constraints},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn raw_declared_premise_selection_does_not_assume_its_own_lifetime_debt() -> Result {
    let report = crate::test_source::resolve(
        "trait V<'a>{const X:u8;}impl<'a,T:'a>V<'a>for T{const X:u8=7;}struct S<'a,T>(&'a u8,T);impl<'a,T:V<'a>>S<'a,T>{const C:u8={let n=T::X;n};}fn main(){}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let owner = DefId(u32::try_from(
        report
            .resolution
            .definitions
            .iter()
            .position(|definition| report.package.names.get(definition.name) == "C")
            .ok_or("constant owner")?,
    )?);
    let a = universal(0)?;
    let b = universal(1)?;
    for (first, paid, actual) in [
        (100, true, a),
        (200, false, a),
        (300, false, RegionId::STATIC),
    ] {
        let reference = facts.types.intern(Kind::Reference {
            region: actual,
            mutable: false,
            referent: TypeId::primitive(Primitive::U8),
        })?;
        let instance = Instance {
            definition: owner,
            arguments: facts
                .types
                .intern_arguments(&[Argument::Lifetime(b), Argument::Type(reference)])?,
        };
        let mut substitution = instance.substitution(&facts)?;
        let proof = requirements(&mut facts, owner, &mut substitution, 2, first)?
            .ok_or("declared premise did not select an implementation")?;
        assert_eq!(proof.regions.start, first);
        assert!(proof.regions.end >= first + 2);
        assert!(
            !proof.types.is_empty(),
            "selection must retain T:'a as a goal"
        );
        let mut constraints =
            Constraints::new(&facts.types, &facts.lifetimes.variances, 2, 1_000_000)?;
        if paid {
            constraints.assume(a, b)?;
        }
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        proof.require(&mut constraints, &mut queries)?;
        assert_eq!(
            constraints.solve().is_ok(),
            paid || actual == RegionId::STATIC
        );
        assert!(
            !proof.types.is_empty(),
            "solving cannot consume the retained caller debt"
        );
    }
    Ok(())
}
