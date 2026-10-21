unsafe extern {unsafe fn imported(x:i32)->i32;}fn main(){let p:unsafe extern "C" fn(i32)->i32=imported;let _=unsafe{p(39)};}
