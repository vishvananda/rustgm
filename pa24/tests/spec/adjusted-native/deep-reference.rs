fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct S(u8);impl S{fn get(&self)->u8{self.0}}
fn main(){let s=S(7);let r=&&s;let f=||r.get();verify(f()==7);verify(f()==7);}
