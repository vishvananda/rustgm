use crate::semantic;
use rustgm::{body, borrow, lower, sema};

fn lower(source: &str) -> Result<rustgm::lowir::program::Program, String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let contracts = body::regions::collect(&facts)?;
    let proof = borrow::regions::check_bodies(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )
    .map_err(|e| e.to_string())?;
    let checked = borrow::verify(
        &mut borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            copies: &mut facts.copies,
        },
        &proof,
        &program,
    )
    .map_err(|e| e.to_string())?;
    let entry = sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    lower::build(&mut facts, &checked, entry).map_err(|e| e.message.to_owned())
}

#[test]
fn unused_constructor_templates_never_enter_concrete_ownership() -> Result<(), String> {
    for source in [
        "struct S<T>(T);fn main(){}",
        "enum E<T,const N:usize>{A(T),B([u8;N])}fn main(){}",
        "struct S<T>{value:T}fn main(){let s=S{value:7u8};let _:u8=s.value;}",
    ] {
        lower(source)?;
    }
    Ok(())
}

#[test]
fn callable_constructor_execution_keeps_its_explicit_instance_boundary() {
    for source in [
        "struct S<T>(T);fn main(){let _s=S(1u8);}",
        "struct S<T>(T);fn main(){let f:fn(u8)->S<u8>=S;let _s=f(1u8);}",
    ] {
        let result = lower(source);
        assert_eq!(
            result.err().as_deref(),
            Some("generic callable execution requires demanded instances")
        );
    }
}

use crate::native_references as native;
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn closed_record_instances_match_independent_native_execution()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "record-instance-native",
        &[
            (
                "unused-template",
                "struct S<T>(T);fn main(){check(true);}",
                false,
            ),
            (
                "closed-record",
                "struct S<T,const N:usize>{v:[T;N]}fn main(){let s=S{v:[3u16,5,9]};check(s.v[2]==9);}",
                false,
            ),
        ],
    )
}
