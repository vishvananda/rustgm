fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<const N:usize,F:for<'a>Fn(&'a [u8;N])->&'a [u8;N]>(f:F,x:[u8;N])->F{let _=x;f}fn main(){let f=infer(|x|x,[1u8,2]);let n=[7u8,9];check(f(&n)[0]==7);}
