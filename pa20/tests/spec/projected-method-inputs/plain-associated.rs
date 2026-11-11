trait Family{type Out;}
struct Wrap<T>(T);impl<T>Wrap<T>{fn get(&self)->u8{23}}
fn probe<R:Family>(w:&Wrap<R::Out>)->u8{w.get()}
fn main(){}
