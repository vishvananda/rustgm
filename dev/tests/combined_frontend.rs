//! Parallel frontend features share body identities, demand readiness and cleanup facts.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::resolve as source;

const CASES: &[(&str, &str, bool)] = &[
    (
        "const-and-opaque",
        "const fn size(n:usize)->usize{n+1}type A=[u8;size(1)];const B:usize=size(2);fn opaque()->impl Copy{3u8}fn main(){let _=opaque();let a:A=[7;2];check(a[1]==7);check(B==3);}",
        false,
    ),
    (
        "closure-constant",
        "const fn value()->u8{7}const A:u8=value();fn main(){let f=|x:u8|x+A;check(f(2)==9);}",
        false,
    ),
    (
        "closure-pointer-constant",
        "const fn value()->u8{7}const A:u8=value();fn main(){let f:fn(u8)->u8=|x|x+A;check(f(2)==9);}",
        false,
    ),
    (
        "closure-static-constant",
        "const fn value()->u8{7}static mut X:u8=value();fn main(){let f=||unsafe{X+=1;X};check(f()==8);check(f()==9);}",
        false,
    ),
    (
        "const-option",
        "const fn some(n:u8)->Option<u8>{Some(n)}const A:Option<u8>=some(7);fn main(){let n=match A{Some(x)=>x,None=>0};check(n==7);}",
        false,
    ),
    (
        "closure-result",
        "fn main(){let f=|x:u8|->Result<u8,u8>{Ok(x+1)};let n=match f(7){Ok(x)=>x,Err(x)=>x};check(n==8);}",
        false,
    ),
];

#[test]
fn independently_qualified_features_share_the_checked_pipeline()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, _) in CASES {
        let text = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("combined-{name}"), &text, true)?;
        let mut report = source::resolve(&text, true)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    }
    Ok(())
}

#[test]
fn opaque_scheduling_reuses_early_constant_function_bodies()
-> Result<(), Box<dyn std::error::Error>> {
    let text = "const fn size(n:usize)->usize{n+1}type A=[u8;size(1)];const B:usize=size(2);fn opaque()->impl Copy{3u8}fn main(){let _=opaque();let _:A=[7;2];let _=B;}";
    let mut report = source::resolve(text, true)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    assert_eq!(facts.body_work.cfg_functions.get(), 3);
    Ok(())
}

#[test]
#[ignore = "requires the configured CPPGM native backend"]
fn combined_features_execute_their_checked_source_bodies() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases("parallel/integration-constants", CASES)
}
