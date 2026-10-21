union U{x:u8,b:bool}const X:bool=unsafe{U{x:3}.b};struct S([u8;X as usize]);fn main(){}
