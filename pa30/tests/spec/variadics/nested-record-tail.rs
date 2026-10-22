#[repr(C)]struct Inner{a:u32,b:f32}#[repr(C)]struct Value{head:u32,a:[Inner;1],tail:f32}unsafe extern "C"{fn nested_tail(tag:u32,...)->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(unsafe{nested_tail(0,Value{head:7,a:[Inner{a:11,b:1.5}],tail:2.5})}==1);}
