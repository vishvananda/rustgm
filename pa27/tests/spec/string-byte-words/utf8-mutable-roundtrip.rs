const fn view_mut(v:&mut [u8])->&mut str{unsafe{&mut *(v as *mut [u8] as *mut str)}}
union U{a:*mut str,b:*mut str}const N:u8={let mut b=[65u8,66];let s=view_mut(&mut b);let p=unsafe{U{a:s as *mut str}.b};unsafe{*(p as *mut u8)=90;}b[0]};fn main(){if N!=90{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
