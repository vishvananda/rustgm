fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
struct S{é:u8,数:u64}fn main(){let x=S{é:7,数:99};check(x.é==7 && x.数==99);}
