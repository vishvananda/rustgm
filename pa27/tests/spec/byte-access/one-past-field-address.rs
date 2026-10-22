#[repr(C)]struct Pair{a:u32,b:u32}static X:u32=7;const P:*const u32=unsafe{&raw const (*(&raw const X as *const Pair)).b};fn main(){}
