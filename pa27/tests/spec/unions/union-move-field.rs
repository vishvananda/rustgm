union U<'a>{x:&'a mut u32,y:u32}fn f<'a>(u:U<'a>)->&'a mut u32{unsafe{u.x}}fn main(){}
