fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
const fn f(n:u32)->u32{if n==0{1}else{n*f(n-1)}}const C:u32=f(6);fn main(){verify(C==720);verify(f(5)==120);}