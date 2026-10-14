fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:u8>()->u8{N}static P:[fn()->u8;2]=[f::<7>,f::<9>];fn main(){check(P[0]()==7&&P[1]()==9);}
