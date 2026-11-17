macro_rules! values { () => { 1u8; 2u8; }; }
macro_rules! forward { ($value:expr) => { $value }; }
fn main() { forward!(values!()); }
