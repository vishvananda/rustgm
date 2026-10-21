union U<'a>{x:&'a mut u32,y:u32}fn f<'a>(u:U<'a>){unsafe{let x=u.x;let y=u.x;*x=7;*y=9;}}fn main(){}
