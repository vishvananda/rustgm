fn check(b:bool){if !b{let z=0;let _=1/z;}}const M:u128=u128::MAX;fn main(){check(M==340282366920938463463374607431768211455u128);check(u128::MAX==M);}
