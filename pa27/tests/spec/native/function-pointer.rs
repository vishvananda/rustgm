fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
unsafe fn get(p:*const u8)->u8{unsafe{*p}}fn main(){let x=9u8;let f:unsafe fn(*const u8)->u8=get;check(unsafe{f(&raw const x)}==9);}