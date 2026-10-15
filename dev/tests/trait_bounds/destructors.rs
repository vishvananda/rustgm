//! Drop impl predicates are compared in the nominal declaration parameter context.

#[test]
fn destructor_headers_map_complete_trait_arguments_before_coverage_proof()
-> Result<(), Box<dyn std::error::Error>> {
    super::cases(&[
        (
            "type-bound",
            "trait A<U>{}struct D<T:A<u8>>(T);impl<T:A<u8>> Drop for D<T>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "type-remap",
            "trait A<U>{}struct D<T:A<U>,U>(T,U);impl<X,Y:A<X>> Drop for D<Y,X>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "const-remap",
            "trait A<const N:usize>{}struct D<T:A<N>,const N:usize>(T);impl<const M:usize,U:A<M>> Drop for D<U,M>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "lifetime-remap",
            "trait A<'a>{}struct D<'a,T:A<'a>>(&'a (),T);impl<'b,U:A<'b>> Drop for D<'b,U>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "type-stronger",
            "trait A<U>{}struct D<T:A<u8>>(T);impl<T:A<u8>+A<u16>> Drop for D<T>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
        (
            "const-stronger",
            "trait A<const N:usize>{}struct D<T:A<N>,const N:usize>(T);impl<const M:usize,U:A<M>+A<7>> Drop for D<U,M>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
        (
            "missing-bound",
            "trait A<U>{}struct D<T:A<u8>>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
        (
            "supertrait-redundant",
            "trait Weak{}trait Strong:Weak{}struct D<T:Strong>(T);impl<T:Strong+Weak> Drop for D<T>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "all-kinds-remap",
            "trait A<'a,U,const N:usize>{}struct D<'a,T:A<'a,U,N>,U,const N:usize>(&'a (),T,U);impl<'b,const M:usize,X,Y:A<'b,X,M>> Drop for D<'b,Y,X,M>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "lifetime-equivalent",
            "trait A<'a>{}struct D<'a:'b,'b:'a,T:A<'a>>(&'a (),&'b (),T);impl<'x:'y,'y:'x,U:A<'y>> Drop for D<'x,'y,U>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "lifetime-stronger",
            "trait A<'a>{}struct D<'a,T:A<'a>>(&'a (),T);impl<'b:'static,U:A<'b>> Drop for D<'b,U>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn destructor_bodies_keep_only_their_declared_permissions() -> Result<(), Box<dyn std::error::Error>>
{
    super::cases(&[
        (
            "supertrait-body",
            "trait A:Copy{}struct D<T:A>(T);impl<T:A> Drop for D<T>{fn drop(&mut self){let _a=self.0;let _b=self.0;}}fn main(){}",
            true,
        ),
        (
            "shared-move",
            "trait A{}struct D<T:A>(T);impl<T:A> Drop for D<T>{fn drop(&mut self){let _a=self.0;}}fn main(){}",
            false,
        ),
        (
            "shadow-copy",
            "trait Copy{}struct D<T:Copy>(T);impl<T:Copy> Drop for D<T>{fn drop(&mut self){let _a=self.0;}}fn main(){}",
            false,
        ),
        (
            "renamed-trait",
            "mod m{pub trait A{}}use m::A as B;struct D<T:B>(T);impl<T:B> Drop for D<T>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "generic-construction",
            "trait A{}struct D<T:A>(T);impl<T:A> Drop for D<T>{fn drop(&mut self){}}fn f<T:A>(t:T){let _d=D(t);}fn main(){}",
            true,
        ),
        (
            "owned-field-move",
            "trait A{}struct D<T:A>(T);impl<T:A> Drop for D<T>{fn drop(&mut self){}}fn f<T:A>(d:D<T>){let _t=d.0;}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn unused_destructor_lifetimes_require_actual_equivalence() -> Result<(), Box<dyn std::error::Error>>
{
    super::cases(&[
        (
            "static-extra",
            "trait A<'a>{}struct D<T:A<'static>>(T);impl<'x:'static,T:A<'x>> Drop for D<T>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "linked-extra",
            "trait A<'a>{}struct D<'a,T:A<'a>>(&'a (),T);impl<'x:'y,'y:'x,U:A<'y>> Drop for D<'x,U>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "unlinked-extra",
            "trait A<'a>{}struct D<'a,T:A<'a>>(&'a (),T);impl<'x,'y,U:A<'y>> Drop for D<'x,U>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
        (
            "static-extra-no-bound",
            "trait A<'a>{}struct D<T:A<'static>>(T);impl<'x,T:A<'x>> Drop for D<T>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
        (
            "permuted-three",
            "trait A<'a>{}struct D<'a:'b,'b:'a,'c,T:A<'a>>(&'a (),&'b (),&'c (),T);impl<'x:'z,'y,'z:'x,U:A<'z>> Drop for D<'x,'z,'y,U>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "permuted-three-stronger",
            "trait A<'a>{}struct D<'a:'b,'b:'a,'c,T:A<'a>>(&'a (),&'b (),&'c (),T);impl<'x:'y+'z,'y:'x,'z:'x,U:A<'y>> Drop for D<'x,'z,'y,U>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
        (
            "extra-path-given",
            "trait A<'a>{}struct D<'a:'b,'b,T:A<'a>>(&'a (),&'b (),T);impl<'x:'z,'y,'z:'x+'y,U:A<'z>> Drop for D<'x,'y,U>{fn drop(&mut self){}}fn main(){}",
            true,
        ),
        (
            "extra-path-stronger",
            "trait A<'a>{}struct D<'a,'b,T:A<'a>>(&'a (),&'b (),T);impl<'x:'z,'y,'z:'x+'y,U:A<'z>> Drop for D<'x,'y,U>{fn drop(&mut self){}}fn main(){}",
            false,
        ),
    ])
}
