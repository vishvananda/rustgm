#[repr(C)]struct Pair{a:u32,b:u32}const N:u32={let x=7u32;unsafe{(*(&raw const x as *const Pair)).b}};fn main(){}
