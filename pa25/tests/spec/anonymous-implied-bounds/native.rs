enum Slot<T>{Empty,Full(T)}
pub struct Holder<'a,T:'a>{slice:&'a[T]}impl<'a,T>Holder<'a,T>{fn make<const N:usize>(&mut self)->[Slot<&'a T>;N]{[const{Slot::<&'a T>::Empty};N]}}#[unsafe(no_mangle)]pub extern "C" fn observe()->u32{let x=7u32;let mut h=Holder{slice:&[x]};let a=h.make::<2>();match a[0]{Slot::Empty=>x,Slot::Full(p)=>*p}}
