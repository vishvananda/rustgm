macro_rules! assert { ($ok:expr) => { check($ok) }; }
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
extern crate hidden;
fn main(){assert!(*hidden::FIELD==31);assert!(hidden::WHOLE.value==31);assert!((hidden::FIELD as *const u32)==(hidden::ALIAS as *const u32));assert!((hidden::FIELD as *const u32)==(&hidden::WHOLE.value as *const u32));}
