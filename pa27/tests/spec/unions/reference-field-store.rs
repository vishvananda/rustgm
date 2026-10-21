struct S{x:u32}union U<'a>{r:&'a mut S,x:u32}fn f(mut u:U){u.r.x=7;}fn main(){}
