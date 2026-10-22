fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut a=[1u8,2];for x in a.iter_mut(){*x+=2u8;}check(a[0]==3&&a[1]==4);}
