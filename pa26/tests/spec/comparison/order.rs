fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0+1==r.0}}fn mark(n:&mut u8)->S{*n+=1;S(*n)}fn main(){let mut n=0;check(mark(&mut n)==mark(&mut n));check(n==2);}
