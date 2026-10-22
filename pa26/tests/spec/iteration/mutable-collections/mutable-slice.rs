fn check(b:bool){if !b{let z=0;let _=1/z;}}fn fill(a:&mut[u8]){for x in a{*x=7u8;}}fn main(){let mut a=[1u8,2];fill(&mut a);check(a[0]==7&&a[1]==7);}
