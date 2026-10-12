fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
macro_rules! run {($n:expr)=>{{let mut x=0;while x<$n{x+=1;}x}}} fn main(){check(run!(42)==42);}
