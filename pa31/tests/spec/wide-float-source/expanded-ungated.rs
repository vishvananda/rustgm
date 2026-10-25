macro_rules! demanded { () => { 1.0_f128 }; }
pub fn live() { let _ = demanded!(); }
