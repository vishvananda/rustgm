fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}const N:u16={let mut x=[1u16,7];unsafe{*<*mut u16>::add(&raw mut x as *mut u16,1)=11;}x[1]};fn main(){check(N==11);}
