union U{small:u8,wide:u64}const X:u64={let u=U{small:7};unsafe{let r=&u.wide;*r}};fn main(){let _=X;}
