#[repr(C)]struct A{a:[u32;2]}unsafe extern "C"{safe fn echo(x:A)->A;}fn main(){let _=echo(A{a:[1,2]},3);}
