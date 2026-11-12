unsafe extern "C" { fn mark(step:u8); fn combine(a:u8,b:u8)->u8; }
struct Wrap<T>(T);struct Other<T>(T);
trait Try {type Output;}
impl<T>Try for Wrap<T>{type Output=T;}
impl<T>Try for Other<T>{type Output=T;}
trait Iterator {type Item;
 fn try_fold<B,F,R>(&mut self,init:B,f:F)->R where Self:Sized,F:FnMut(B,Self::Item)->R,R:Try<Output=B>{loop{}}
 fn fold<B,F>(self,init:B,f:F)->B where Self:Sized,F:FnMut(B,Self::Item)->B{loop{}}
}
impl<T>Wrap<T>{fn wrap_mut_2<A,B,F>(mut f:F)->impl FnMut(A,B)->Self where F:FnMut(A,B)->T{unsafe{mark(2)};move|a,b|Self(f(a,b))}}
struct Once{value:u8}
impl Iterator for Once{type Item=u8;
 fn try_fold<B,F,R>(&mut self,init:B,mut f:F)->R where F:FnMut(B,Self::Item)->R,R:Try<Output=B>{unsafe{mark(3)};f(init,self.value)}
}
struct ByRefSized<'a,I>(&'a mut I);
impl<I:Iterator>Iterator for ByRefSized<'_,I>{type Item=I::Item;
 fn fold<B,F>(self,init:B,f:F)->B where F:FnMut(B,Self::Item)->B{I::try_fold(self.0,init,Wrap::wrap_mut_2(f)).0}
}
#[unsafe(no_mangle)]pub extern "C" fn code()->u8{unsafe{mark(1)};let mut iter=Once{value:12};ByRefSized(&mut iter).fold(11,|a,b|unsafe{combine(a,b)})}
