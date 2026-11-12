enum Option<T>{None,Some(T)}
impl<T> Option<T>{
fn and_then<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->Option<U>{loop{}}
fn map<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->U{loop{}}
fn or(self,other:Self)->Self{self}
}
trait Iterator{type Item;fn last(self)->Option<Self::Item>where Self:Sized{loop{}}}
struct Buffer<T,const N:usize>{data:[T;N]}
impl<T,const N:usize>Buffer<T,N>{fn as_array_ref(&self)->&[T;N]{&self.data}}
fn code<'a,I:Iterator,const N:usize>(x:Option<&'a Buffer<I::Item,N>>)->Option<&'a[I::Item;N]>{x.map(Buffer::as_array_ref)}
