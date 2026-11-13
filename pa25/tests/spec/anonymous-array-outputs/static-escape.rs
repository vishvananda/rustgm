enum Slot<T>{Empty,Full(T)}
pub fn bad<'a>()->[Slot<&'static u8>;1]{let a=[const{Slot::<&'a u8>::Empty};1];a}
