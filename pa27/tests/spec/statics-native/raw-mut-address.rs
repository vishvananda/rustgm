fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static mut X:u8=3;fn main(){let p=&raw mut X;unsafe{*p=9;check(X==9);}}