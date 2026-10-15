fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{V(u8)}impl E{fn V()->u8{9}}fn main(){let E::V(x)=<E>::V(11);check(x==11);}