#[derive(Copy,Clone)]#[repr(align(8))]union U{x:u8,y:u32}#[repr(packed)]struct S{u:U}fn main(){}
