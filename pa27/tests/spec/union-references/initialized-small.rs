union U{small:u8,wide:u64}const X:u8={let u=U{small:7};let p=&raw const u.small;unsafe{let r=&*p;*r}};fn main(){let _=X;}
