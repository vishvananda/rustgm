static mut X:u8=0;#[expect(static_mut_refs)]fn f(){unsafe{let _=&X;}}fn main(){}
