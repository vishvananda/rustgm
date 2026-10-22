//! Mutable static storage retains declaration permission without reference promises.
use crate::native_references as native;
use crate::semantic;
use crate::source_cases as source;

const CASES: &[(&str, bool, &str)] = &[
    (
        "raw-mut-reference-store",
        true,
        "fn f(p:*mut &mut u8){unsafe{**p=7;}}",
    ),
    (
        "raw-const-reference-store",
        false,
        "fn f(p:*const &mut u8){unsafe{**p=7;}}",
    ),
    (
        "raw-mut-shared-reference-store",
        false,
        "fn f(p:*mut &u8){unsafe{**p=7;}}",
    ),
    ("store", true, "static mut X:u8=0;fn f(){unsafe{X=7;}}"),
    ("store-safe", false, "static mut X:u8=0;fn f(){X=7;}"),
    (
        "immutable-store",
        false,
        "static X:u8=0;fn f(){unsafe{X=7;}}",
    ),
    (
        "field-store",
        true,
        "struct S{x:u8}static mut X:S=S{x:0};fn f(){unsafe{X.x=7;}}",
    ),
    (
        "immutable-field-store",
        false,
        "struct S{x:u8}static X:S=S{x:0};fn f(){unsafe{X.x=7;}}",
    ),
    (
        "index-store",
        true,
        "static mut X:[u8;2]=[0,0];fn f(){unsafe{X[1]=7;}}",
    ),
    (
        "compound-store",
        true,
        "static mut X:u8=0;fn f(){unsafe{X+=7;}}",
    ),
    (
        "unsafe-function-store",
        true,
        "static mut X:u8=0;unsafe fn f(){X=7;}",
    ),
    (
        "raw-mut-address",
        true,
        "static mut X:u8=0;fn f()->*mut u8{&raw mut X}",
    ),
    (
        "raw-mut-immutable",
        false,
        "static X:u8=0;fn f()->*mut u8{&raw mut X}",
    ),
    (
        "shared-reference",
        false,
        "static mut X:u8=0;fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "mutable-reference",
        false,
        "static mut X:u8=0;fn f()->&'static mut u8{unsafe{&mut X}}",
    ),
    (
        "field-reference",
        false,
        "struct S{x:u8}static mut X:S=S{x:0};fn f()->&'static u8{unsafe{&X.x}}",
    ),
    (
        "method-reference",
        false,
        "struct S;impl S{fn g(&self){}}static mut X:S=S;fn f(){unsafe{X.g();}}",
    ),
    (
        "unused-reference",
        false,
        "static mut X:u8=0;fn f(){unsafe{let _=&X;}}",
    ),
    (
        "unreachable-reference",
        false,
        "static mut X:u8=0;fn f(){unsafe{if false{let _=&X;}}}",
    ),
    (
        "immutable-reference",
        true,
        "static X:u8=0;fn f()->&'static u8{&X}",
    ),
    (
        "ref-binding",
        false,
        "static mut X:u8=0;fn f(){unsafe{let ref r=X;let _=r;}}",
    ),
    (
        "ref-mut-binding",
        false,
        "static mut X:u8=0;fn f(){unsafe{let ref mut r=X;let _=r;}}",
    ),
    (
        "paren-ref-binding",
        false,
        "static mut X:u8=0;fn f(){unsafe{let (ref r)=X;let _=r;}}",
    ),
    (
        "let-else-ref-binding",
        false,
        "static mut X:u8=0;fn f(){unsafe{let ref r=X else{return;};let _=r;}}",
    ),
    (
        "match-ref-lint-control",
        true,
        "static mut X:u8=0;fn f(){unsafe{match X{ref r=>{let _=r;}}}}",
    ),
    (
        "struct-ref-lint-control",
        true,
        "struct S{x:u8}static mut X:S=S{x:0};fn f(){unsafe{let S{x:ref r}=X;let _=r;}}",
    ),
    (
        "if-let-ref-lint-control",
        true,
        "static mut X:u8=0;fn f(){unsafe{if let ref r=X{let _=r;}}}",
    ),
    (
        "guard-copy",
        true,
        "static mut X:u8=0;fn f(){unsafe{match X{x if x==0=>(),_=>()}}}",
    ),
    (
        "destructor-store",
        true,
        "static mut N:u8=0;struct S(u8);impl Drop for S{fn drop(&mut self){unsafe{N+=1;}}}static mut X:S=S(3);fn f(){unsafe{X=S(9);}}",
    ),
    (
        "raw-mut-field",
        true,
        "struct S{x:u8}static mut X:S=S{x:0};fn f()->*mut u8{unsafe{&raw mut X.x}}",
    ),
];

#[test]
fn static_storage_and_default_reference_lint() -> Result<(), Box<dyn std::error::Error>> {
    source::check("unsafe-statics", CASES)
}

#[test]
fn mutable_static_permissions_are_shared_across_unrelated_bodies()
-> Result<(), Box<dyn std::error::Error>> {
    use std::{fmt::Write, sync::Arc};
    for statics in [1, 8, 32] {
        for callers in [8, 32, 128] {
            let mut text = String::new();
            for index in 0..statics {
                writeln!(text, "static mut S{index}:u8=0;")?;
            }
            for index in 0..callers {
                writeln!(text, "fn f{index}(){{unsafe{{S0=7;}}}}")?;
            }
            let (report, facts) = semantic::analyze(&text)?;
            let declared = facts
                .mutable_statics
                .as_ref()
                .ok_or("missing static permissions")?;
            assert_eq!(declared.len(), statics);
            let program = rustgm::body::build(&report, &facts)?;
            assert_eq!(program.bodies.len(), statics + callers);
            for body in &program.bodies {
                let permissions = body
                    .mutable_statics
                    .as_ref()
                    .ok_or("missing body permissions")?;
                assert!(Arc::ptr_eq(declared, permissions));
            }
            assert!(facts.body_work.steps < 32 * (statics + callers + 1));
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn static_stores_and_raw_addresses_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "unsafe-statics/native",
        &[
            (
                "destructor-store",
                "static mut N:u8=0;struct S(u8);impl Drop for S{fn drop(&mut self){unsafe{N+=1;}}}static mut X:S=S(3);fn main(){unsafe{X=S(9);check(N==1);check(X.0==9);}}",
                false,
            ),
            (
                "direct-store",
                "static mut X:u8=3;fn main(){unsafe{X=9;check(X==9);X+=2;check(X==11);}}",
                false,
            ),
            (
                "field-store",
                "struct S{x:u8,y:u8}static mut X:S=S{x:3,y:5};fn main(){unsafe{X.x=9;check(X.x==9);check(X.y==5);}}",
                false,
            ),
            (
                "indexed-store",
                "static mut X:[u8;2]=[3,5];fn main(){unsafe{X[1]=9;check(X[0]==3);check(X[1]==9);}}",
                false,
            ),
            (
                "raw-mut-address",
                "static mut X:u8=3;fn main(){let p=&raw mut X;unsafe{*p=9;check(X==9);}}",
                false,
            ),
        ],
    )
}
