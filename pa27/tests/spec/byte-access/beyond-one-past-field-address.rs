#[repr(C)]struct Triple{a:u32,b:u32,c:u32}static X:u32=7;const P:*const u32=unsafe{&raw const (*(&raw const X as *const Triple)).c};fn main(){}
