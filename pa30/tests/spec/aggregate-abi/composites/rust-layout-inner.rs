struct Inner{a:u8,b:u32}#[repr(C)]struct Value{inner:Inner}extern "C" fn echo(x:Value)->Value{x}fn main(){let _=echo(Value{inner:Inner{a:1,b:2}});}
