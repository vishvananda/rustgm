fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Z;fn main(){let z=Z;let f=move|x:&u8|{let r=&z;let _=r;*x};let p:&dyn Fn(&u8)->u8=&f;check(p(&7)==7);}
