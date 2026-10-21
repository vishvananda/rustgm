fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]struct Pair{a:u32,b:f64}
unsafe extern "C"{
 #[link_name="provided_readonly"]static SHARED:u32;
 #[link_name="provided_readonly"]safe static SAFE:u32;
 #[link_name="provided_mutable"]safe static mut MUTABLE:u32;
 #[link_name="provided_float"]safe static FLOAT:f64;
 #[link_name="provided_pair"]safe static PAIR:Pair;
 #[link_name="provided_callback"]safe static CALLBACK:extern "C" fn(u32)->u32;
 #[link_name="provided_pointer"]static POINTER:*const u32;
}
fn main(){check(unsafe{SHARED}==7);check(SAFE==7);check(FLOAT==1.25);check(PAIR.a==7);check(PAIR.b==2.5);check(CALLBACK(39)==42);unsafe{MUTABLE=11;check(MUTABLE==11);check(*POINTER==7);}}
