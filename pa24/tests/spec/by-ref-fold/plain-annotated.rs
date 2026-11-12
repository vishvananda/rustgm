struct Wrap<T>(T);
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
impl<T> Wrap<T> {
 fn wrap_mut_2<A,B,F>(mut f:F)->impl FnMut(A,B)->Self
 where F:FnMut(A,B)->T { move |a,b| Self(f(a,b)) }
}
fn wrap_free<A,B,T,F>(mut f:F)->impl FnMut(A,B)->Wrap<T>
where F:FnMut(A,B)->T { move |a,b| Wrap(f(a,b)) }
fn code<I:Iterator,B,F>(iter:&mut I,init:B,f:F)->B where F:FnMut(B,I::Item)->B {let result:Wrap<B>=I::try_fold(iter,init,Wrap::wrap_mut_2(f));result.0}
