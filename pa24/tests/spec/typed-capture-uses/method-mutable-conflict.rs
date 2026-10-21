struct P(u8);impl P{fn read(&self)->u8{self.0}fn change(&mut self){self.0+=1;}fn take(self)->u8{self.0}}fn main(){let mut x=P(7);let mut f=||x.change();let _=x.read();f();}
