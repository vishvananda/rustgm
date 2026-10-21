fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct P(u8);fn main(){let mut x=[P(7),P(9)];{let mut f=||{x[0].0+=2;};f();f();}let f=||x[0].0;verify(f()==11);verify(f()==11);verify(x[1].0==9);}
