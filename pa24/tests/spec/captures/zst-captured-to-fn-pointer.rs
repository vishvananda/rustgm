fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Z;fn consume(_:Z){}fn main(){let z=Z;let _:fn()=move ||consume(z);}
