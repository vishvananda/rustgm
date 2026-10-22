//! Canonical volatile calls keep unsafe permission, const eligibility and effects separate.
use crate::native_references as native;
use crate::semantic;
use crate::source_cases as source;

const CASES: &[(&str, bool, &str)] = &[
    (
        "reference-result-lifetime",
        false,
        "fn f<'a>()->&'a u8{let x=7;let r=&x;unsafe{core::ptr::read_volatile(&raw const r)}}",
    ),
    (
        "reference-pointee-lifetime",
        true,
        "unsafe fn f<'a>(p:*const &'a u8)->&'a u8{unsafe{core::ptr::read_volatile(p)}}",
    ),
    (
        "unsafe-loaded-function",
        false,
        "fn f(p:*const unsafe fn(u8)->u8)->u8{let g=unsafe{core::ptr::read_volatile(p)};g(7)}",
    ),
    (
        "unsafe-loaded-function-call",
        true,
        "fn f(p:*const unsafe fn(u8)->u8)->u8{unsafe{let g=core::ptr::read_volatile(p);g(7)}}",
    ),
    (
        "unsafe-read",
        true,
        "fn f(p:*const u32)->u32{unsafe{core::ptr::read_volatile(p)}}",
    ),
    (
        "unsafe-write",
        true,
        "fn f(p:*mut u32){unsafe{std::ptr::write_volatile(p,7);}}",
    ),
    (
        "safe-read",
        false,
        "fn f(p:*const u32)->u32{core::ptr::read_volatile(p)}",
    ),
    (
        "safe-write",
        false,
        "fn f(p:*mut u32){std::ptr::write_volatile(p,7);}",
    ),
    (
        "implicit-unsafe-function",
        true,
        "unsafe fn f(p:*const u32)->u32{core::ptr::read_volatile(p)}",
    ),
    (
        "denied-unsafe-function",
        false,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u32)->u32{core::ptr::read_volatile(p)}",
    ),
    (
        "explicit-unsafe-function",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u32)->u32{unsafe{core::ptr::read_volatile(p)}}",
    ),
    (
        "mutable-read-pointer",
        true,
        "fn f(p:*mut u32)->u32{unsafe{core::ptr::read_volatile(p)}}",
    ),
    (
        "immutable-write-pointer",
        false,
        "fn f(p:*const u32){unsafe{core::ptr::write_volatile(p,7);}}",
    ),
    (
        "pointer-type-mismatch",
        false,
        "fn f(p:*const i32)->u32{unsafe{core::ptr::read_volatile::<u32>(p)}}",
    ),
    (
        "stored-type-mismatch",
        false,
        "fn f(p:*mut u32){unsafe{core::ptr::write_volatile(p,7u16);}}",
    ),
    (
        "import-alias",
        true,
        "use core::ptr::read_volatile as read;fn f(p:*const u32)->u32{unsafe{read(p)}}",
    ),
    (
        "glob-import",
        true,
        "use std::ptr::*;fn f(p:*mut u32)->u32{unsafe{write_volatile(p,7);read_volatile(p)}}",
    ),
    (
        "crate-alias",
        true,
        "extern crate core as c;fn f(p:*const u32)->u32{unsafe{c::ptr::read_volatile(p)}}",
    ),
    (
        "local-name",
        true,
        "fn read_volatile(x:u32)->u32{x+1}fn f()->u32{read_volatile(7)}",
    ),
    (
        "module-shadow",
        true,
        "mod std{pub mod ptr{pub fn read_volatile(x:u32)->u32{x+1}}}fn f()->u32{std::ptr::read_volatile(7)}",
    ),
    (
        "item-formation",
        true,
        "fn f(){let _=core::ptr::read_volatile::<u32>;}",
    ),
    (
        "unsafe-pointer",
        true,
        "fn f()->unsafe fn(*const u32)->u32{core::ptr::read_volatile::<u32>}",
    ),
    (
        "safe-pointer",
        false,
        "fn f()->fn(*const u32)->u32{core::ptr::read_volatile::<u32>}",
    ),
    (
        "pointer-safe-call",
        false,
        "fn f(p:*const u32)->u32{let r:unsafe fn(*const u32)->u32=core::ptr::read_volatile::<u32>;r(p)}",
    ),
    (
        "constant-pointer",
        true,
        "const R:unsafe fn(*const u32)->u32=core::ptr::read_volatile::<u32>;",
    ),
    (
        "const-initializer",
        false,
        "const X:u32=unsafe{core::ptr::read_volatile(&7)};",
    ),
    (
        "const-function",
        false,
        "const fn f(p:*const u32)->u32{unsafe{core::ptr::read_volatile(p)}}",
    ),
    (
        "const-skipped-read",
        false,
        "const fn f(p:*const u32)->u32{if false{unsafe{core::ptr::read_volatile(p)}}else{0}}",
    ),
    (
        "generic-const-function",
        false,
        "const fn f<T>(p:*const T)->T{unsafe{core::ptr::read_volatile(p)}}",
    ),
    (
        "generic-const-skipped-read",
        false,
        "const fn f<T>(p:*const u32)->u32{if false{unsafe{core::ptr::read_volatile(p)}}else{0}}",
    ),
    (
        "uninitialized-callee",
        false,
        "fn f(p:*const u32,b:bool)->u32{let r;if b{r=core::ptr::read_volatile::<u32>;}unsafe{r(p)}}",
    ),
    (
        "const-skipped-write",
        false,
        "const fn f(p:*mut u32){if false{unsafe{core::ptr::write_volatile(p,7);}}}",
    ),
    (
        "no-std",
        true,
        "#![no_std]extern crate std;fn f(p:*const u32)->u32{unsafe{core::ptr::read_volatile(p)}}",
    ),
    (
        "generic-wrapper",
        true,
        "unsafe fn f<T>(p:*mut T,x:T)->T{unsafe{core::ptr::write_volatile(p,x);core::ptr::read_volatile(p)}}",
    ),
    (
        "closure-inheritance",
        true,
        "fn f(){let _=unsafe{||core::ptr::read_volatile(0 as *const u32)};}",
    ),
    (
        "nested-item-boundary",
        false,
        "fn f(){unsafe{fn g(p:*const u32)->u32{core::ptr::read_volatile(p)}}}",
    ),
];

