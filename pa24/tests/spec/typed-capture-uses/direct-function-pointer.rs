fn take(x:u8)->u8{x}fn main(){let x=7u8;let p:fn(u8)->u8=take;let f=||p(x);let _:u8=f();}
