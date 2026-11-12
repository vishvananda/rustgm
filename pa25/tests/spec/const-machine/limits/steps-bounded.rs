fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
const fn sum(n:u32)->u32{let mut i:u32=0;let mut s:u32=0;while i<n{s=(s+i)%1000003;i+=1;}s}const C:u32=sum(100000);fn main(){verify(C==935003);}
