static S:u32=37;static mut COUNT:u32=0;#[repr(transparent)]struct R(&'static u32);impl Drop for R{fn drop(&mut self){unsafe{COUNT+=1}}}fn consume(_:Option<R>){}fn main(){consume(Some(R(&S)));consume(None);check(unsafe{COUNT}==1)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
