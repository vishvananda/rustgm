fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>{r:&'a mut u8}fn inc(s:S<'_>){*s.r+=1}fn main(){let mut x=41;inc(S{r:&mut x});check(x==42);}
