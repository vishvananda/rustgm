fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn a(x:usize,y:usize)->usize{x+y} fn b(x:isize,y:isize)->isize{x*y} fn main(){check(a(18446744073709551614,1)==18446744073709551615);check(b(-13,7)== -91);check((-1isize) as usize == 18446744073709551615);}
