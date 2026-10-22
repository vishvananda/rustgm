//! Union storage overlap, field validity gates and lexical access rules.
use crate::native_references as native;
use crate::semantic;
use crate::source_cases as source;

const CASES: &[(&str, bool, &str)] = &[
    (
        "unsafe-read",
        true,
        "union U{x:u32,y:u32}fn f(u:U)->u32{unsafe{u.x}}",
    ),
    (
        "safe-read",
        false,
        "union U{x:u32,y:u32}fn f(u:U)->u32{u.x}",
    ),
    (
        "safe-write",
        true,
        "union U{x:u32,y:u32}fn f(mut u:U){u.x=7;}",
    ),
    (
        "safe-compound",
        false,
        "union U{x:u32,y:u32}fn f(mut u:U){u.x+=7;}",
    ),
    (
        "unsafe-compound",
        true,
        "union U{x:u32,y:u32}fn f(mut u:U){unsafe{u.x+=7;}}",
    ),
    ("single-init", true, "union U{x:u32,y:u32}fn f()->U{U{x:7}}"),
    ("empty-init", false, "union U{x:u32,y:u32}fn f()->U{U{}}"),
    (
        "multiple-init",
        false,
        "union U{x:u32,y:u32}fn f()->U{U{x:7,y:8}}",
    ),
    (
        "update-init",
        false,
        "union U{x:u32,y:u32}fn f(u:U)->U{U{x:7,..u}}",
    ),
    ("empty-union", false, "union U{}"),
    ("nondrop-noncopy-field", false, "struct S(u8);union U{x:S}"),
    (
        "copy-field",
        true,
        "#[derive(Copy,Clone)]struct S(u8);union U{x:S}",
    ),
    (
        "mut-reference-field",
        true,
        "union U<'a>{x:&'a mut u32,y:u32}",
    ),
    (
        "tuple-mut-reference-field",
        true,
        "union U<'a>{x:(&'a mut u32,),y:u32}",
    ),
    (
        "array-mut-reference-field",
        true,
        "union U<'a>{x:[&'a mut u32;2],y:u32}",
    ),
    (
        "manuallydrop-field",
        true,
        "struct S(u8);union U{x:std::mem::ManuallyDrop<S>,y:u32}",
    ),
    (
        "fake-manuallydrop",
        false,
        "struct ManuallyDrop<T>(T);struct S(u8);union U{x:ManuallyDrop<S>}",
    ),
    ("copy-generic", true, "union U<T:Copy>{x:T,y:u32}"),
    ("unbound-generic", false, "union U<T>{x:T,y:u32}"),
    ("c-layout", true, "#[repr(C)]union U{x:u32,y:u32}"),
    (
        "packed-layout",
        true,
        "#[repr(C,packed)]union U{x:u32,y:u32}",
    ),
    (
        "packed-field-reference",
        false,
        "#[repr(C,packed)]union U{x:u32,y:u8}fn f(u:&U)->&u32{unsafe{&u.x}}",
    ),
    (
        "packed-raw-field-address",
        true,
        "#[repr(C,packed)]union U{x:u32,y:u8}fn f(u:&U)->*const u32{&raw const u.x}",
    ),
    (
        "transparent-layout",
        false,
        "#[repr(transparent)]union U{x:u32}",
    ),
    (
        "borrow-safe",
        false,
        "union U{x:u32,y:u32}fn f(u:&U)->&u32{&u.x}",
    ),
    (
        "borrow-unsafe",
        true,
        "union U{x:u32,y:u32}fn f(u:&U)->&u32{unsafe{&u.x}}",
    ),
    (
        "raw-address-safe",
        true,
        "union U{x:u32,y:u32}fn f(u:&U)->*const u32{&raw const u.x}",
    ),
    (
        "uninit-field-write",
        false,
        "union U{x:u32,y:u32}fn f()->u32{let mut u:U;u.x=7;unsafe{u.y}}",
    ),
    (
        "uninit-field-read",
        false,
        "union U{x:u32,y:u32}fn f()->u32{let u:U;unsafe{u.x}}",
    ),
    (
        "nested-field-write",
        true,
        "#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,x:u32}fn f(mut u:U){u.s.x=7;}",
    ),
    (
        "array-element-write",
        false,
        "union U{a:[u32;2],x:u32}fn f(mut u:U){u.a[0]=7;}",
    ),
    (
        "different-field-borrow",
        false,
        "union U{x:u32,y:u32}fn f(u:&mut U){unsafe{let x=&mut u.x;let y=&mut u.y;*x=7;*y=9;}}",
    ),
    (
        "same-field-subfield-disjoint",
        true,
        "#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,t:S}fn f(u:&mut U){unsafe{let x=&mut u.s.x;let y=&mut u.s.y;*x=7;*y=9;}}",
    ),
    (
        "different-field-subfield-overlap",
        false,
        "#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,t:S}fn f(u:&mut U){unsafe{let x=&mut u.s.x;let y=&mut u.t.y;*x=7;*y=9;}}",
    ),
    (
        "union-move-field",
        true,
        "union U<'a>{x:&'a mut u32,y:u32}fn f<'a>(u:U<'a>)->&'a mut u32{unsafe{u.x}}",
    ),
    (
        "union-move-reuse",
        false,
        "union U<'a>{x:&'a mut u32,y:u32}fn f<'a>(u:U<'a>){unsafe{let x=u.x;let y=u.x;*x=7;*y=9;}}",
    ),
    (
        "pattern-safe",
        false,
        "union U{x:u32,y:u32}fn f(u:U)->u32{let U{x}=u;x}",
    ),
    (
        "pattern-unsafe",
        true,
        "union U{x:u32,y:u32}fn f(u:U)->u32{unsafe{let U{x}=u;x}}",
    ),
    (
        "pattern-two-fields",
        false,
        "union U{x:u32,y:u32}fn f(u:U){unsafe{let U{x,y}=u;}}",
    ),
    (
        "pattern-rest",
        false,
        "union U{x:u32,y:u32}fn f(u:U)->u32{unsafe{let U{x,..}=u;x}}",
    ),
    (
        "invalid-bool-const",
        false,
        "union U{x:u8,b:bool}const X:bool=unsafe{U{x:3}.b};struct S([u8;X as usize]);",
    ),
    (
        "uninitialized-byte-const",
        false,
        "union U{x:u8,y:u32}const X:u32=unsafe{U{x:1}.y};struct S([u8;X as usize]);",
    ),
    (
        "raw-nested-field-safe",
        false,
        "#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,x:u32}fn f(u:&U)->*const u32{&raw const u.s.x}",
    ),
    (
        "raw-array-element-safe",
        false,
        "union U{a:[u32;2],x:u32}fn f(u:&U)->*const u32{&raw const u.a[0]}",
    ),
    (
        "index-field-write-unsafe",
        true,
        "union U{a:[u32;2],x:u32}fn f(mut u:U){unsafe{u.a[0]=7;}}",
    ),
    (
        "uninit-field-write-unsafe",
        false,
        "union U{x:u32,y:u32}fn f()->u32{let mut u:U;unsafe{u.x=7;u.y}}",
    ),
    (
        "wildcard-pattern-safe",
        true,
        "union U{x:u32,y:u32}fn f(u:U){let U{x:_}=u;}",
    ),
    (
        "reference-field-store",
        false,
        "struct S{x:u32}union U<'a>{r:&'a mut S,x:u32}fn f(mut u:U){u.r.x=7;}",
    ),
    (
        "tuple-reference-field-store",
        false,
        "struct S{x:u32}union U<'a>{r:(&'a mut S,),x:u32}fn f(mut u:U){u.r.0.x=7;}",
    ),
    (
        "clone-without-copy",
        false,
        "#[derive(Clone)]union U{x:u32,y:u32}",
    ),
    (
        "clone-with-copy",
        true,
        "#[derive(Copy,Clone)]union U{x:u32,y:u32}",
    ),
    (
        "derive-equality",
        false,
        "#[derive(PartialEq)]union U{x:u32,y:u32}",
    ),
    (
        "uninhabited-union-match",
        false,
        "#[derive(Copy,Clone)]enum E{}union U{e:E}fn f(u:U)->u32{match u{}}",
    ),
    (
        "uninhabited-union-wildcard",
        true,
        "#[derive(Copy,Clone)]enum E{}union U{e:E}fn f(u:U)->u32{match u{_=>7}}",
    ),
    (
        "packed-aligned-union",
        false,
        "#[derive(Copy,Clone)]#[repr(align(8))]struct A(u8);#[repr(packed)]union U{a:A,x:u8}",
    ),
    (
        "packed-aligned-union-container",
        false,
        "#[derive(Copy,Clone)]#[repr(align(8))]union U{x:u8,y:u32}#[repr(packed)]struct S{u:U}",
    ),
    (
        "grouped-wildcard-safe",
        true,
        "union U{x:u32,y:u32}fn f(u:U){let U{x:(_)}=u;}",
    ),
    (
        "nested-wildcard-safe",
        false,
        "#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,x:u32}fn f(u:U){let U{s:S{x:_,y:_}}=u;}",
    ),
    (
        "uninit-wildcard-safe",
        true,
        "union U{x:u32,y:u32}fn f(){let u:U;let U{x:_}=u;}",
    ),
    (
        "raw-uninit-union-address",
        false,
        "union U{x:u32,y:u32}fn f(){let u:U;let p=&raw const u.x;let _=p;}",
    ),
    (
        "destructor-field-move",
        false,
        "union U<'a>{r:&'a mut u32,x:u32}impl Drop for U<'_>{fn drop(&mut self){}}fn f<'a>(u:U<'a>)->&'a mut u32{unsafe{u.r}}",
    ),
];

