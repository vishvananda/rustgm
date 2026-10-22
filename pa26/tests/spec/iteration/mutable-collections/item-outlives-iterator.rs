fn check(b:bool){if !b{let z=0;let _=1/z;}}struct V(u8);fn main(){let mut a=[V(1)];let x={let mut i=a.iter_mut();match i.next(){Some(x)=>x,None=>return}};x.0=7;check(a[0].0==7);}
