enum Option<T>{None,Some(T)}
impl<T> Option<T>{
fn and_then<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->Option<U>{loop{}}
fn map<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->U{loop{}}
fn or(self,other:Self)->Self{self}
}
trait Iterator{type Item;fn last(self)->Option<Self::Item>where Self:Sized{loop{}}}
fn last<U:Iterator>(last:Option<U::Item>,iter:U)->Option<U::Item>{iter.last().or(last)}
fn iter_fold<U:Iterator,F>(init:Option<U::Item>,f:F)->Option<U::Item>where F:FnMut(Option<U::Item>,U)->Option<U::Item>{loop{}}
fn code<U:Iterator>()->Option<U::Item>{iter_fold::<U,_>(Option::None,last)}
