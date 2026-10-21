union U{word:u32,byte:u8}const X:u32={let mut u=U{byte:1};u.byte=2;unsafe{u.word}};fn main(){let _=X;}
