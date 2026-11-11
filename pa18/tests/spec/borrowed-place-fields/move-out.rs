struct Wrap<T>(T);impl<T>Wrap<T>{fn get(&self)->u8{23}fn change(&mut self)->u8{41}}
fn probe<T>(w:&Wrap<T>)->T{w.0}fn main(){}
