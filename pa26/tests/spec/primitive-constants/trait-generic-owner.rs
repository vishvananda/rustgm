fn check(b:bool){if !b{let z=0;let _=1/z;}}trait K{const MAX:u8;}impl K for u32{const MAX:u8=7;}fn get<T:K>()->u8{T::MAX}fn main(){check(get::<u32>()==7);}
