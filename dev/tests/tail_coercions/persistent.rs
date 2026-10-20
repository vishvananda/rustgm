//! Constant pointer metadata preserves the original backing allocation and field offsets.
use super::native;

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn constant_tails_keep_lengths_locations_and_promotions() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "pa22-native/persistent-tails",
        &[
            (
                "const",
                "struct S<T:?Sized>{tag:u8,tail:T}\nconst C:&S<[u8]>=&S{tag:7,tail:[3,5]};fn main(){check(C.tag==7);check(C.tail[1]==5);}",
                false,
            ),
            (
                "static",
                "struct S<T:?Sized>{tag:u8,tail:T}\nstatic C:&S<[u8]>=&S{tag:7,tail:[3,5]};fn main(){check(C.tag==7);check(C.tail[1]==5);}",
                false,
            ),
            (
                "static-base",
                "struct S<T:?Sized>{tag:u8,tail:T}\nstatic B:S<[u8;2]>=S{tag:7,tail:[3,5]};const C:&S<[u8]>=&B;fn main(){check(C.tag==7);check(C.tail[1]==5);}",
                false,
            ),
            (
                "promotion",
                "struct S<T:?Sized>{tag:u8,tail:T}\nfn make()->&'static S<[u8]>{&S{tag:7,tail:[3,5]}}fn main(){check(make().tag==7);check(make().tail[1]==5);}",
                false,
            ),
            (
                "associated",
                "struct S<T:?Sized>{tag:u8,tail:T}\ntrait A{const C:&'static S<[u8]>;}impl A for u8{const C:&'static S<[u8]>=&S{tag:7,tail:[3,5]};}fn main(){check(u8::C.tag==7);check(u8::C.tail[1]==5);}",
                false,
            ),
            (
                "generic-const",
                "struct S<T:?Sized>{tag:u8,tail:T}\nstruct G<const N:usize>;impl<const N:usize>G<N>{const C:&'static S<[u8]>=&S{tag:7,tail:[5;N]};}fn main(){check(G::<2>::C.tag==7);match &G::<2>::C.tail{[a,b]=>check(*a==5&&*b==5),_=>check(false)}}",
                false,
            ),
            (
                "read-field",
                "struct S<T:?Sized>{tag:u8,tail:T}\nconst C:&S<[u8]>=&S{tag:7,tail:[3,5]};const N:u8=C.tail[1];fn main(){check(N==5);}",
                false,
            ),
            (
                "tail-reference",
                "struct S<T:?Sized>{tag:u8,tail:T}\nconst C:&S<[u8]>=&S{tag:7,tail:[3,5]};const T:&[u8]=&C.tail;fn main(){check(T[1]==5);match T{[_,_]=>(),_=>check(false)}}",
                false,
            ),
            (
                "nested-tail-reference",
                "struct S<T:?Sized>{tag:u8,tail:T}struct W<T:?Sized>{prefix:u32,tail:S<T>}static B:W<[u8;2]>=W{prefix:17,tail:S{tag:7,tail:[3,5]}};const C:&W<[u8]>=&B;const T:&S<[u8]>=&C.tail;fn main(){check(C.prefix==17);check(T.tag==7);check(T.tail[1]==5);}",
                false,
            ),
            (
                "aggregate-promotion",
                "struct S<T:?Sized>{tag:u8,tail:T}struct R{p:&'static S<[u8]>}fn make()->&'static R{&R{p:&S{tag:7,tail:[3,5]}}}fn main(){check(make().p.tail[1]==5);}",
                false,
            ),
            (
                "empty",
                "struct S<T:?Sized>{tag:u8,tail:T}const C:&S<[u8]>=&S{tag:7,tail:[]};fn main(){check(C.tag==7);match &C.tail{[]=>(),_=>check(false)}}",
                false,
            ),
            (
                "zst",
                "struct S<T:?Sized>{tag:u8,tail:T}const C:&S<[()]>=&S{tag:7,tail:[();3]};fn main(){check(C.tag==7);match &C.tail{[(),(),()]=>(),_=>check(false)}}",
                false,
            ),
            (
                "projected",
                "trait A{type Out:?Sized;}impl A for u8{type Out=[u16;3];}impl A for u16{type Out=[u16];}struct S<T:A>{tag:u8,tail:T::Out}const C:&S<u16>=&S::<u8>{tag:7,tail:[3,5,7]};fn main(){check(C.tag==7);check(C.tail[2]==7);}",
                false,
            ),
            (
                "repeat-reference",
                "struct S<T:?Sized>{tag:u8,tail:T}const C:&S<[u8]>=&S{tag:7,tail:[3,5]};static A:[&S<[u8]>;2]=[C;2];fn main(){check(A[0].tag==7);check(A[1].tail[1]==5);}",
                false,
            ),
        ],
    )
}
