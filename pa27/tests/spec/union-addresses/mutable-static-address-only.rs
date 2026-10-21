union U{a:u32,b:u8}static mut S:U=U{a:7};const P:*mut u32=unsafe{&raw mut S.a};const X:u32=0;fn main(){let _=P;let _=X;}
