union U{a:*const u32,b:usize}const X:usize={let u=U{b:0};let p=&raw const u.b;unsafe{let r=&*p;*r}};fn main(){let _=X;}
