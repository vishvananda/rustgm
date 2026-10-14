fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:u8>()->u8{N}const P:&'static (fn()->u8,fn()->u8)=&(f::<7>,f::<9>);fn main(){check(P.0()==7&&P.1()==9);}
