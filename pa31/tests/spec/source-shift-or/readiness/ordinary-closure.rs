pub fn closure(value: usize) -> usize { let f = |n: usize| n << 1 | 1; f(value) }
pub fn inline_constant() -> usize { const { 3 << 1 | 1 } }
