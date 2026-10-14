fn f<const N:usize>(a:&mut [u8;N],i:usize){let r=&mut a[i];a[i]=2;*r=3;}
fn main(){}
