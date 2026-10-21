union U<T:Copy>{a:T,b:u8}static S:U<u32>=U{a:83};const fn address<T:Copy>(u:&U<T>)->*const T{&raw const u.a}const P:*const u32=address(&S);const X:u32=unsafe{*P};
fn main(){check(X==83);unsafe{check(*P==83);}}
