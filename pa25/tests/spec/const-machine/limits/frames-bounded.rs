fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
const fn depth(n:u32)->u32{if n==0{0}else{1+depth(n-1)}}const D:u32=depth(100);fn main(){verify(D==100);}
