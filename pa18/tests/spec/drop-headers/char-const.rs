struct S<const C:char>;impl<const C:char> Drop for S<C>{fn drop(&mut self){}}
fn main(){}
