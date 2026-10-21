union U{small:u8,wide:u64}const X:u64={let u=U{small:7};let p=&raw const u.wide;unsafe{let r=&*p;*r}};fn main(){let _=X;}
