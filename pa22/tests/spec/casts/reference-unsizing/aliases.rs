fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T,const N:usize>=[T;N];type S<'a,T>=&'a [T];fn cast<'a>(p:&'a A<u16,2>)->S<'a,u16>{p as S<'a,u16>}fn main(){let a=[300,500];let s=cast(&a);check(s[1]==500);}
