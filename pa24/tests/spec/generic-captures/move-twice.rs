fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Packet;fn take<T>(x:T){let f=move||x;let _=f();let _=f();}fn main(){take(Packet);}
