union U{word:u32,byte:u8}const fn f()->u32{let mut u=U{word:37};u.word=91;unsafe{u.word}}const X:u32=f();fn main(){let _=X;}
