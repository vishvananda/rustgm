#[repr(C)]struct A{a:[u32;2]}#[repr(C)]struct B{a:[u32;2]}extern "C" fn echo(x:A)->A{x}fn main(){let _=echo(B{a:[1,2]});}
