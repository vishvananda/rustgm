pub const fn id<T:Copy>(value:T)->T{value}
fn helper<T:Copy>(value:T)->T{value}
pub fn through<T:Copy>(value:T)->T{helper(value)}
pub fn reserve<T:Copy,U:Copy>(value:T)->T{value}
pub fn reborrow<'a,T>(value:&'a T)->&'a T{value}
pub const fn array<const N:usize>()->[u8;N]{[7;N]}
pub trait Step{fn step(self)->u8;}
pub fn invoke<T:Step>(value:T)->u8{value.step()}
