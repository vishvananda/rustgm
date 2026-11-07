#[macro_export]
macro_rules! select { ($p:pat) => { match 0u8 { $p => 1u8, _ => 0u8 } }; }
