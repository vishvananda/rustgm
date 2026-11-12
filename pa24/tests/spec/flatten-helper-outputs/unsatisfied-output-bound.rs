trait Try{type Output;}
trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter;}
trait Iterator:Sized{type Item;fn try_fold<B,F,R>(&mut self,init:B,f:F)->R where F:FnMut(B,Self::Item)->R;}
enum Option<T>{None,Some(T)}
impl<T> Option<T>{fn insert(&mut self,value:T)->&mut T{*self=Option::Some(value);match self{Option::Some(x)=>x,Option::None=>loop{}}}}
fn flatten<'a,T:IntoIterator,B,R:Try<Output=B>>(front:&'a mut Option<T>)->impl FnMut(B,T)->R+'a{move|_acc,item|{front.insert(item);loop{}}}
fn code<I:Iterator,B,R>(iter:&mut I,front:&mut Option<I::Item>,acc:B)->R where I::Item:IntoIterator{iter.try_fold(acc,flatten(front))}
