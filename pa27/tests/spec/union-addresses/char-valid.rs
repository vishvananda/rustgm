union U{a:u32,b:char}const X:char={let u=U{a:0x3bb};let p=&raw const u.b;unsafe{*p}};fn main(){let _=X;}
