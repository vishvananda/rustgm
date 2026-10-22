const X:u32={let x=7u32;let r=&x;let s=&r;let p=(&raw const s)as *const &'static &'static u32;unsafe{***p}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(X==7);}
