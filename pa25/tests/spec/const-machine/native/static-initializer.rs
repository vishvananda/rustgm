fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
const fn f(n:u8)->u8{n+1}const C:u8=f(6);static S:u8=f(8);fn main(){verify(C==7);verify(S==9);}