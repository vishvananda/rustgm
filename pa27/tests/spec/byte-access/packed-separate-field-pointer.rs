#[repr(C,packed)]struct Packed{a:u8,b:u32}const N:u32={let x=[0u8,7,0,0,0];let p=unsafe{&raw const (*(&raw const x as *const Packed)).b};unsafe{*p}};fn main(){}
