fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut a=[1u8,2];let mut i=a.iter_mut();let x=i.next();a[0]=3;match x{Some(x)=>check(*x==1),None=>()}}
