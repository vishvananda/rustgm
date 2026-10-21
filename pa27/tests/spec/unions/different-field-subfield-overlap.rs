#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,t:S}fn f(u:&mut U){unsafe{let x=&mut u.s.x;let y=&mut u.t.y;*x=7;*y=9;}}fn main(){}
