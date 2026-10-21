#[derive(Copy,Clone)]#[repr(align(8))]struct A(u8);#[repr(packed)]union U{a:A,x:u8}fn main(){}
