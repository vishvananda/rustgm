const fn count(a:&[u8])->u8{match a{[_,..]=>1,_=>0}}static A:[u8;1]=[count(&A)];fn main(){let _=A[0];}
