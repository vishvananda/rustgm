union U{a:u32,b:char}const X:char={let u=U{a:0xd800};let p=&raw const u.b;unsafe{*p}};fn main(){let _=X;}
