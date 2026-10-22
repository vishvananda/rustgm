fn check(b:bool){if !b{let z=0;let _=1/z;}}struct W<const N:usize>;fn take(_:W<{u8::MAX as usize}>)->u8{3}fn main(){check(take(W::<255>)==3);}
