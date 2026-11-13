fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn take<'a,T,F:Fn(&'a T)->&'a T>(x:&'a T,f:F)->&'a T{f(x)}fn f<T>(x:&T)->&T{let c=|s|s;take(x,c)}
fn main(){let x=7u8;check(*f(&x)==7);}
