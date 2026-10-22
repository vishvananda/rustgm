const fn count(a:&[u8])->u8{match a{[_,_,_,..]=>3,_=>0}}unsafe extern "C"{static A:[u8;3];}const N:u8=unsafe{count(&A)};fn main(){let _=N;}
