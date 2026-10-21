union U{small:u8,wide:u64}const X:u64={let mut u=U{small:7};unsafe{let r=&mut u.wide;*r=37;u.wide}};fn main(){let _=X;}
