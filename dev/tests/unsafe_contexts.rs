//! Unsafe permission is lexical; raw access keeps its separate typed place identity.
use crate::native_references as native;
use crate::semantic;
use crate::source_cases as source;

const CASES: &[(&str, bool, &str)] = &[
    ("unsafe-block-value", true, "fn f()->u8{unsafe{7}}"),
    (
        "unsafe-constant-block",
        true,
        "const C:u8=unsafe{7};fn f()->u8{C}",
    ),
    ("safe-reference", true, "fn f(x:&u8)->u8{*x}"),
    ("raw-read", true, "fn f(x:*const u8)->u8{unsafe{*x}}"),
    ("raw-read-safe", false, "fn f(x:*const u8)->u8{*x}"),
    ("raw-write", true, "fn f(x:*mut u8){unsafe{*x=7;}}"),
    (
        "const-pointer-write",
        false,
        "fn f(x:*const u8){unsafe{*x=7;}}",
    ),
    ("raw-fn-body", true, "unsafe fn f(x:*const u8)->u8{*x}"),
    (
        "unsafe-call",
        true,
        "unsafe fn g()->u8{7}fn f()->u8{unsafe{g()}}",
    ),
    (
        "unsafe-call-safe",
        false,
        "unsafe fn g()->u8{7}fn f()->u8{g()}",
    ),
    (
        "unsafe-fn-caller",
        true,
        "unsafe fn g()->u8{7}unsafe fn f()->u8{g()}",
    ),
    (
        "unsafe-pointer-call",
        true,
        "fn f(g:unsafe fn()->u8)->u8{unsafe{g()}}",
    ),
    (
        "unsafe-pointer-call-safe",
        false,
        "fn f(g:unsafe fn()->u8)->u8{g()}",
    ),
    (
        "unsafe-method",
        true,
        "struct S;impl S{unsafe fn g(&self)->u8{7}}fn f(s:&S)->u8{unsafe{s.g()}}",
    ),
    (
        "unsafe-method-safe",
        false,
        "struct S;impl S{unsafe fn g(&self)->u8{7}}fn f(s:&S)->u8{s.g()}",
    ),
    ("nested-block", true, "fn f(x:*const u8)->u8{unsafe{{*x}}}"),
    (
        "unsafe-does-not-leak",
        false,
        "fn f(x:*const u8)->u8{unsafe{let _=*x;}*x}",
    ),
    (
        "nested-item-reset",
        false,
        "fn f(){unsafe{fn g(x:*const u8)->u8{*x}}}",
    ),
    (
        "nested-unsafe-item",
        true,
        "fn f(){unsafe{unsafe fn g(x:*const u8)->u8{*x}}}",
    ),
    (
        "raw-address-cancel",
        true,
        "fn f(x:*const u8)->*const u8{&raw const *x}",
    ),
    (
        "raw-address-field-needs-unsafe",
        false,
        "struct S{x:u8}fn f(p:*const S)->*const u8{&raw const (*p).x}",
    ),
    (
        "raw-field",
        true,
        "struct S{x:u8}fn f(p:*const S)->u8{unsafe{(*p).x}}",
    ),
    (
        "raw-index",
        true,
        "fn f(p:*const [u8;2])->u8{unsafe{(*p)[1]}}",
    ),
    (
        "raw-slice",
        true,
        "fn f(p:*const [u8])->u8{unsafe{(*p)[1]}}",
    ),
    (
        "raw-to-reference",
        true,
        "fn f<'a>(p:*const u8)->&'a u8{unsafe{&*p}}",
    ),
    (
        "raw-to-mut-reference",
        true,
        "fn f<'a>(p:*mut u8)->&'a mut u8{unsafe{&mut *p}}",
    ),
    (
        "raw-const-to-mut-reference",
        false,
        "fn f<'a>(p:*const u8)->&'a mut u8{unsafe{&mut *p}}",
    ),
    (
        "raw-copy-move",
        false,
        "struct S(u8);fn f(p:*const S)->S{unsafe{*p}}",
    ),
    (
        "raw-uninitialized",
        false,
        "fn f()->u8{let p:*const u8;unsafe{*p}}",
    ),
    (
        "raw-uninitialized-write",
        false,
        "fn f(){let p:*mut u8;unsafe{*p=7;}}",
    ),
    (
        "mutable-static-read-safe",
        false,
        "static mut X:u8=7;fn f()->u8{X}",
    ),
    (
        "mutable-static-read",
        true,
        "static mut X:u8=7;fn f()->u8{unsafe{X}}",
    ),
    (
        "const-mutable-static-read",
        false,
        "static mut X:u8=7;const C:u8=unsafe{X};fn f()->u8{C}",
    ),
    (
        "mutable-static-raw-address",
        true,
        "static mut X:u8=7;fn f()->*const u8{&raw const X}",
    ),
    (
        "mutable-static-raw-field",
        false,
        "struct S{x:u8}static mut X:S=S{x:7};fn f()->*const u8{&raw const X.x}",
    ),
    (
        "mutable-static-raw-field-unsafe",
        true,
        "struct S{x:u8}static mut X:S=S{x:7};fn f()->*const u8{unsafe{&raw const X.x}}",
    ),
    (
        "mutable-static-pointer-read",
        false,
        "static mut X:*const u8=0 as *const u8;fn f()->*const u8{&raw const *X}",
    ),
    (
        "unsafe-type-checks",
        false,
        "fn f(p:*mut u8){unsafe{*p=true;}}",
    ),
    (
        "packed-reference",
        false,
        "#[repr(packed)]struct S{a:u8,b:u32}fn f(s:&S){unsafe{let _=&s.b;}}",
    ),
    (
        "packed-raw-address",
        true,
        "#[repr(packed)]struct S{a:u8,b:u32}fn f(s:&S)->*const u32{&raw const s.b}",
    ),
];

