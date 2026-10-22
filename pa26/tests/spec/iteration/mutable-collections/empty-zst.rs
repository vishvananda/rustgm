fn check(b:bool){if !b{let z=0;let _=1/z;}}struct Z;fn main(){let mut empty:[Z;0]=[];let mut n=0u8;for _ in &mut empty{n=n+1u8;}let mut a=[Z,Z,Z];for _ in a.iter_mut(){n=n+1u8;}check(n==3);}
