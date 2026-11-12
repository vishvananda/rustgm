trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter;}
trait Iterator:Sized{type Item;fn try_fold<B,F,R>(&mut self,init:B,f:F)->R where F:FnMut(B,Self::Item)->R;}
enum Option<T>{None,Some(T)}
impl<T> Option<T>{fn insert(&mut self,value:T)->&mut T{*self=Option::Some(value);match self{Option::Some(x)=>x,Option::None=>loop{}}}}
fn flatten<'a,B,R>(front:&'a mut Option<u16>)->impl FnMut(B,u16)->R+'a{move|_acc,item|{front.insert(item);loop{}}}
fn code<I:Iterator,B,R>(iter:&mut I,front:&mut Option<u16>,acc:B)->R{iter.try_fold(acc,flatten(front))}
