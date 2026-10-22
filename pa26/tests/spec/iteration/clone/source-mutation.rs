struct V(u8);fn main(){let mut a=[V(1)];let i=a.iter();let mut j=i.clone();a[0]=V(2);let _=j.next();}
