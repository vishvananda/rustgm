union U{a:u32,b:u8}static S:U=U{a:0x12345678};const P:*const u8=&raw const S.b;const X:u8=unsafe{*P};fn main(){let _=X;}
