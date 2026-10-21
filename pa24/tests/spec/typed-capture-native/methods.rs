fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct P(u8);impl P{fn read(&self)->u8{self.0}fn change(&mut self){self.0+=1;}fn take(self)->u8{self.0}}fn main(){let mut x=P(7);{let mut f=||x.change();f();f();}let f=||x.read();verify(f()==9);verify(f()==9);let g=||x.take();verify(g()==9);}
