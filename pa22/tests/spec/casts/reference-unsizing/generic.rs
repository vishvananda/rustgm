fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<T,const N:usize>(p:&[T;N])->&[T]{p as &[T]}fn main(){let a=[3u8,5,7];let s=cast(&a);check(s[2]==7);check(match s{[_,_,_]=>true,_=>false});}
