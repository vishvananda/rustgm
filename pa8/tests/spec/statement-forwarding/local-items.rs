macro_rules! values { () => { const A: u8 = 1; const B: u8 = 2; }; }
macro_rules! forward { () => { values!() }; }
fn main() { forward!(); }
