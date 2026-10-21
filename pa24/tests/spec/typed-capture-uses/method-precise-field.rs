struct P(u8);impl P{fn read(&self)->u8{self.0}fn change(&mut self){self.0+=1;}fn take(self)->u8{self.0}}fn main(){let x=(P(7),P(9));let f=||x.0.read();let _=x.1;let _=f();}
