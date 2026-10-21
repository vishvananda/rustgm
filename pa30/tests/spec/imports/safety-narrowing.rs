unsafe extern "C"{unsafe fn imported(x:i32)->i32;}fn main(){let _:extern "C" fn(i32)->i32=imported;}
