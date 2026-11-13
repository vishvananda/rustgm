enum Slot<T>{Empty,Full(T)}
pub fn make<'a,T>(input:&'a T)->Slot<&'a T>{const{Slot::<&'a T>::Empty}}
