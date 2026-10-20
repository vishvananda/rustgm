trait T{fn get(&self)->u8;}fn f(x:&dyn T)->u8{<dyn T>::get(x)}
fn main(){}
