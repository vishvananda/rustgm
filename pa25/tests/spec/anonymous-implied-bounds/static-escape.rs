enum Slot<T>{Empty,Full(T)}
pub fn bad<'a,const N:usize>()->[Slot<&'static u8>;N]{let values:[Slot<&'a u8>;N]=[const{Slot::<&'a u8>::Empty};N];values}

