pub fn forward<T:Copy>(value:T)->T{base::through(value)}
pub const fn array<const N:usize>()->[u8;N]{base::array::<N>()}
pub fn invoke<T:base::Step>(value:T)->u8{base::invoke(value)}
