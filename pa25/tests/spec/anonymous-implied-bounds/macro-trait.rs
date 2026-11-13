enum Slot<T>{Empty,Full(T)}
pub struct Holder<'a,T:'a>{slice:&'a[T]}
trait Chunks{type Item;fn make<const N:usize>(&mut self)->[Slot<Self::Item>;N];}
macro_rules! generate{($elem:ty)=>{impl<'a,T> Chunks for Holder<'a,T>{type Item=$elem;fn make<const N:usize>(&mut self)->[Slot<$elem>;N]{[const{Slot::<$elem>::Empty};N]}}}}generate!(&'a T);
