static mut X:u8=0;#[allow(static_mut_refs)]fn f(){unsafe{let ref r=X;let _=r;}}fn main(){}
