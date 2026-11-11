struct Wrap<T>(T);impl<T>Wrap<T>{fn get(&self)->u8{23}fn change(&mut self)->u8{41}}
fn probe<T>(w:&Wrap<T>)->u8{Wrap::<T>::get(w)}fn main(){}
