fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<const N:usize>=[u8;N];fn read(x:A<2>,i:usize)->u8{x[i]}fn main(){read([3,4],2);}
