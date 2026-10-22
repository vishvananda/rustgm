//! Raw pointer conversions preserve pointee variance and erase only outer loans.
use crate::native_references as native;
use crate::source_cases as source;

const CASES: &[(&str, bool, &str)] = &[
    ("shared-const", true, "fn f(x:&u8)->*const u8{x}"),
    ("mutable-mut", true, "fn f(x:&mut u8)->*mut u8{x}"),
    ("mutable-const", true, "fn f(x:&mut u8)->*const u8{x}"),
    ("weaken", true, "fn f(x:*mut u8)->*const u8{x}"),
    ("shared-mut", false, "fn f(x:&u8)->*mut u8{x}"),
    ("strengthen", false, "fn f(x:*const u8)->*mut u8{x}"),
    ("raw-array", true, "fn f(x:*const [u8;2])->*const [u8]{x}"),
    ("raw-mut-array", true, "fn f(x:*mut [u8;2])->*mut [u8]{x}"),
    ("reference-array", true, "fn f(x:&[u8;2])->*const [u8]{x}"),
    (
        "reference-mut-array",
        true,
        "fn f(x:&mut [u8;2])->*mut [u8]{x}",
    ),
    (
        "reference-tail",
        true,
        "struct S<T:?Sized>{tail:T}fn f(x:&S<[u8;2]>)->*const S<[u8]>{x}",
    ),
    (
        "reference-mut-tail",
        true,
        "struct S<T:?Sized>{tail:T}fn f(x:&mut S<[u8;2]>)->*mut S<[u8]>{x}",
    ),
    (
        "raw-tail-weaken",
        true,
        "struct S<T:?Sized>{tail:T}fn f(x:*mut S<[u8;2]>)->*const S<[u8]>{x}",
    ),
    (
        "nested-reference",
        false,
        "fn f(x:&&[u8;2])->*const [u8]{x}",
    ),
    ("outer-region", true, "fn f()->*const u8{let x=7;&x}"),
    (
        "embedded-region",
        false,
        "fn f<'a>(x:&'a u8)->*const &'static u8{&x}",
    ),
    (
        "embedded-control",
        true,
        "fn f<'a>(x:&'a u8)->*const &'a u8{&x}",
    ),
    (
        "const-covariance",
        true,
        "fn f<'a,'b:'a>(p:*const &'b u8)->*const &'a u8{p}",
    ),
    (
        "mut-invariance",
        false,
        "fn f<'a,'b:'a>(p:*mut &'b u8)->*mut &'a u8{p}",
    ),
    (
        "reborrow",
        true,
        "fn f(){let mut x=7;let r=&mut x;let p:*mut u8=r;*r=9;let _q=r;let _p=p;}",
    ),
    (
        "reference-tail-region",
        false,
        "struct S<T:?Sized>{tail:T}fn f<'a>(x:&S<[&'a u8;2]>)->*const S<[&'static u8]>{x}",
    ),
    (
        "reference-tail-control",
        true,
        "struct S<T:?Sized>{tail:T}fn f<'a>(x:&S<[&'a u8;2]>)->*const S<[&'a u8]>{x}",
    ),
    (
        "pointer-join",
        true,
        "fn f(ok:bool,a:*const u8,b:*mut u8){let _p=if ok{a}else{b};}",
    ),
    (
        "contextual-join",
        true,
        "fn f(ok:bool,a:&[u8;2],b:&[u8;3])->*const [u8]{if ok{a}else{b}}",
    ),
    (
        "discarded-repeat",
        true,
        "fn f<'a>(x:&'a u8){let _a:[*const &'static u8;0]=[&x;0];}",
    ),
    (
        "thin-equality",
        true,
        "fn f(p:*const u8,q:*const u8)->bool{p==q}",
    ),
    (
        "wide-equality",
        true,
        "fn f(p:*const [u8],q:*const [u8])->bool{p!=q}",
    ),
    (
        "reverse-equality",
        true,
        "fn f(a:*const u8,b:*mut u8)->bool{a==b}",
    ),
    (
        "mixed-equality",
        false,
        "fn f(a:*mut u8,b:*const u8)->bool{a==b}",
    ),
    (
        "pointer-add",
        false,
        "fn f(a:*const u8,b:*const u8){let _p=a+b;}",
    ),
    (
        "const-value",
        true,
        "const P:*const u8=&7;fn f()->*const u8{P}",
    ),
    (
        "const-tail",
        true,
        "struct S<T:?Sized>{tail:T}const P:*const S<[u8]>=&S{tail:[7,9]};fn f()->*const S<[u8]>{P}",
    ),
];

