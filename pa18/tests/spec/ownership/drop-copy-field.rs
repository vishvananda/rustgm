struct D(u8);impl Drop for D{fn drop(&mut self){}}fn f<T>(d:D){let n=d.0;}
fn main(){}
