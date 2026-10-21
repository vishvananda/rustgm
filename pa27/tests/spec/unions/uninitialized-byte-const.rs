union U{x:u8,y:u32}const X:u32=unsafe{U{x:1}.y};struct S([u8;X as usize]);fn main(){}
