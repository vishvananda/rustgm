enum Slot<T>{Empty,Full(T)}
pub fn bad<'a>()->Slot<&'a u8>{let a=const{let x=7u8;Slot::Full(&x)};a}
