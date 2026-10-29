fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
const C:u8=f();const fn f()->u8{N+1}const N:u8=6;fn main(){verify(C==7);}