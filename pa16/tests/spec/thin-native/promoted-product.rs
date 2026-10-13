fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f()->&'static (u64,[u8;3]){&(42,[7;3])}fn main(){let r=f();check(r.0==42 && r.1[2]==7);}
