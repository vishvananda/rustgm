fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T,const N:usize>=[T;N];fn a(x:A<u8,2>)->u8{x[1]}fn b(x:A<u16,3>)->u16{x[2]}fn main(){check(a([1,7])==7);check(b([2,3,600])==600);}
