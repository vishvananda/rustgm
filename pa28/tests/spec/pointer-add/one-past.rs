fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}static X:[u16;2]=[1,7];const P:*const u16=unsafe{(&raw const X as *const u16).add(2)};fn main(){let _=P;}
