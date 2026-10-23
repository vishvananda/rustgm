const fn helper<T:Copy>(value:T)->T{value}
pub const fn through<T:Copy>(value:T)->T{helper(value)}
pub const fn reserve<T:Copy,U:Copy>(value:T)->T{helper(value)}
