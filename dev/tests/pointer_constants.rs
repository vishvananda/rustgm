//! Pointer constants retain storage identity and reject CTFE address observation.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/source_cases.rs"]
mod source;
const CASES: &[(&str, bool, &str)] = &[
    (
        "absolute-unsize",
        true,
        "const A:*const [u8;2]=42usize as *const [u8;2];const P:*const [u8]=A;fn f()->*const [u8]{P}",
    ),
    (
        "null-unsize",
        true,
        "const A:*const [u8;0]=0usize as *const [u8;0];const P:*const [u8]=A;fn f()->*const [u8]{P}",
    ),
    (
        "text-remetadata",
        true,
        "const S:*const str=\"abc\";const A:*const [u8;2]=S as *const [u8;2];const P:*const [u8]=A;fn f()->*const [u8]{P}",
    ),
    (
        "bytes-remetadata",
        true,
        "const S:*const [u8;3]=b\"abc\";const A:*const [u8;2]=S as *const [u8;2];const P:*const [u8]=A;fn f()->*const [u8]{P}",
    ),
    (
        "raw-null-address",
        true,
        "const P:*const u8=0usize as *const u8;const Q:*const u8=&raw const *P;fn f()->*const u8{Q}",
    ),
    (
        "inferred-pointer-element",
        true,
        "fn f(x:&[u8;2])->*const u8{x as *const _}",
    ),
    (
        "constant-null",
        true,
        "const P:*const u8=0usize as *const u8;fn f()->*const u8{P}",
    ),
    (
        "constant-nonnull",
        true,
        "const P:*const u8=42usize as *const u8;fn f()->*const u8{P}",
    ),
    (
        "constant-pointer-int",
        false,
        "const P:usize=&7u8 as *const u8 as usize;fn f()->usize{P}",
    ),
    (
        "constant-address-int",
        false,
        "const P:usize=42usize as *const u8 as usize;fn f()->usize{P}",
    ),
    (
        "constant-raw-static",
        true,
        "static X:u8=7;const P:*const u8=&raw const X;fn f()->*const u8{P}",
    ),
    (
        "constant-raw-temporary",
        false,
        "const P:*const u8=&raw const 7u8;fn f()->*const u8{P}",
    ),
    (
        "constant-pointer-mut",
        true,
        "const P:*mut u8=&7u8 as *const u8 as *mut u8;fn f()->*mut u8{P}",
    ),
    (
        "constant-wide-thin",
        true,
        "const P:*const u8=(&[7u8,9] as *const [u8]) as *const u8;fn f()->*const u8{P}",
    ),
    (
        "constant-string-thin",
        true,
        "const P:*const u8=(\"abc\" as *const str) as *const u8;fn f()->*const u8{P}",
    ),
    (
        "constant-pointer-compare",
        false,
        "const P:*const u8=&7;const B:bool=P==P;fn f()->bool{B}",
    ),
    (
        "constant-pointer-dead-integer",
        false,
        "const P:*const u8=&7;const N:usize=if false{P as usize}else{0};fn f()->usize{N}",
    ),
    (
        "constant-pointer-dead-comparison",
        false,
        "const P:*const u8=&7;const N:bool=if false{P==P}else{true};fn f()->bool{N}",
    ),
    (
        "inferred-reference-cast",
        true,
        "fn f(x:&u8)->*const u8{x as _}",
    ),
    (
        "inferred-unsizing-cast",
        true,
        "fn f(x:*const [u8;2])->*const [u8]{x as _}",
    ),
    (
        "inferred-element-cast",
        true,
        "fn f(x:&[u8;2])->*const u8{x as _}",
    ),
    (
        "constant-raw-reference",
        true,
        "const R:&u8=&7;const P:*const u8=&raw const *R;fn f()->*const u8{P}",
    ),
    (
        "constant-raw-packed",
        true,
        "#[repr(packed)]struct S{a:u8,b:u16}static X:S=S{a:7,b:9};const P:*const u16=&raw const X.b;fn f()->*const u16{P}",
    ),
];
#[test]
fn pointer_constants_and_cast_inference() -> Result<(), Box<dyn std::error::Error>> {
    source::check("pointer-constants", CASES)
}
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn pointer_constants_execute_with_backing_and_metadata() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pointer-constants/native",
        &[
            (
                "absolute-metadata",
                "const A:*const [u8;2]=42usize as *const [u8;2];const P:*const [u8]=A;const B:*const [u8;3]=42usize as *const [u8;3];const Q:*const [u8]=B;fn main(){check(P!=Q);check(P<Q);check(P as *const u8 as usize==42);check(Q as *const u8 as usize==42);}",
                false,
            ),
            (
                "null-metadata",
                "const A:*const [u8;0]=0usize as *const [u8;0];const P:*const [u8]=A;const B:*const [u8;1]=0usize as *const [u8;1];const Q:*const [u8]=B;const Z:*const u8=&raw const *(0usize as *const u8);fn main(){check(P!=Q);check(P as *const u8==Z);check(Q as *const u8==Z);}",
                false,
            ),
            (
                "text-metadata",
                "const S:*const str=\"abc\";const A:*const [u8;2]=S as *const [u8;2];const P:*const [u8]=A;const Q:*const [u8]=S as *const [u8];fn main(){check(P!=Q);check(P<Q);check(P as *const u8==Q as *const u8);}",
                false,
            ),
            (
                "bytes-metadata",
                "const S:*const [u8;3]=b\"abc\";const A:*const [u8;2]=S as *const [u8;2];const P:*const [u8]=A;const Q:*const [u8]=S;fn main(){check(P!=Q);check(P<Q);check(P as *const u8==Q as *const u8);}",
                false,
            ),
            (
                "absolute",
                "const P:*const u8=42usize as *const u8;const Z:*const u8=0usize as *const u8;fn main(){check(P as usize==42);check(Z as usize==0);}",
                false,
            ),
            (
                "static",
                "static X:u8=7;const P:*const u8=&raw const X;fn main(){let q=&raw const X;check(P==q);}",
                false,
            ),
            (
                "packed",
                "#[repr(packed)]struct S{a:u8,b:u16}static X:S=S{a:7,b:9};const P:*const u16=&raw const X.b;fn main(){let q=&raw const X.b;check(P==q);}",
                false,
            ),
            (
                "wide-thin",
                "const W:*const [u8]=&[7u8,9] as *const [u8];const P:*const u8=W as *const u8;fn main(){check(P==W as *const u8);}",
                false,
            ),
            (
                "string-thin",
                "const W:*const str=\"é\" as *const str;const P:*const u8=W as *const u8;fn main(){check(P==W as *const u8);}",
                false,
            ),
            (
                "metadata",
                "const W:*const [u16]=&[7u16,9];const P:*const [u8]=W as *const [u8];fn main(){check(P==W as *const [u8]);}",
                false,
            ),
            (
                "tail",
                "struct S<T:?Sized>{head:u16,tail:T}static X:S<[u8;2]>=S{head:3,tail:[5,7]};const P:*const S<[u8]>=(&raw const X) as *const S<[u8]>;fn main(){let q:*const S<[u8]>=&X;check(P==q);}",
                false,
            ),
            (
                "inference",
                "fn f(x:&[u8;2])->*const [u8]{x as _}fn g(x:&[u8;2])->*const u8{x as _}fn main(){let a=[7u8,9];let p:*const [u8]=&a;let q:*const u8=&a[0];check(f(&a)==p);check(g(&a)==q);}",
                false,
            ),
        ],
    )
}
