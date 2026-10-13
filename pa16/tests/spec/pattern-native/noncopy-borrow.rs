fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct N(u8);fn f(s:&[N])->u8{match s{[a,..,b]=>a.0+b.0,_=>0}}fn main(){check(f(&[N(7),N(9)])==16);}
