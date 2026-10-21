union U{a:u32,b:u8}static mut S:U=U{a:7};const X:u32={let p=unsafe{&raw const S.a};unsafe{*p}};fn main(){let _=X;}
