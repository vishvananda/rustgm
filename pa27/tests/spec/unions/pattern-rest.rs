union U{x:u32,y:u32}fn f(u:U)->u32{unsafe{let U{x,..}=u;x}}fn main(){}
