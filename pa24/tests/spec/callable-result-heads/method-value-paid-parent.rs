enum Option<T>{None,Some(T)}
impl<T> Option<T>{
fn and_then<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->Option<U>{loop{}}
fn map<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->U{loop{}}
fn or(self,other:Self)->Self{self}
}
trait Iterator{type Item;fn last(self)->Option<Self::Item>where Self:Sized{loop{}}}
trait IntoIterator{type IntoIter;fn into_iter(self)->Self::IntoIter where Self:Sized{loop{}}}
fn code<A:Iterator+IntoIterator<IntoIter=<A as Iterator>::Item>>(x:Option<A>)->Option<A::Item>{x.map(IntoIterator::into_iter)}
