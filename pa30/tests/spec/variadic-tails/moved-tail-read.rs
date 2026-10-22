unsafe extern "C"{fn ignores_tail(tag:u32,...)->u32;}
#[repr(C)]struct Pair{a:u64,b:f64}fn main(){let p=Pair{a:7,b:1.5};unsafe{ignores_tail(3,p)};let read=p.a;let _=read;}
