enum Slot<T>{Empty,Full(T)}
const fn need_static<T:'static>()->Slot<T>{Slot::Empty}pub struct Holder<'a,T:'a>{slice:&'a[T]}impl<'a,T> Holder<'a,T>{pub fn make(&mut self)->Slot<T>{const{need_static::<T>()}}}
