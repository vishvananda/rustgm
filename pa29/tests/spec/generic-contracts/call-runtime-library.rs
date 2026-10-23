fn helper<T:Copy>(value:T)->T{value}
pub fn through<T:Copy>(value:T)->T{helper(value)}
pub fn reserve<T:Copy,U:Copy>(value:T)->T{helper(value)}
