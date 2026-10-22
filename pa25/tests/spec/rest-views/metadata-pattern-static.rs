const fn count(a:&[u8])->u8{match a{[_,_,_,..]=>3,_=>0}}static A:[u8;1_000_000]=[7;1_000_000];const N:u8=count(&A);fn main(){let _=N;}
