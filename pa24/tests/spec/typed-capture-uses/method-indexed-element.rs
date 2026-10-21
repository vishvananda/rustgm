struct P(u8);impl P{fn read(&self)->u8{self.0}}fn main(){let x=[P(7),P(9)];let f=||x[0].read();let _:u8=f();let _=f();}
