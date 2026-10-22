fn check(b:bool){if !b{let z=0;let _=1/z;}}fn fill(a:&mut[u8]){for x in &mut *a{*x+=1u8;}check(a[0]==2);}fn main(){let mut a=[1u8];fill(&mut a);}
