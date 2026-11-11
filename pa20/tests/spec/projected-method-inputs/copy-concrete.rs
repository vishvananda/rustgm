#[derive(Copy,Clone)]struct Wrap<T>(T);impl<T>Wrap<T>{fn get(&self)->u8{23}}
fn probe(w:&Wrap<u8>)->u8{w.get()}
fn main(){}
