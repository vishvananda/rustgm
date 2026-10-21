union U{small:u8,wide:u64}const X:u64={let mut u=U{small:7};let p=&raw mut u.wide;unsafe{*p=37;let r=&*p;*r}};fn main(){let _=X;}