#[test]
fn lexical_unsafe_permission_and_raw_access() -> Result<(), Box<dyn std::error::Error>> {
    source::check("unsafe-contexts", CASES)
}

#[test]
fn raw_places_retain_no_reference_region() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::body::ir::ProjectionKind;
    let (report, facts) = semantic::analyze("fn f(p:*mut u8,q:&u8)->u8{unsafe{*p=*q;*p}}")?;
    let program = rustgm::body::build(&report, &facts)?;
    let mut raw = 0;
    let mut references = 0;
    for body in &program.bodies {
        for projection in &body.projections {
            match projection.kind {
                ProjectionKind::RawDereference { mutable: true } => raw += 1,
                ProjectionKind::Dereference { .. } => references += 1,
                _ => {}
            }
        }
    }
    assert_eq!(raw, 2);
    assert_eq!(references, 1);
    Ok(())
}

#[test]
fn unsafe_permission_does_not_allow_mutable_static_const_reads()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) =
        semantic::analyze("static mut X:u8=7;const C:u8=unsafe{X};fn f()->u8{C}")?;
    let id = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| report.package.names.get(d.name) == "C")
        .map(|(i, _)| u32::try_from(i).map(rustgm::resolve::model::DefId))
        .ok_or("missing const C")??;
    let result = rustgm::sema::values::Evaluator::new(&report, &mut facts).definition(id);
    let Err(error) = result else {
        return Err("accepted mutable static constant read".into());
    };
    assert!(
        error
            .to_string()
            .contains("constant evaluation cannot read mutable static storage")
    );
    Ok(())
}

#[test]
fn unsafe_nesting_and_raw_reads_have_additive_body_work() -> Result<(), Box<dyn std::error::Error>>
{
    for depth in [1, 8, 32] {
        for width in [8, 32, 128] {
            let mut text = String::from("fn f(p:*const u8)->u8{");
            text.push_str(&"unsafe{".repeat(depth));
            text.push_str(&"let _=*p;".repeat(width));
            text.push_str("*p");
            text.push_str(&"}".repeat(depth + 1));
            let (_, facts) = semantic::analyze(&text)?;
            let work = &facts.body_work;
            assert!(work.steps < 32 * (depth + width + 1));
            assert!(work.inference_steps < 128 * (depth + width + 1));
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn raw_access_and_unsafe_calls_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "unsafe-contexts/native",
        &[
            (
                "read-write",
                "fn main(){let mut x=3u8;let p=&raw mut x;unsafe{check(*p==3);*p=9;}check(x==9);}",
                false,
            ),
            (
                "reborrow",
                "unsafe fn get<'a>(p:*mut u8)->&'a mut u8{unsafe{&mut *p}}fn main(){let mut x=3u8;let p=&raw mut x;let r=unsafe{get(p)};*r=9;check(x==9);}",
                false,
            ),
            (
                "function-pointer",
                "unsafe fn get(p:*const u8)->u8{unsafe{*p}}fn main(){let x=9u8;let f:unsafe fn(*const u8)->u8=get;check(unsafe{f(&raw const x)}==9);}",
                false,
            ),
            (
                "wide-slice",
                "fn main(){let a=[3u8,9];let p:*const [u8]=&a;unsafe{check((*p)[1]==9);}}",
                false,
            ),
            (
                "raw-aliases",
                "fn main(){let mut x=3u8;let p=&raw mut x;let q=p;unsafe{*p=9;check(*q==9);}}",
                false,
            ),
        ],
    )
}
