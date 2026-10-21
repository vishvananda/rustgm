union U{small:u8,wide:u64}const X:u64={let mut u=U{small:7};let p=&raw mut u.wide;unsafe{let r=&mut *p;*r=37;u.wide}};fn main(){let _=X;}
