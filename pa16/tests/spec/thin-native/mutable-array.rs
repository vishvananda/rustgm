fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=&mut [1u8,2,3];a[1]+=5;check(a[0]==1 && a[1]==7 && a[2]==3);}
