fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Packet{value:u8}fn take<T>(x:T)->T{let f=move||x;f()}fn main(){check(take(Packet{value:7}).value==7);}
