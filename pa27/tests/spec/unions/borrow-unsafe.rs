union U{x:u32,y:u32}fn f(u:&U)->&u32{unsafe{&u.x}}fn main(){}
