//! Raw addresses check the selected place without acquiring a lasting loan.
use crate::native_references as native;
use crate::source_cases as source;

const CASES: &[(&str, bool, &str)] = &[
    (
        "uninitialized",
        false,
        "fn f()->*const u8{let x:u8;&raw const x}",
    ),
    (
        "packed",
        true,
        "#[repr(packed)]struct P{a:u8,b:u16}fn f(x:&P)->*const u16{&raw const x.b}",
    ),
    ("mutable", true, "fn f()->*mut u8{let mut x=7;&raw mut x}"),
    (
        "immutable-mut",
        false,
        "fn f()->*mut u8{let x=7;&raw mut x}",
    ),
    ("temporary", false, "fn f()->*const u8{&raw const 7}"),
    (
        "temporary-field",
        false,
        "struct S{a:u8}fn f()->*const u8{&raw const (S{a:7}).a}",
    ),
    (
        "constant-item",
        false,
        "const X:u8=7;fn f()->*const u8{&raw const X}",
    ),
    (
        "shared-live-unique",
        false,
        "fn f(){let mut x=7;let r=&mut x;let p=&raw const x;*r=9;let _q=p;}",
    ),
    (
        "mutable-live-shared",
        false,
        "fn f(){let mut x=7;let r=&x;let p=&raw mut x;let _y=*r;let _q=p;}",
    ),
    (
        "shared-live-shared",
        true,
        "fn f(){let x=7;let r=&x;let p=&raw const x;let _y=*r;let _q=p;}",
    ),
    (
        "no-lasting-loan",
        true,
        "fn f(){let mut x=7;let p=&raw mut x;x=9;let _q=p;let _v=x;}",
    ),
    (
        "raw-deref",
        true,
        "fn f(p:*const u8)->*const u8{&raw const *p}",
    ),
    (
        "raw-mut-deref",
        true,
        "fn f(p:*mut u8)->*mut u8{&raw mut *p}",
    ),
    (
        "raw-weaken",
        true,
        "fn f(p:*mut [u8])->*const [u8]{&raw const (*p)}",
    ),
    (
        "raw-strengthen",
        false,
        "fn f(p:*const u8)->*mut u8{&raw mut *p}",
    ),
    (
        "raw-field",
        false,
        "struct S{a:u8}fn f(p:*const S)->*const u8{&raw const (*p).a}",
    ),
    ("raw-read", false, "fn f(p:*const u8)->u8{*p}"),
    (
        "reference-deref",
        true,
        "fn f(p:&u8)->*const u8{&raw const *p}",
    ),
    (
        "shared-reference-mut",
        false,
        "fn f(p:&u8)->*mut u8{&raw mut *p}",
    ),
    (
        "unique-reference-mut",
        true,
        "fn f(p:&mut u8)->*mut u8{&raw mut *p}",
    ),
    (
        "embedded-region",
        false,
        "fn f<'a>(x:&'a u8)->*const &'static u8{&raw const x}",
    ),
    (
        "embedded-control",
        true,
        "fn f<'a>(x:&'a u8)->*const &'a u8{&raw const x}",
    ),
];

#[test]
fn raw_addresses_observe_place_mutability_and_existing_loans()
-> Result<(), Box<dyn std::error::Error>> {
    source::check("pointer-addresses", CASES)
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn raw_addresses_preserve_metadata_and_packed_places() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pointer-addresses/native",
        &[
            (
                "local",
                "fn main(){let mut x=7u8;let p=&raw mut x;x=9;let q=&raw const x;check(q==p);check(x==9);}",
                false,
            ),
            (
                "packed",
                "#[repr(packed)]struct P{a:u8,b:u16}fn main(){let mut p=P{a:3,b:7};let x=&raw const p.b;let y=&raw mut p.b;p.b=9;check(x==y);check(p.b==9);}",
                false,
            ),
            (
                "wide",
                "fn echo(p:*const [u8])->*const [u8]{&raw const *p}fn main(){let a=[3u8,5];let s:&[u8]=&a;let p=&raw const *s;let q:*const [u8]=s;check(echo(p)==q);}",
                false,
            ),
            (
                "tail",
                "struct S<T:?Sized>{head:u16,tail:T}fn main(){let a=S{head:3,tail:[5u8,7]};let p=&raw const a;let q:*const S<[u8]>=p;let s:&S<[u8]>=&a;let r=&raw const *s;check(q==r);let t=&raw const s.tail;let u:*const [u8]=&s.tail;check(t==u);}",
                false,
            ),
            (
                "unique-reference",
                "fn main(){let mut x=7u8;let r=&mut x;let p=&raw mut *r;*r=9;let q=&raw const x;check(q==p);check(x==9);}",
                false,
            ),
        ],
    )
}
