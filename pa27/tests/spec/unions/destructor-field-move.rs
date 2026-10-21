union U<'a>{r:&'a mut u32,x:u32}impl Drop for U<'_>{fn drop(&mut self){}}fn f<'a>(u:U<'a>)->&'a mut u32{unsafe{u.r}}fn main(){}
