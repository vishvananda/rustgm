//! Library nominal identities preserve ordinary layout, formation and ownership checks.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "option",
        "fn get(x:Option<u8>)->u8{match x{Some(x)=>x,None=>0}}fn main(){let _=get(Some(7));let _=get(None);}",
        true,
    ),
    (
        "result",
        "fn get(x:Result<u8,u16>)->u16{match x{Ok(x)=>x as u16,Err(x)=>x}}fn main(){let _=get(Ok(7));let _=get(Err(9));}",
        true,
    ),
    (
        "option-alias",
        "use core::option::Option as Maybe;fn get(x:Maybe<u8>)->u8{match x{Maybe::Some(x)=>x,Maybe::None=>0}}fn main(){let _=get(Maybe::Some(7));}",
        true,
    ),
    (
        "option-value-namespace",
        "fn main(){let Option=7u8;let x:Option<u8>=Some(Option);let _=x;}",
        true,
    ),
    (
        "option-shadow",
        "enum Option{Only}fn main(){let _x=Option::Only;}",
        true,
    ),
    (
        "some-shadow",
        "fn Some()->u8{7}fn main(){let _x=Some();}",
        true,
    ),
    (
        "manual-private-literal",
        "fn main(){let _x=core::mem::ManuallyDrop{value:7u16};}",
        false,
    ),
    (
        "manual-private-rest",
        "fn main(){let x=core::mem::ManuallyDrop::new(7u16);let core::mem::ManuallyDrop{..}=x;}",
        true,
    ),
    (
        "manual-not-tuple",
        "fn main(){let x=core::mem::ManuallyDrop::new(7u16);let core::mem::ManuallyDrop(..)=x;}",
        false,
    ),
    (
        "manual-new",
        "fn main(){let _x=core::mem::ManuallyDrop::new(7u16);}",
        true,
    ),
    (
        "manual-private",
        "fn main(){let x=core::mem::ManuallyDrop::new(7u16);let _y=x.value;}",
        false,
    ),
    (
        "manual-private-pattern",
        "fn main(){let x=core::mem::ManuallyDrop::new(7u16);let core::mem::ManuallyDrop{value}=x;let _=value;}",
        false,
    ),
    (
        "manual-lookalike",
        "pub struct ManuallyDrop{pub value:u16}fn main(){let x=ManuallyDrop{value:7};let ManuallyDrop{value}=x;let _=value;}",
        true,
    ),
    (
        "manual-copy",
        "fn main(){let x=core::mem::ManuallyDrop::new(7u16);let _a=x;let _b=x;}",
        true,
    ),
    (
        "manual-not-copy",
        "struct D(u8);fn main(){let x=core::mem::ManuallyDrop::new(D(7));let _a=x;let _b=x;}",
        false,
    ),
    (
        "manual-drop-lifetime",
        "struct D<'a>(&'a u8);impl<'a> Drop for D<'a>{fn drop(&mut self){let _x=*self.0;}}fn main(){let value;{let n=7;value=core::mem::ManuallyDrop::new(D(&n));}}",
        true,
    ),
    (
        "ordinary-drop-lifetime",
        "struct D<'a>(&'a u8);impl<'a> Drop for D<'a>{fn drop(&mut self){let _x=*self.0;}}struct W<T>(T);fn main(){let value;{let n=7;value=W(D(&n));}}",
        false,
    ),
];
#[test]
fn library_nominal_contracts_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("library-nominal-{name}"), source, accepted)?;
        let result = checked(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
fn checked(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn library_nominals_execute_ordinary_shapes() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "library-nominals",
        &[
            (
                "option",
                "fn get(x:Option<u16>)->u16{match x{Some(x)=>x,None=>0}}fn main(){check(get(Some(7))==7);check(get(None)==0);}",
                false,
            ),
            (
                "result",
                "fn get(x:Result<u8,u16>)->u16{match x{Ok(x)=>x as u16,Err(x)=>x}}fn main(){check(get(Ok(7))==7);check(get(Err(9))==9);}",
                false,
            ),
            (
                "manual-lookalike",
                "struct ManuallyDrop<T>(T);struct D<'a>(&'a mut u8);impl<'a> Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}fn main(){let mut n=0;{let _x=ManuallyDrop(D(&mut n));}check(n==1);}",
                false,
            ),
            (
                "manual-drop",
                "struct D<'a>(&'a mut u8);impl<'a> Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}struct W<T>(T);fn main(){let mut n=0;{let _x=W(core::mem::ManuallyDrop::new(D(&mut n)));}check(n==0);{let _x=W(D(&mut n));}check(n==1);}",
                false,
            ),
        ],
    )
}

#[test]
fn metadata_fields_fail_at_the_ordinary_privacy_boundary() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, source, _) in CASES {
        if matches!(
            name,
            "manual-private" | "manual-private-pattern" | "manual-private-literal"
        ) {
            let error = checked(source)
                .err()
                .ok_or("private metadata field was accepted")?;
            assert!(
                error.contains("aggregate field is private"),
                "{name}: {error}"
            );
        }
    }
    Ok(())
}
