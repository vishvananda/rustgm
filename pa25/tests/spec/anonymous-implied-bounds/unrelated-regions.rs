enum Slot<T>{Empty,Full(T)}
pub struct Holder<'a,T:'a>{slice:&'a[T]}impl<'a,T>Holder<'a,T>{pub fn make<'b>(&mut self)->Slot<&'b T>{const{let value:Slot<&'a T>=Slot::<&'a T>::Empty;value}}}
