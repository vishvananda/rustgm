union U{a:u32,b:u8}static S:U=U{a:0x12345678};const A:*const u32=&raw const S.a;const B:*const u8=&raw const S.b;const X:u8=unsafe{*B};
static mut M:U=U{a:7};const MP:*mut u32=unsafe{&raw mut M.a};
fn main(){unsafe{check(*MP==7);*MP=19;check(M.b==19);}check(X==0x78);unsafe{check(*A==0x12345678);check(*B==0x78);}check(A as *const u8==B);}
