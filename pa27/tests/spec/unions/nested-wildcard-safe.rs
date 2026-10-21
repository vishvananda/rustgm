#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,x:u32}fn f(u:U){let U{s:S{x:_,y:_}}=u;}fn main(){}
