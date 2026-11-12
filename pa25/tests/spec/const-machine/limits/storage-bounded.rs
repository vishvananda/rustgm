fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
const fn fill()->u32{let mut a=[0u32;100000];let mut i:usize=0;while i<100000{a[i]=i as u32;i+=1;}a[99999]+a[12345]}const S:u32=fill();fn main(){verify(S==112344);}
