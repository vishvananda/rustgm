macro_rules! assert { ($ok:expr) => { check($ok) }; }
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
extern crate cycle;fn main(){assert!(cycle::NODE.next.value==41);assert!((&cycle::NODE as *const cycle::Node)==(cycle::NODE.next as *const cycle::Node));}
