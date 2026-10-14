struct N;struct D(N);impl Drop for D{fn drop(&mut self){}}fn f<T>(d:D){let n=d.0;}
fn main(){}
