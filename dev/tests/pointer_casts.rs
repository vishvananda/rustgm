//! Explicit pointer casts retain metadata while allowing raw pointee reinterpretation.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/source_cases.rs"]
mod source;
const CASES: &[(&str, bool, &str)] = &[
    (
        "raw-pointer-cast",
        true,
        "fn f(p:*const u8)->*mut u8{p as *mut u8}",
    ),
    (
        "reference-cast",
        true,
        "fn f(p:&u8)->*const u8{p as *const u8}",
    ),
    (
        "cast-unsizing",
        true,
        "fn f(p:*const [u8;2])->*const [u8]{p as *const [u8]}",
    ),
    (
        "cast-embedded-lifetime",
        true,
        "fn f<'a>(p:*const &'a u8)->*const &'static u8{p as *const &'static u8}",
    ),
    (
        "cast-wide-elements",
        true,
        "fn f(p:*const [u16])->*const [u8]{p as *const [u8]}",
    ),
    (
        "cast-wide-thin",
        true,
        "fn f(p:*const [u8])->*const u8{p as *const u8}",
    ),
    (
        "cast-reference-elements",
        false,
        "fn f(p:&u8)->*const u16{p as *const u16}",
    ),
    (
        "cast-pointer-integer",
        true,
        "fn f(p:*const u8)->usize{p as usize}",
    ),
    (
        "cast-wide-integer",
        false,
        "fn f(p:*const [u8])->usize{p as usize}",
    ),
    (
        "cast-integer-pointer",
        true,
        "fn f(x:usize)->*const u8{x as *const u8}",
    ),
    (
        "cast-integer-wide",
        false,
        "fn f(x:usize)->*const [u8]{x as *const [u8]}",
    ),
    (
        "cast-unsize-strengthen",
        false,
        "fn f(p:*const [u8;2])->*mut [u8]{p as *mut [u8]}",
    ),
    (
        "cast-unsize-lifetime",
        false,
        "fn f<'a>(p:&[&'a u8;2])->*const [&'static u8]{p as *const [&'static u8]}",
    ),
    (
        "cast-unsize-elements",
        false,
        "fn f(p:*const [u16;2])->*const [u8]{p as *const [u8]}",
    ),
    (
        "reference-array-element",
        true,
        "fn f(p:&[u8;2])->*const u8{p as *const u8}",
    ),
    (
        "mutable-array-element",
        true,
        "fn f(p:&mut [u8;2])->*mut u8{p as *mut u8}",
    ),
    (
        "shared-array-mut-element",
        false,
        "fn f(p:&[u8;2])->*mut u8{p as *mut u8}",
    ),
    (
        "reference-array-lifetime",
        false,
        "fn f<'a>(p:&[&'a u8;2])->*const &'static u8{p as *const &'static u8}",
    ),
    (
        "raw-array-lifetime",
        true,
        "fn f<'a>(p:*const [&'a u8;2])->*const &'static u8{p as *const &'static u8}",
    ),
    (
        "generic-thin-cast",
        true,
        "fn f<T>(p:*const T)->*const u8{p as *const u8}",
    ),
    (
        "generic-wide-thin",
        true,
        "fn f<T:?Sized>(p:*const T)->*const u8{p as *const u8}",
    ),
    (
        "generic-unknown-target",
        false,
        "fn f<T:?Sized>(p:*const u8)->*const T{p as *const T}",
    ),
    (
        "generic-metadata-tail",
        true,
        "struct S<T:?Sized>{head:u8,tail:T}fn f<T:?Sized>(p:*const T)->*const S<T>{p as *const S<T>}",
    ),
    (
        "generic-metadata-other",
        false,
        "fn f<T:?Sized,U:?Sized>(p:*const T)->*const U{p as *const U}",
    ),
    (
        "string-slice-cast",
        true,
        "fn f(p:*const str)->*const [u8]{p as *const [u8]}",
    ),
    (
        "slice-string-cast",
        true,
        "fn f(p:*const [u8])->*const str{p as *const str}",
    ),
    (
        "reference-string-slice",
        false,
        "fn f(p:&str)->*const [u8]{p as *const [u8]}",
    ),
    (
        "function-item-pointer",
        true,
        "fn g(){}fn f()->*const u8{g as *const u8}",
    ),
    (
        "function-pointer-pointer",
        true,
        "fn f(g:fn())->*const u8{g as *const u8}",
    ),
    (
        "function-item-integer",
        true,
        "fn g(){}fn f()->usize{g as usize}",
    ),
    (
        "raw-function-pointer",
        false,
        "fn f(p:*const u8)->fn(){p as fn()}",
    ),
    (
        "small-signed-address",
        true,
        "fn f(x:i8)->*const u8{x as *const u8}",
    ),
    (
        "wide-integer-address",
        true,
        "fn f(x:u128)->*const u8{x as *const u8}",
    ),
    (
        "bool-address",
        false,
        "fn f(x:bool)->*const u8{x as *const u8}",
    ),
    (
        "char-address",
        false,
        "fn f(x:char)->*const u8{x as *const u8}",
    ),
    ("pointer-small-int", true, "fn f(x:*const u8)->u8{x as u8}"),
    ("pointer-bool", false, "fn f(x:*const u8)->bool{x as bool}"),
];
#[test]
fn explicit_cast_rules() -> Result<(), Box<dyn std::error::Error>> {
    source::check("pointer-casts", CASES)
}
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn pointer_casts_execute_with_explicit_metadata() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pointer-casts/native",
        &[
            (
                "array-element",
                "fn main(){let mut a=[7u8,9];let r=&mut a;let p=r as *mut u8;r[0]=11;let q:*const u8=&a[0];check(q==p);check(a[0]==11);}",
                false,
            ),
            (
                "wide-thin",
                "fn data<T:?Sized>(p:*const T)->*const u8{p as *const u8}fn main(){let a=[7u8,9];let p:*const [u8]=&a;let q:*const u8=&a[0];check(data(p)==q);check(p as *const u8==q);}",
                false,
            ),
            (
                "metadata-count",
                "fn main(){let a=[7u16,9];let p:*const [u16]=&a;let q=p as *const [u8];let thin=q as *const u8;let array=thin as *const [u8;4];let r:*const [u8]=array;check(q!=r);let two=thin as *const [u8;2];let s:*const [u8]=two;check(q==s);}",
                false,
            ),
            (
                "round-trip",
                "fn main(){let a=7u8;let p:*const u8=&a;let n=p as usize;let q=n as *const u8;check(p==q);}",
                false,
            ),
            (
                "integer-widths",
                "fn main(){let x=-1i8;let p=x as *const u8;check(p as usize==18446744073709551615usize);let n=(1u128<<80)|17;let q=n as *const u8;check(q as usize==17);check(q as u8==17);}",
                false,
            ),
            (
                "functions",
                "fn g(){}fn main(){let f:fn()=g;let p=g as *const u8;let q=f as *const u8;check(p==q);check(p as usize==g as usize);}",
                false,
            ),
            (
                "generic-tail",
                "struct S<T:?Sized>{head:u8,tail:T}fn f<T:?Sized>(p:*const T)->*const S<T>{p as *const S<T>}fn main(){let a=[();3];let p:*const [()]=&a;let q=f(p);check(q as *const [()]==p);}",
                false,
            ),
            (
                "explicit-unsizing",
                "struct S<T:?Sized>{tail:T}fn main(){let a=S{tail:[7u8,9]};let p:*const S<[u8;2]>=&a;let q=p as *const S<[u8]>;let r:*const S<[u8]>=&a;check(q==r);let p=&a.tail as *const [u8];let q:*const [u8]=&a.tail;check(p==q);}",
                false,
            ),
        ],
    )
}
