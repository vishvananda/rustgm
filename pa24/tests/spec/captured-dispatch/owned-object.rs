struct V(u8);fn main(){let x=V(2);let f=move||{let y=x;y.0};let p:&dyn Fn()->u8=&f;let _=p();}
