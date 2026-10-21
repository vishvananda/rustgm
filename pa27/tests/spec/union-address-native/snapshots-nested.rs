#[derive(Clone,Copy)]union U{a:u32,b:u8}struct S{pad:u8,u:U}
const A:(u32,u32)={let mut u=U{a:0x12345678};let v=u;let a=&raw const u.a;let b=&raw mut u.b;unsafe{*b=0xab;(*a,v.a)}};
const B:S={let mut s=S{pad:1,u:U{a:0x12345678}};let a=&raw mut s.u.a;let b=&raw const s.u.b;unsafe{*a=0x31415926;let _=*b;}s};
fn main(){check(A.0==0x123456ab);check(A.1==0x12345678);check(B.pad==1);unsafe{check(B.u.a==0x31415926);}}
