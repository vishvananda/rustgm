macro_rules! values { () => { 1u8; 2u8; }; }
macro_rules! forward { () => { values![] }; }
fn main() { forward![]; }
