const unsafe fn helper<T:Copy>(value:T)->T{value}
pub const fn through<T:Copy>(value:T)->T{unsafe{helper(value)}}
