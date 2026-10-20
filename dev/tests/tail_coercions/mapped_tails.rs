//! The normalized physical tail supplies metadata independently of the changed argument.
use super::native;

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn projected_unsizing_consumes_the_actual_final_field() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pa22-native/projected-coercions",
        &[
            (
                "borrowed-literal",
                "trait A{type Out:?Sized;}impl A for u8{type Out=[u8;2];}impl A for u16{type Out=[u8];}struct S<T:A>{tag:u8,tail:T::Out}fn read(s:&S<u16>)->u8{check(s.tag==9);s.tail[1]}fn main(){check(read(&S::<u8>{tag:9,tail:[3,5]})==5);}",
                false,
            ),
            (
                "multiple-types",
                "trait A<P>{type Out:?Sized;}impl A<u8> for u8{type Out=[u16;3];}impl A<u16> for u16{type Out=[u16];}struct S<P,T:A<P>>{tag:u8,tail:T::Out}fn read(s:&S<u16,u16>)->u16{check(s.tag==9);s.tail[2]}fn main(){let s=S::<u8,u8>{tag:9,tail:[3,5,7]};check(read(&s)==7);}",
                false,
            ),
            (
                "type-and-const",
                "trait A<const N:usize>{type Out:?Sized;}impl A<2> for u8{type Out=[u16;3];}impl A<7> for u16{type Out=[u16];}struct S<T:A<N>,const N:usize>{tag:u8,tail:T::Out}fn read(s:&S<u16,7>)->u16{check(s.tag==9);s.tail[2]}fn main(){let s=S::<u8,2>{tag:9,tail:[3,5,7]};check(read(&s)==7);}",
                false,
            ),
            (
                "branch",
                "trait A{type Out:?Sized;}impl A for u8{type Out=[u8;2];}impl A for u16{type Out=[u8;3];}impl A for u32{type Out=[u8];}struct S<T:A>{tail:T::Out}fn read(a:&S<u8>,b:&S<u16>,yes:bool)->u8{let s:&S<u32>=if yes{a}else{b};s.tail[1]}fn main(){let a=S::<u8>{tail:[3,5]};let b=S::<u16>{tail:[7,11,13]};check(read(&a,&b,true)==5);check(read(&a,&b,false)==11);}",
                false,
            ),
            (
                "match",
                "trait A{type Out:?Sized;}impl A for u8{type Out=[u8;2];}impl A for u16{type Out=[u8;3];}impl A for u32{type Out=[u8];}struct S<T:A>{tail:T::Out}fn read(a:&S<u8>,b:&S<u16>,yes:bool)->u8{let s:&S<u32>=match yes{true=>a,false=>b};s.tail[1]}fn main(){let a=S::<u8>{tail:[3,5]};let b=S::<u16>{tail:[7,11,13]};check(read(&a,&b,true)==5);check(read(&a,&b,false)==11);}",
                false,
            ),
            (
                "identity",
                "trait A{type Out:?Sized;}impl<T,const N:usize>A for [T;N]{type Out=[T;N];}impl<T>A for [T]{type Out=[T];}struct S<T:A+?Sized>{tag:u8,tail:T::Out}fn read(s:&S<[u8]>)->u8{check(s.tag==9);s.tail[1]}fn main(){let s=S::<[u8;2]>{tag:9,tail:[3,5]};check(read(&s)==5);}",
                false,
            ),
            (
                "different-length",
                "trait A{type Out:?Sized;}impl<const N:usize>A for [u8;N]{type Out=[u16;3];}impl A for [u8]{type Out=[u16];}struct S<T:A+?Sized>{tag:u8,tail:T::Out}fn read(s:&S<[u8]>)->u16{match &s.tail{[a,b,c]=>*a+*b+*c,_=>0}}fn main(){let s=S::<[u8;1]>{tag:9,tail:[3,5,7]};check(read(&s)==15);}",
                false,
            ),
            (
                "mutable",
                "trait A{type Out:?Sized;}impl<const N:usize>A for [u8;N]{type Out=[u16;3];}impl A for [u8]{type Out=[u16];}struct S<T:A+?Sized>{tag:u8,tail:T::Out}fn edit(s:&mut S<[u8]>){s.tail[2]=23;}fn main(){let mut s=S::<[u8;1]>{tag:9,tail:[3,5,7]};edit(&mut s);check(s.tail[2]==23);}",
                false,
            ),
            (
                "trait-argument",
                "trait A<P>{type Out:?Sized;}impl<P,const N:usize>A<P> for [u8;N]{type Out=[P;N];}impl<P>A<P> for [u8]{type Out=[P];}struct S<P,T:A<P>+?Sized>{tail:T::Out}fn read(s:&S<u16,[u8]>)->u16{s.tail[1]}fn main(){let s=S::<u16,[u8;2]>{tail:[3,5]};check(read(&s)==5);}",
                false,
            ),
            (
                "nested",
                "trait A{type Out:?Sized;}impl<T,const N:usize>A for [T;N]{type Out=[T;N];}impl<T>A for [T]{type Out=[T];}struct S<T:A+?Sized>{tail:T::Out}struct W<T:A+?Sized>{tag:u8,inner:S<T>}fn read(s:&W<[u8]>)->u8{check(s.tag==9);s.inner.tail[1]}fn main(){let s=W::<[u8;2]>{tag:9,inner:S::<[u8;2]>{tail:[3,5]}};check(read(&s)==5);}",
                false,
            ),
            (
                "generic",
                "trait A{type Out:?Sized;}impl<T,const N:usize>A for [T;N]{type Out=[T;N];}impl<T>A for [T]{type Out=[T];}struct S<T:A+?Sized>{tail:T::Out}fn convert<T,const N:usize>(s:&S<[T;N]>)->&S<[T]>{s}fn main(){let s=S::<[u8;2]>{tail:[3,5]};check(convert(&s).tail[1]==5);}",
                false,
            ),
        ],
    )
}
