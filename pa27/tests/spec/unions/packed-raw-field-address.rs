#[repr(C,packed)]union U{x:u32,y:u8}fn f(u:&U)->*const u32{&raw const u.x}fn main(){}
