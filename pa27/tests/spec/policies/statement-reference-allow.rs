static mut X:u8=0;fn f(){unsafe{#[allow(static_mut_refs)]let _=&X;}}fn main(){}
