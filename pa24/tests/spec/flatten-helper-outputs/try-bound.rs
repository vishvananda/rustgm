trait Try{type Output;}
trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter;}
trait Iterator:Sized{type Item;fn try_fold<B,F,R>(&mut self,init:B,f:F)->R where F:FnMut(B,Self::Item)->R,R:Try<Output=B>;}
enum Option<T>{None,Some(T)}
impl<T> Option<T>{fn insert(&mut self,value:T)->&mut T{*self=Option::Some(value);match self{Option::Some(x)=>x,Option::None=>loop{}}}}
fn flatten<'a,T:IntoIterator,B,R:Try<Output=B>>(front:&'a mut Option<T::IntoIter>,fold:&'a mut impl FnMut(B,&mut T::IntoIter)->R)->impl FnMut(B,T)->R+'a{move|acc,item|fold(acc,front.insert(item.into_iter()))}
fn code<I:Iterator,U,B,R:Try<Output=B>,F:FnMut(B,&mut U)->R>(iter:&mut I,front:&mut Option<U>,acc:B,mut fold:F)->R where I::Item:IntoIterator<IntoIter=U>{iter.try_fold(acc,flatten(front,&mut fold))}
