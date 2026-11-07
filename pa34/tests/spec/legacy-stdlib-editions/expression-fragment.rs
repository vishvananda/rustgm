macro_rules! pick { ($e:expr) => { 1 }; (const $b:block) => { 2 }; }
pub const PICKED: [u8; pick!(const { 0 })] = [0, 0];
