#![feature(const_closures,const_trait_impl,const_destruct)]
use std::marker::Destruct;
struct Wrap<T>(T);impl<T>Wrap<T>{const fn two<A,B,F>(mut f:F)->impl [const]FnMut(A,B)->Self+[const]Destruct where F:[const]FnMut(A,B)->T+[const]Destruct{const move|a,b|Self(f(a,b))}}
struct Other<T>(T);
trait Try { type Output; }
impl<T> Try for Wrap<T> { type Output=T; }
impl<T> Try for Other<T> { type Output=T; }
trait Iterator {
 type Item;
 fn try_fold<B,F,R>(&mut self,init:B,f:F)->R
 where Self:Sized,F:FnMut(B,Self::Item)->R,R:Try<Output=B> { loop{} }
 fn fold<B,F>(self,init:B,f:F)->B
 where Self:Sized,F:FnMut(B,Self::Item)->B { loop{} }
}
struct ByRefSized<'a,I>(&'a mut I);
impl<I:Iterator> Iterator for ByRefSized<'_,I> {
 type Item=I::Item;
 fn fold<B,F>(self,init:B,f:F)->B where F:FnMut(B,Self::Item)->B {
  I::try_fold(self.0,init,Wrap::two(f)).0
 }
}
