fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:u8>()->u8{N}fn main(){let p:&'static fn()->u8=&(f::<7> as fn()->u8);let q:&'static fn()->u8=&(f::<9> as fn()->u8);check((*p)()==7&&(*q)()==9);}
