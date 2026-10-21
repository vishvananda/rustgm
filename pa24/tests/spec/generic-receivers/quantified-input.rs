fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:for<'a>Fn(&'a u8)->&'a u8>(f:F)->F{f}fn run<F:for<'a>FnOnce(&'a u8)->&'a u8>(f:F)->u8{let n=7;*f(&n)}fn take<T:Copy>(x:T)->u8{let f=infer(move|p|{let _v=x;p});run(f)}fn main(){check(take(3u8)==7);}
