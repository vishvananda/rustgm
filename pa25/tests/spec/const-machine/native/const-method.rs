fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct S(u8);impl S{const fn get(&self)->u8{self.0}}const C:u8=S(7).get();fn main(){verify(C==7);verify(S(9).get()==9);}