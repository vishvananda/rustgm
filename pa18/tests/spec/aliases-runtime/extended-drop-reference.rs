fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S{n:usize}impl Drop for S{fn drop(&mut self){check(false);}}const R:&S=&S{n:5};type A<const N:usize>=[u8;N];fn main(){let x:A<{R.n}>=[2;5];check(x[4]==2);check(R.n==5);}
