#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,x:u32}fn f(mut u:U){u.s.x=7;}fn main(){}