// Retain the unused forms independently: the frontend currently fails to demand
// these invalid initializers. They are release blockers, not passing validity cases.
const INVALID_CONST_INITIALIZERS: &[(&str, bool, &str)] = &[
    (
        "unused-invalid-bool-const",
        false,
        "union U{x:u8,b:bool}const X:bool=unsafe{U{x:3}.b};",
    ),
    (
        "unused-uninitialized-byte-const",
        false,
        "union U{x:u8,y:u32}const X:u32=unsafe{U{x:1}.y};",
    ),
];

#[test]
fn unused_union_constants_are_independently_invalid() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, expected, text) in INVALID_CONST_INITIALIZERS {
        source::qualify(name, text, expected)?;
    }
    Ok(())
}

#[test]
fn union_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    source::check("unsafe-unions", CASES)
}

#[test]
fn union_field_checks_and_construction_scale_additively() -> Result<(), Box<dyn std::error::Error>>
{
    use rustgm::body::ir::{ProjectionKind, Rvalue};
    use std::fmt::Write;
    for fields in [1, 32, 256] {
        for callers in [8, 32, 128] {
            let mut text = String::from("union U{");
            for field in 0..fields {
                write!(text, "f{field}:u32,")?;
            }
            text.push('}');
            for caller in 0..callers {
                write!(
                    text,
                    "fn f{caller}()->u32{{let u=U{{f0:7}};unsafe{{u.f{}}}}}",
                    fields - 1
                )?;
            }
            let (report, facts) = semantic::analyze(&text)?;
            assert_eq!(facts.union_work.fields, fields);
            assert_eq!(facts.union_work.steps, fields);
            assert_eq!(facts.union_work.unique_types, 1);
            assert!(facts.body_work.steps < 64 * (fields + callers + 1));
            let program = rustgm::body::build(&report, &facts)?;
            assert_eq!(program.bodies.len(), callers);
            for body in &program.bodies {
                assert!(body.arguments.is_empty());
                assert_eq!(
                    body.values
                        .iter()
                        .filter(|v| matches!(v.kind, Rvalue::Union { .. }))
                        .count(),
                    1
                );
                assert_eq!(
                    body.projections
                        .iter()
                        .filter(|p| matches!(p.kind, ProjectionKind::UnionField(_)))
                        .count(),
                    1
                );
            }
        }
    }
    Ok(())
}

