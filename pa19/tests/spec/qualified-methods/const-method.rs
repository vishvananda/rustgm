fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl S{fn sum<const N:usize>(&self,x:[u8;N])->u8{self.0+x[N-1]}}fn main(){check(S::sum::<2>(&S(3),[5,7])==10);}