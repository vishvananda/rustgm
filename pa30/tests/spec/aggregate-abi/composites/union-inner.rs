#[repr(C)]union Inner{a:u64,b:f64}#[repr(C)]struct Value{inner:Inner}extern "C" fn echo(x:Value)->Value{x}fn main(){let _=echo(Value{inner:Inner{a:7}});}