#[test]
fn union_scalar_constant_initialization_executes() -> Result<(), Box<dyn std::error::Error>> {
    source::check(
        "parallel/pa27/union-scalar-constant",
        &[(
            "valid-bool",
            true,
            "union U{x:u8,b:bool}const X:bool=unsafe{U{x:1}.b};",
        )],
    )?;
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn union_storage_executes() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "unsafe-unions/native",
        &[
            (
                "shared-bytes",
                "#[repr(C)]union U{x:u32,b:[u8;4]}fn main(){let mut u=U{x:16909060};unsafe{check(u.b[0]==4);check(u.b[1]==3);u.b=[8,7,6,5];check(u.x==84281096);}}",
                false,
            ),
            (
                "float-bits",
                "#[repr(C)]union U{x:u32,y:f32}fn main(){let u=U{x:1065353216};unsafe{check(u.y==1.0);}}",
                false,
            ),
            (
                "value-transfer",
                "union U{x:u32,y:u32}fn make()->U{U{x:17}}fn take(u:U)->u32{unsafe{u.y}}fn main(){check(take(make())==17);}",
                false,
            ),
            (
                "raw-field-address",
                "union U{x:u32,y:u32}fn main(){let mut u=U{x:3};let p=&raw mut u.x;unsafe{*p=19;check(u.y==19);}}",
                false,
            ),
            (
                "union-destructor",
                "static mut N:u8=0;union U{x:u32,y:u32}impl Drop for U{fn drop(&mut self){unsafe{N+=1;}}}fn main(){{let mut u=U{x:3};u.y=9;unsafe{check(N==0);check(u.x==9);}}unsafe{check(N==1);}}",
                false,
            ),
        ],
    )
}
