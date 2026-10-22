fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct P;impl P{fn add(self,n:usize)->usize{n+7}}fn main(){check(P.add(1)==8);}
