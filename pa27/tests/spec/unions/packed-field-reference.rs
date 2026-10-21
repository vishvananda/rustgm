#[repr(C,packed)]union U{x:u32,y:u8}fn f(u:&U)->&u32{unsafe{&u.x}}fn main(){}