#[test]
fn volatile_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    source::check("parallel/pa28/volatile/source", CASES)
}

#[test]
fn unused_volatile_reads_and_reified_shims_remain_explicit()
-> Result<(), Box<dyn std::error::Error>> {
    use std::{fmt::Write, path::Path, process::Command};
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa28/volatile-effects-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for calls in [1, 8, 32] {
        let mut text = String::from(
            "const R:unsafe fn(*const u32)->u32=core::ptr::read_volatile::<u32>;fn main(){let mut x:u32=7;unsafe{let _=core::ptr::read_volatile(&raw const x);core::ptr::write_volatile(&raw mut x,9);",
        );
        for call in 0..calls {
            write!(
                text,
                "let f{call}:unsafe fn(*const u32)->u32=R;let _=f{call}(&raw const x);"
            )?;
        }
        text.push_str("}}");
        let path = folder.join(format!("{calls}.rs"));
        std::fs::write(&path, &text)?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(path)
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout)?;
        assert_eq!(text.matches("load volatile").count(), 2);
        assert_eq!(text.matches("store volatile").count(), 1);
        assert_eq!(
            text.matches("function @rg.intrinsic.read_volatile").count(),
            1
        );
    }
    Ok(())
}

#[test]
fn volatile_instances_share_canonical_identity_and_additive_body_work()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::support::hash::HashSet;
    use rustgm::{body::ir::Rvalue, support::intrinsics::Intrinsic};
    use std::fmt::Write;
    for callers in [8, 32, 128] {
        let mut text = String::new();
        for caller in 0..callers {
            write!(
                text,
                "fn f{caller}(p:*const u32)->u32{{unsafe{{core::ptr::read_volatile(p)}}}}"
            )?;
        }
        let (report, facts) = semantic::analyze(&text)?;
        // Repeated callers must not register additional canonical definitions.
        assert_eq!(
            facts
                .intrinsics
                .iter()
                .map(|entry| entry.kind)
                .collect::<HashSet<_>>(),
            HashSet::from_iter([
                Intrinsic::ReadVolatile,
                Intrinsic::WriteVolatile,
                Intrinsic::Read,
                Intrinsic::Write,
                Intrinsic::ReadUnaligned,
                Intrinsic::WriteUnaligned,
                Intrinsic::CopyNonoverlapping,
                Intrinsic::Copy,
                Intrinsic::WriteBytes,
            ])
        );
        assert_ne!(
            facts
                .intrinsics
                .get(Intrinsic::ReadVolatile)
                .map(|d| d.definition),
            facts
                .intrinsics
                .get(Intrinsic::WriteVolatile)
                .map(|d| d.definition)
        );
        assert!(facts.body_work.steps < 128 * (callers + 1));
        let program = rustgm::body::build(&report, &facts)?;
        assert_eq!(program.bodies.len(), callers);
        let mut instances = HashSet::default();
        let mut operations = 0;
        for body in &program.bodies {
            for value in &body.values {
                if let Rvalue::Intrinsic {
                    intrinsic,
                    instance,
                    ..
                } = value.kind
                {
                    assert_eq!(intrinsic, Intrinsic::ReadVolatile);
                    assert_eq!(
                        intrinsic.descriptor().constant,
                        rustgm::support::intrinsics::ConstPermission::RuntimeOnly
                    );
                    instances.insert(instance);
                    operations += 1;
                }
            }
        }
        assert_eq!(operations, callers);
        assert_eq!(instances.len(), 1);
    }
    Ok(())
}

