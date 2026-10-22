fn main(){let mut a=[1u8];let mut i=a.iter();let x=i.next();a[0]=2;let _=match x{Some(x)=>*x,None=>0};}
