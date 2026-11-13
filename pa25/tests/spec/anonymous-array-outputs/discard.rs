enum Slot<T>{Empty,Full(T)}
enum Answer<T,E>{Ok(T),Err(E)}struct Owned<T,const N:usize>([Slot<T>;N]);
pub struct Holder<'a,T:'a>{slice:&'a[T]}impl<'a,T>Holder<'a,T>{fn make<const N:usize>(&mut self)->u8{let a=[const{Slot::<&'a T>::Empty};N];7u8}}
