#[derive(Copy,Clone)]struct S{x:u32,y:u32}union U{s:S,x:u32}fn f(u:&U)->*const u32{&raw const u.s.x}fn main(){}
