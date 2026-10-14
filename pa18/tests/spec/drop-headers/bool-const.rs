struct S<const B:bool>;impl<const B:bool> Drop for S<B>{fn drop(&mut self){}}
fn main(){}
