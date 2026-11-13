enum Slot<T>{Empty,Full(T)}
pub struct Holder<'a,T:'a>{value:&'a T}impl<'a,T>Holder<'a,T>{fn make(&mut self)->u8{const{let value:Slot<&'a T>=Slot::<&'a T>::Empty;7u8}}}#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{let x=7u8;let mut h=Holder{value:&x};h.make()}
