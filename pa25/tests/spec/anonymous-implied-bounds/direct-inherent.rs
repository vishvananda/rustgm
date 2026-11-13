enum Slot<T>{Empty,Full(T)}
pub struct Holder<'a,T:'a>{slice:&'a[T]}
impl<'a,T> Holder<'a,T>{fn make<const N:usize>(&mut self)->[Slot<&'a T>;N]{[const{Slot::<&'a T>::Empty};N]}}
