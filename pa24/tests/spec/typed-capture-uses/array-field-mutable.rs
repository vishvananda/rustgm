struct P(u8);fn main(){let mut x=[P(7),P(9)];let mut f=||{x[0].0+=1;};f();f();}
