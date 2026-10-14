fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T=u16,const N:usize=3>=[T;N];fn take(x:A)->u16{x[0]+x[1]+x[2]}fn main(){check(take([2,3,4])==9);}
