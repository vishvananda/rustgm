trait T{fn get(&self)->u8 where Self:Sized;}fn f(x:&dyn T)->u8{x.get()}
fn main(){}
