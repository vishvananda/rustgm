union U{x:u32,y:u32}fn f(u:&mut U){unsafe{let x=&mut u.x;let y=&mut u.y;*x=7;*y=9;}}fn main(){}
