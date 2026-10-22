fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn view<'a,T,const N:usize>(a:&'a[T;N])->&'a[T]{let _=a.len();a}fn main(){let a=[1u8,2,3];check(view(&a).len()==3);}
