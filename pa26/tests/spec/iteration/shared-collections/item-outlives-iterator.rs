fn check(b:bool){if !b{let z=0;let _=1/z;}}fn first<'a>(s:&'a[u8])->&'a u8{let mut i=s.iter();match i.next(){Some(x)=>x,None=>{let z=0;let _=1/z;&s[0]}}}fn main(){check(*first(&[7u8])==7);}
