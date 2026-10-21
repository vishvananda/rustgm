fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn main(){let mut x=[7u8,9];{let mut f=||{x[0]+=1;x[1]+=2;};f();f();}let i=1usize;let f=||x[i];verify(f()==13);verify(f()==13);verify(x[0]==9);}
