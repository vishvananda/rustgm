fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Z;fn main(){let z=Z;let f=move||{let _=&z;7u8};let p:&dyn Fn()->u8=&f;check(p()==7);}
