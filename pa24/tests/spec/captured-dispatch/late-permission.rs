fn main(){let x=2u8;let f=||x;let p:&dyn Fn()->u8=&f;let _=p();let q:&'static dyn Fn()->u8=&f;let _=q();}
