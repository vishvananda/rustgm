fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:u8=7;const A:[u8;{let n=match 7u8{N=>1,_=>2};forward(n)}]=[9;1];const fn forward(n:usize)->usize{n}fn main(){check(A[0]==9);}
