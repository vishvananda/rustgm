fn value()->u8{7} fn main(){let p:fn()->u8=value;let r=&p;let f=||r();let _:u8=f();}
