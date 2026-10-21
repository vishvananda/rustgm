union U{a:u32,b:u32}struct W{u:U}const X:u32={let mut x=W{u:U{a:7}};let p=&raw mut x.u.b;unsafe{let r=&mut *p;*r=37;x.u.a}};fn main(){let _=X;}
