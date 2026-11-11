struct Wrap<T>(T);impl<T>Wrap<T>{fn get(&self)->u8{23}fn change(&mut self)->u8{41}}
fn probe(w:&Wrap<u8>)->u8{Wrap::<u8>::get(w)}fn main(){}
