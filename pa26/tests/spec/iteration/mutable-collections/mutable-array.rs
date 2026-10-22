fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut a=[1u8,2];for x in &mut a{*x+=1u8;}check(a[0]==2&&a[1]==3);}
