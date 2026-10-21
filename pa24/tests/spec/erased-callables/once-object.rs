fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let n=S(7);let f=move||n;let _g:&dyn FnOnce()->S=&f;}
