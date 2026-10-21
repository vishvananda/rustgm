#[repr(packed)]struct S{a:u8,b:u32}fn f(s:&S)->*const u32{&raw const s.b}fn main(){}
