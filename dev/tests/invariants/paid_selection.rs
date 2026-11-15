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
        let proof = requirements(&mut facts, owner, &mut substitution, None, 2, first)?
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

#[test]
fn an_opaque_callable_first_interned_by_an_instance_is_stored_before_its_premise_is_read() -> Result
{
    // `drive` promises its callable's output under a caller lifetime, so its
    // instance reads the callable's closure leaf. That callable is `compose`'s
    // opaque result, and its application with the actual closures first exists
    // when `total` is instantiated.
    let text =
        crate::test_source::fixture("pa31/tests/spec/user-callable-output/struct-output.rs")?
            + "fn compose<T,B,Acc>(mut f:impl FnMut(T)->B,mut g:impl FnMut(Acc,B)->Acc)\
           ->impl FnMut(Acc,T)->Acc{move|acc,value|g(acc,f(value))}\n\
           fn drive<'a,B,F>(first:&'a i32,second:&'a i32,init:B,mut f:F)->B \
           where F:FnMut(B,&'a i32)->B{let acc=f(init,first);f(acc,second)}\n\
           fn total<'a,F,G>(first:&'a i32,second:&'a i32,f:F,g:G)->i32 \
           where F:FnMut(&'a i32)->i32,G:FnMut(i32,i32)->i32{\
           drive(first,second,0,compose(f,g))}\n\
           #[unsafe(no_mangle)]pub extern \"C\" fn run()->i32{\
           let first=1;let second=2;total(&first,&second,|value|*value,|_,value|value)}\n";
    let report = crate::test_source::source_provider(&text, "opaque_callable_premise")?;
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
    )
    .map_err(|error| error.to_string())?;
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "run" && symbol.body.is_some()),
        "the exported caller was not lowered"
    );
    Ok(())
}
