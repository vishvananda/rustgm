enum Option<T>{None,Some(T)}
impl<T> Option<T>{
fn and_then<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->Option<U>{loop{}}
fn map<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->U{loop{}}
fn or(self,other:Self)->Self{self}
}
trait Iterator{type Item;fn last(self)->Option<Self::Item>where Self:Sized{loop{}}}
fn last<U:Iterator>(last:Option<U::Item>,iter:U)->Option<U::Item>{iter.last().or(last)}
fn iter_fold<U:Iterator,B,F>(init:B,f:F)->B where F:FnMut(B,U)->B{loop{}}
fn code<U:Iterator>()->Option<U::Item>{iter_fold::<U,_,_>(Option::None,last)}
