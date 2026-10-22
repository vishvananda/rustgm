fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let a=[3u8,8];let mut i=a.iter();let x=match i.next(){Some(x)=>x,None=>&a[0]};let y=match i.next(){Some(x)=>x,None=>&a[0]};check(*x==3&&*y==8);}
