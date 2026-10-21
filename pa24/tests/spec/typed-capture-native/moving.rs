fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct P(u8);impl P{fn read(&self)->u8{self.0}fn change(&mut self){self.0+=1;}fn take(self)->u8{self.0}}fn main(){let x=P(7);let f=move||x.read();verify(f()==7);verify(f()==7);let a=[3u8,5];let g=move||a[1];verify(g()==5);verify(g()==5);}
