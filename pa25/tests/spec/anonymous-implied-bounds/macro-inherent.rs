enum Slot<T>{Empty,Full(T)}
pub struct Holder<'a,T:'a>{slice:&'a[T]}
macro_rules! generate{($elem:ty)=>{impl<'a,T> Holder<'a,T>{fn make<const N:usize>(&mut self)->[Slot<$elem>;N]{[const{Slot::<$elem>::Empty};N]}}}}generate!(&'a T);
