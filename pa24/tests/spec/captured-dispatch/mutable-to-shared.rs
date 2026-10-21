fn main(){let mut x=0u8;let mut f=||{x+=1;x};let p:&dyn Fn()->u8=&f;let _=p();}
