enum Option<T>{None,Some(T)}
impl<T> Option<T>{
fn and_then<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->Option<U>{loop{}}
fn map<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->U{loop{}}
fn or(self,other:Self)->Self{self}
}
trait Iterator{type Item;fn last(self)->Option<Self::Item>where Self:Sized{loop{}}}
fn last<V:Iterator>(iter:V,last:Option<V::Item>)->Option<V::Item>{iter.last().or(last)}
struct Outer<U>{inner:Option<U>}
impl<U:Iterator>Outer<U>{fn iter_fold<B,F>(self,init:B,f:F)->B where F:FnMut(U,B)->B{loop{}}fn code(self)->Option<U::Item>{self.iter_fold(Option::None,last)}}
