trait T{fn get(&self)->&u8;}fn f(x:&dyn T)->&'static u8{x.get()}
fn main(){}