#[test]
fn pointer_coercions_keep_independent_region_and_ownership_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    source::check("pointer-coercions", CASES)
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn pointer_metadata_and_reborrow_execution() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pointer-coercions/native",
        &[
            (
                "thin",
                "fn pass(p:*const u8)->*const u8{p}fn main(){let mut x=7u8;let r=&mut x;let p:*mut u8=r;*r=9;let q:*const u8=&x;check(pass(p)==q);check(x==9);}",
                false,
            ),
            (
                "slice",
                "fn pass(p:*const [u8])->*const [u8]{p}fn main(){let a=[3u8,5,7];let p:*const [u8;3]=&a;let q:*const [u8]=p;let s:&[u8]=&a;let r:*const [u8]=s;check(pass(q)==r);}",
                false,
            ),
            (
                "zst-length",
                "fn main(){let a=[();3];let s:&[()]=&a;let p:*const [()]=s;match s{[_,rest @ ..]=>{let q:*const [()]=rest;check(p!=q);check(p>q);check(p>=q);check(q<p);check(q<=p);},_=>check(false)}}",
                false,
            ),
            (
                "empty-tail",
                "struct S<T:?Sized>{head:u16,tail:T}fn pass(p:*const S<[u8]>)->*const S<[u8]>{p}fn main(){let a=S{head:7,tail:[]};let p:*const S<[u8;0]>=&a;let q:*const S<[u8]>=p;let s:&S<[u8]>=&a;let r:*const S<[u8]>=s;check(pass(q)==r);}",
                false,
            ),
            (
                "mutable-tail",
                "struct S<T:?Sized>{tail:T}fn pass(p:*mut S<[u8]>)->*const S<[u8]>{p}fn main(){let mut a=S{tail:[3u8,5]};let r=&mut a;let p:*mut S<[u8]>=r;r.tail[0]=9;let q:*const S<[u8]>=&a;check(pass(p)==q);check(a.tail[0]==9);}",
                false,
            ),
            (
                "contextual-branch",
                "fn pick(ok:bool,a:&[u8;2],b:&[u8;3])->*const [u8]{if ok{a}else{b}}fn main(){let a=[3u8,5];let b=[7u8,9,11];let p:*const [u8]=&a;let q:*const [u8]=&b;check(pick(true,&a,&b)==p);check(pick(false,&a,&b)==q);}",
                false,
            ),
            (
                "str",
                "fn pass(p:*const str)->*const str{p}fn main(){let s=\"é\";let p:*const str=s;let q:*const str=s;check(pass(p)==q);}",
                false,
            ),
            (
                "bytes",
                "fn main(){let s=b\"abc\";let p:*const [u8]=s;let q:*const [u8]=s;check(p==q);}",
                false,
            ),
            (
                "const-tail",
                "struct S<T:?Sized>{head:u16,tail:T}const P:*const S<[u8]>=&S{head:5,tail:[3,7]};fn pass(p:*const S<[u8]>)->*const S<[u8]>{p}fn main(){let p=P;check(pass(p)==p);}",
                false,
            ),
            (
                "projected-tail",
                "trait A{type Out:?Sized;}impl A for u8{type Out=[u8;2];}impl A for u16{type Out=[u8];}struct S<T:A>{head:u16,tail:T::Out}fn pass(p:*const S<u16>)->*const S<u16>{p}fn main(){let s=S::<u8>{head:5,tail:[3,7]};let p:*const S<u8>=&s;let q:*const S<u16>=p;let r:&S<u16>=&s;let t:*const S<u16>=r;check(pass(q)==t);}",
                false,
            ),
        ],
    )
}
