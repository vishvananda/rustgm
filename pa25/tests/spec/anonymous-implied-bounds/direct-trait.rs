enum Slot<T>{Empty,Full(T)}
pub struct Holder<'a,T:'a>{slice:&'a[T]}
trait Chunks{type Item;fn make<const N:usize>(&mut self)->[Slot<Self::Item>;N];}
impl<'a,T> Chunks for Holder<'a,T>{type Item=&'a T;fn make<const N:usize>(&mut self)->[Slot<&'a T>;N]{[const{Slot::<&'a T>::Empty};N]}}