const BOUNDARIES: &[(&str, &str)] = &[
    (
        "aggregate-read",
        "#[derive(Copy,Clone)]struct S{x:u32}fn main(){let s=S{x:7};unsafe{let _=core::ptr::read_volatile(&raw const s);}}",
    ),
    (
        "aggregate-write",
        "struct S{x:u32}fn main(){let mut s=S{x:7};unsafe{core::ptr::write_volatile(&raw mut s,S{x:9});}}",
    ),
    (
        "zero-sized-read",
        "fn main(){let x=();unsafe{let _=core::ptr::read_volatile(&raw const x);}}",
    ),
    (
        "wide-pointer-read",
        "fn main(){let a=[7u8,9];let s:&[u8]=&a;unsafe{let _=core::ptr::read_volatile(&raw const s);}}",
    ),
    (
        "aggregate-pointer",
        "#[derive(Copy,Clone)]struct S{x:u32}fn main(){let f:unsafe fn(*const S)->S=core::ptr::read_volatile::<S>;let s=S{x:7};unsafe{let _=f(&raw const s);}}",
    ),
];

#[test]
fn volatile_aggregate_boundaries_are_explicit() -> Result<(), Box<dyn std::error::Error>> {
    use std::{path::Path, process::Command};
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa28/volatile-boundaries-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, text) in BOUNDARIES {
        source::qualify_at(&folder.join("rustc"), name, text, true)?;
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, text)?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(input)
            .output()?;
        assert_eq!(output.status.code(), Some(1), "{name}");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("volatile operations are not implemented"),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn scalar_volatile_operations_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "parallel/pa28/volatile/native",
        &[
            (
                "function-pointer-value",
                "fn inc(x:u8)->u8{x+1}fn main(){let p:fn(u8)->u8=inc;unsafe{let f=core::ptr::read_volatile(&raw const p);check(f(7)==8);}}",
                false,
            ),
            (
                "mutable-reference-value",
                "fn main(){let mut x:u8=7;let mut y:u8=11;let mut r=&mut x;unsafe{core::ptr::write_volatile(&raw mut r,&mut y);let p=core::ptr::read_volatile(&raw const r);*p=17;}check(y==17);}",
                false,
            ),
            (
                "read-write",
                "fn main(){let mut x:u32=3;let p=&raw mut x;unsafe{core::ptr::write_volatile(p,17);check(core::ptr::read_volatile(p)==17);}}",
                false,
            ),
            (
                "function-pointers",
                "const R:unsafe fn(*const u32)->u32=core::ptr::read_volatile::<u32>;fn main(){let mut x:u32=3;let w:unsafe fn(*mut u32,u32)=core::ptr::write_volatile::<u32>;unsafe{w(&raw mut x,23);check(R(&raw const x)==23);}}",
                false,
            ),
            (
                "integer-widths",
                "unsafe fn replace<T>(p:*mut T,v:T)->T{unsafe{core::ptr::write_volatile(p,v);core::ptr::read_volatile(p)}}fn main(){let mut a:i8=0;let mut b:u16=0;let mut c:i32=0;let mut d:u64=0;let mut e:u128=0;unsafe{check(replace(&raw mut a,-7)==-7);check(replace(&raw mut b,65001)==65001);check(replace(&raw mut c,-123456)==-123456);check(replace(&raw mut d,18446744073709551610)==18446744073709551610);check(replace(&raw mut e,340282366920938463463374607431768211450)==340282366920938463463374607431768211450);}}",
                false,
            ),
            (
                "float-and-validity-scalars",
                "fn main(){let mut b=false;let mut c='a';let mut x=1.0f32;let mut y=2.0f64;unsafe{core::ptr::write_volatile(&raw mut b,true);core::ptr::write_volatile(&raw mut c,'z');core::ptr::write_volatile(&raw mut x,3.5f32);core::ptr::write_volatile(&raw mut y,7.25f64);check(core::ptr::read_volatile(&raw const b));check(core::ptr::read_volatile(&raw const c)=='z');check(core::ptr::read_volatile(&raw const x)==3.5f32);check(core::ptr::read_volatile(&raw const y)==7.25f64);}}",
                false,
            ),
            (
                "pointer-values",
                "fn main(){let x:u8=7;let y:u8=11;let mut p=&raw const x;unsafe{core::ptr::write_volatile(&raw mut p,&raw const y);let q=core::ptr::read_volatile(&raw const p);check(*q==11);}}",
                false,
            ),
            (
                "reference-value",
                "fn main(){let x:u8=7;let r=&x;unsafe{let s=core::ptr::read_volatile(&raw const r);check(*s==7);}}",
                false,
            ),
            (
                "unused-result-effect",
                "fn main(){let mut x:u8=0;unsafe{core::ptr::write_volatile(&raw mut x,29);let _=core::ptr::read_volatile(&raw const x);}check(x==29);}",
                false,
            ),
        ],
    )
}
