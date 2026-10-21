union U{word:u32,byte:u8}const fn f(u:&mut U)->u32{u.byte=9;unsafe{u.word}}const X:u32={let mut u=U{word:16909060};f(&mut u)};fn main(){let _=X;}
