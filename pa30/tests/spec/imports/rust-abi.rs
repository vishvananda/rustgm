unsafe extern "Rust"{safe fn imported(x:i32)->i32;}fn main(){let p:fn(i32)->i32=imported;let _=p(39);}
