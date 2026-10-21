fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>{n:&'a mut u8}impl<'a> Drop for D<'a>{fn drop(&mut self){*self.n+=1;}}fn main(){let mut count=0u8;{let value=D{n:&mut count};let f=move||{let _=&value;7u8};let p:&dyn Fn()->u8=&f;check(p()==7);check(p()==7);}check(count==1);}
