fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
unsafe fn f(x:u8)->u8{x}fn main(){let g:&dyn Fn(u8)->u8=&f;g(7);}
