macro_rules! values { () => {}; }
macro_rules! forward { () => { values!() }; }
fn main() { forward!(); }
