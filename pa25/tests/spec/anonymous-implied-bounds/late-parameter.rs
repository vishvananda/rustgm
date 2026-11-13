enum Slot<T>{Empty,Full(T)}
const fn warm<'a>(p:&'a u8)->usize{const{1usize}}
pub fn prefix(value:[u8;warm(&7u8)]){}
pub struct Holder<'a,T:'a>{slice:&'a[T]}impl<'a,T>Holder<'a,T>{fn make<const N:usize>(&mut self)->[Slot<&'a T>;N]{[const{Slot::<&'a T>::Empty};N]}}
